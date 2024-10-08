use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use colored::Colorize;

use crate::logger::LogEntry;
use crate::node::ProcessEntry;
use crate::{Context, EventLogEntry, Message, ProcessEvent, ProcessState, TimerBehavior};

use crate::mc::network::DeliveryOptions;
use crate::mc::{McEvent, McTime, TraceHandler};

#[derive(Debug, Clone)]
pub struct ProcessEntryState {
    pub proc_state: Rc<dyn ProcessState>,
    pub event_log: Vec<EventLogEntry>,
    pub local_outbox: Vec<Message>,
    pub pending_timers: HashMap<String, u64>,
    pub sent_message_count: u64,
    pub received_message_count: u64,
}

impl Hash for ProcessEntryState {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.proc_state.hash_with_dyn(hasher);
        self.local_outbox.hash(hasher);
    }
}

impl PartialEq for ProcessEntryState {
    fn eq(&self, other: &Self) -> bool {
        let equal_process_states = self.proc_state.eq_with_dyn(&*other.proc_state);
        equal_process_states && self.local_outbox == other.local_outbox
    }
}

impl Eq for ProcessEntryState {
    fn assert_receiver_is_total_eq(&self) {}
}

impl ProcessEntry {
    fn get_state(&self) -> Result<ProcessEntryState, String> {
        Ok(ProcessEntryState {
            proc_state: self.proc_impl.state()?,
            event_log: self.event_log.clone(),
            local_outbox: self.local_outbox.clone(),
            pending_timers: self.pending_timers.clone(),
            sent_message_count: self.sent_message_count,
            received_message_count: self.received_message_count,
        })
    }

    fn set_state(&mut self, state: ProcessEntryState) -> Result<(), String> {
        self.proc_impl.set_state(state.proc_state)?;
        self.event_log = state.event_log;
        self.local_outbox = state.local_outbox;
        self.pending_timers = state.pending_timers;
        self.sent_message_count = state.sent_message_count;
        self.received_message_count = state.received_message_count;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct McNodeState {
    pub proc_states: BTreeMap<String, ProcessEntryState>,
    is_crashed: bool,
}

#[derive(Clone)]
pub struct McNode {
    name: String,
    pub(crate) processes: HashMap<String, ProcessEntry>,
    trace_handler: Rc<RefCell<TraceHandler>>,
    clock_skew: f64,
    is_crashed: bool,
}

impl McNode {
    pub(crate) fn new(
        name: String,
        processes: HashMap<String, ProcessEntry>,
        trace_handler: Rc<RefCell<TraceHandler>>,
        clock_skew: f64,
    ) -> Self {
        Self {
            name,
            processes,
            trace_handler,
            clock_skew,
            is_crashed: false,
        }
    }

    pub fn on_message_received(
        &mut self,
        proc: String,
        msg: Message,
        from: String,
        time: f64,
        random_seed: u64,
    ) -> Vec<McEvent> {
        assert!(!self.is_crashed, "should not receive message on crashed node");
        let proc_entry = self.processes.get_mut(&proc).unwrap();
        proc_entry.event_log.push(EventLogEntry::new(
            0.0,
            ProcessEvent::MessageReceived {
                msg: msg.clone(),
                src: from.clone(),
                dst: proc.clone(),
            },
        ));
        proc_entry.received_message_count += 1;

        let mut proc_ctx = Context::basic(proc.to_string(), time, self.clock_skew, random_seed);

        proc_entry
            .proc_impl
            .on_message(msg, from, &mut proc_ctx)
            .map_err(|e| self.handle_process_error(e, proc.clone()))
            .unwrap();

        self.handle_process_actions(proc, 0.0, proc_ctx.actions())
    }

    pub fn on_timer_fired(&mut self, proc: String, timer: String, time: f64, random_seed: u64) -> Vec<McEvent> {
        assert!(!self.is_crashed, "should not fire timer on crashed node");
        let proc_entry = self.processes.get_mut(&proc).unwrap();
        proc_entry.pending_timers.remove(&timer);

        let mut proc_ctx = Context::basic(proc.to_string(), time, self.clock_skew, random_seed);

        proc_entry
            .proc_impl
            .on_timer(timer, &mut proc_ctx)
            .map_err(|e| self.handle_process_error(e, proc.clone()))
            .unwrap();

        self.handle_process_actions(proc, 0.0, proc_ctx.actions())
    }

    pub fn on_local_message_received(
        &mut self,
        proc: String,
        msg: Message,
        time: f64,
        random_seed: u64,
    ) -> Vec<McEvent> {
        assert!(!self.is_crashed, "should not receive local message on crashed node");
        let proc_entry = self.processes.get_mut(&proc).unwrap();
        let mut proc_ctx = Context::basic(proc.to_string(), time, self.clock_skew, random_seed);

        proc_entry
            .proc_impl
            .on_local_message(msg, &mut proc_ctx)
            .map_err(|e| self.handle_process_error(e, proc.clone()))
            .unwrap();

        self.handle_process_actions(proc, time, proc_ctx.actions())
    }

    pub fn get_state(&self) -> McNodeState {
        let proc_states = self
            .processes
            .iter()
            .map(|(proc, entry)| {
                (
                    proc.clone(),
                    entry
                        .get_state()
                        .map_err(|e| self.handle_process_error(e, proc.clone()))
                        .unwrap(),
                )
            })
            .collect();
        McNodeState {
            proc_states,
            is_crashed: self.is_crashed,
        }
    }

    pub fn set_state(&mut self, state: McNodeState) {
        for (proc, state) in state.proc_states {
            self.processes
                .get_mut(&proc)
                .unwrap()
                .set_state(state)
                .map_err(|e| self.handle_process_error(e, proc.clone()))
                .unwrap();
        }
        self.is_crashed = state.is_crashed;
    }

    pub(crate) fn crash(&mut self) {
        self.is_crashed = true;
    }

    fn handle_process_actions(&mut self, proc: String, time: f64, actions: Vec<ProcessEvent>) -> Vec<McEvent> {
        let mut new_events = Vec::new();
        for action in actions {
            let proc_entry = self.processes.get_mut(&proc).unwrap();
            proc_entry.event_log.push(EventLogEntry::new(time, action.clone()));
            match action {
                ProcessEvent::MessageSent { msg, src, dst } => {
                    new_events.push(McEvent::MessageReceived {
                        msg: msg.clone(),
                        src: src.clone(),
                        dst: dst.clone(),
                        options: DeliveryOptions::NoFailures(0.0.into()),
                    });
                    proc_entry.sent_message_count += 1;

                    let log_entry = LogEntry::McMessageSent { msg, src, dst };
                    self.trace_handler.borrow_mut().push(log_entry);
                }
                ProcessEvent::LocalMessageSent { msg } => {
                    proc_entry.local_outbox.push(msg.clone());

                    let log_entry = LogEntry::McLocalMessageSent {
                        msg,
                        proc: proc.clone(),
                    };
                    self.trace_handler.borrow_mut().push(log_entry);
                }
                ProcessEvent::TimerSet { name, delay, behavior } => {
                    if behavior == TimerBehavior::OverrideExisting || !proc_entry.pending_timers.contains_key(&name) {
                        let event = McEvent::TimerFired {
                            timer: name.clone(),
                            proc: proc.clone(),
                            timer_delay: McTime::from(delay),
                        };
                        new_events.push(event);
                        // event_id is 0 since it is not used in model checking
                        proc_entry.pending_timers.insert(name.clone(), 0);

                        let log_entry = LogEntry::McTimerSet {
                            proc: proc.clone(),
                            timer: name,
                        };
                        self.trace_handler.borrow_mut().push(log_entry);
                    }
                }
                ProcessEvent::TimerCancelled { name } => {
                    if proc_entry.pending_timers.remove(&name).is_some() {
                        let event = McEvent::TimerCancelled {
                            timer: name.clone(),
                            proc: proc.clone(),
                        };
                        new_events.push(event);

                        let log_entry = LogEntry::McTimerCancelled {
                            proc: proc.clone(),
                            timer: name,
                        };
                        self.trace_handler.borrow_mut().push(log_entry);
                    }
                }
                _ => {}
            }
        }
        new_events
    }

    fn handle_process_error(&self, err: String, proc: String) -> &str {
        for event in self.trace_handler.borrow().trace() {
            event.print();
        }
        eprintln!(
            "{}",
            format!(
                "\n!!! Error when calling process '{}' on node '{}':\n\n{}",
                proc, self.name, err
            )
            .red()
        );
        "Error when calling process"
    }
}

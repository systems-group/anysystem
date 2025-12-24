use std::rc::Rc;

use anysystem::{Context, Message, Process, ProcessState};

#[derive(Clone)]
pub struct RetryPingClient {
    server: String,
    ping: Option<Message>,
}

impl RetryPingClient {
    pub fn new(server: &str) -> Self {
        Self {
            server: server.to_string(),
            ping: None,
        }
    }
}

impl Process for RetryPingClient {
    fn on_start(&mut self, _ctx: &mut Context) -> Result<(), String> {
        Ok(())
    }

    fn on_message(&mut self, msg: Message, _from: String, ctx: &mut Context) -> Result<(), String> {
        if msg.tip == "PONG" {
            self.ping = None;
            ctx.cancel_timer("check-pong");
            ctx.send_local(msg);
        }
        Ok(())
    }

    fn on_local_message(&mut self, msg: Message, ctx: &mut Context) -> Result<(), String> {
        if msg.tip == "PING" {
            self.ping = Some(msg.clone());
            ctx.send(msg, self.server.clone());
            ctx.set_timer("check-pong", 3.);
        }
        Ok(())
    }

    fn on_timer(&mut self, timer: String, ctx: &mut Context) -> Result<(), String> {
        if timer == "check-pong" && self.ping.is_some() {
            ctx.send(self.ping.as_ref().unwrap().clone(), self.server.clone());
            ctx.set_timer("check-pong", 3.);
        }
        Ok(())
    }

    fn state(&self) -> Result<Rc<dyn ProcessState>, String> {
        Ok(Rc::new(self.ping.clone()))
    }

    fn set_state(&mut self, state: Rc<dyn ProcessState>) -> Result<(), String> {
        self.ping
            .clone_from(state.downcast_rc::<Option<Message>>().unwrap().as_ref());
        Ok(())
    }
}

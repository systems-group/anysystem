//! AnySystem is a framework for deterministic simulation and testing of distributed systems. It allows to model
//! arbitrary systems represented as a set of _processes_ running on a set of _nodes_ connected by a _network_.
//!
//! The processes communicate with each other by sending and receiving _messages_. A message between processes located
//! on different nodes is transmitted over the network. A process can also receive and send _local messages_ which can
//! be used to model the interaction with external entities such as users. Finally, a process can schedule _timers_ to
//! call itself after a specified delay. You can implement arbitrary logic for processing of incoming messages and
//! timers for each process. In addition to Rust, it is possible to implement processes in Python.
//!
//! The system is built by creating the required nodes and processes, and binding each process to some node. Then it is
//! possible to send some messages to initiate the system execution. The execution is implemented as a step-by-step
//! _simulation_. Each step corresponds to the occurrence of some event such as message delivery or timer firing. The
//! events are processed in the order of their timestamps by advancing the simulation time and calling a corresponding
//! process. In response, the process can perform actions that produce new events, e.g. sending a message can produce
//! a new message delivery event. Currently, the process execution time is not modeled, i.e. it is assumed that events
//! are processed instantaneously. The system execution is deterministic by using a common RNG seeded with
//! a user-defined value.
//!
//! AnySystem supports modeling of typical situations found in distributed systems, such as message delays, network
//! failures and node crashes. The provided network model can be configured to introduce fixed or random transmission
//! delays, message loss, duplication or corruption. It is also possible to model node disconnections, control
//! the network between each pair of nodes and in each direction, or introduce network partitions. A node can be crashed
//! by disconnecting it from the network and stopping all processes running on it. A crashed node can be recovered later
//! by restarting its processes and connecting it to the network.
//!
//! The described features can be used for testing any distributed algorithm or application implemented as a set of
//! processes. Step-by-step simulation allows to precisely control the system execution and produce different execution
//! scenarios by sending messages and introducing failures at specific points in time. It also allows to check the state
//! of each process or the global state for correctness at any time. If the error is found, the trace of system
//! execution can be output for debugging. Thanks to deterministic simulation, the erroneous execution can be reliably
//! reproduced until the error is fixed.
//!
//! In addition to simulation-based testing, AnySystem supports _model checking_. This approach is based on  traversing
//! the graph of possible system states, starting from a given initial state. A system state is determined by the states
//! of processes, nodes and the network, and by the list of pending events. Arcs in the graph correspond to transitions
//! between the states caused by events. Each system execution corresponds to a path in this graph. While simulation
//! checks just a single execution path, model checking allows to explore all possible execution paths and exhaustively
//! test the system. However, in practice this approach is often limited by a large size of system states graph.
//! AnySystem allows to flexibly configure the model checking strategy to trade the exhaustiveness for reasonable
//! execution time.

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod context;
pub mod events;
pub mod logger;
pub mod mc;
pub mod message;
pub mod network;
pub mod node;
pub mod process;
pub mod python;
pub mod system;
pub mod test;
mod util;

pub use context::Context;
pub use message::Message;
pub use network::Network;
pub use node::{EventLogEntry, Node, ProcessEvent, TimerBehavior};
pub use process::{Process, ProcessState};
pub use system::System;

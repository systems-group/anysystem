#![warn(missing_docs)]

//! Model checking support.

mod dependency;
pub mod error;
pub mod events;
pub mod model_checker;
pub mod network;
mod node;
mod pending_events;
pub mod predicates;
pub mod state;
pub mod strategies;
pub mod strategy;
pub mod system;
mod trace_handler;
mod util;

use dependency::DependencyResolver;
pub use error::McError;
pub use events::{EventOrderingMode, McEvent, McEventId};
pub use model_checker::ModelChecker;
pub use network::McNetwork;
use node::{McNode, McNodeState};
use pending_events::PendingEvents;
pub use state::McState;
pub use strategy::*;
pub use system::{McSystem, McTime};
use trace_handler::TraceHandler;

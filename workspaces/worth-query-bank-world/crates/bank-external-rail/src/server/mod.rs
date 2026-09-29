//! The Bank external rail's own process-local server: listener, ledger, and
//! per-connection dispatch.

mod completed_effects;
mod completion_delivery;
mod dispatch;
mod fault_behavior;
mod ledger;
mod listener;

pub use completion_delivery::{
    RailCompletionDeliveryConfiguration, RailCompletionDeliveryConfigurationDenial,
    RailCompletionDeliveryPosture,
};
pub use listener::RailServer;

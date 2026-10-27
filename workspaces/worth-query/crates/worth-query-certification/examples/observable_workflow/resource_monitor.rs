//! A separate recoverable effect demonstrates process-local monitoring custody.
#[path = "resource_monitor/authority.rs"]
mod authority;
#[path = "resource_monitor/declaration.rs"]
mod declaration;
#[path = "resource_monitor/handler.rs"]
mod handler;
#[path = "resource_monitor/journey.rs"]
mod journey;
#[path = "../../tests/application_graph/source_only/capability/resources/provider.rs"]
pub mod producer;
#[path = "resource_monitor/resource.rs"]
mod resource;
#[path = "resource_monitor/source_binding.rs"]
mod source_binding;
pub use declaration::{MonitorBinding, MonitorContribution};
pub use journey::run;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct Configuration {
    pub state: Arc<producer::ProducerState>,
    clock: Arc<producer::ClockSource>,
}

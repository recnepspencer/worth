//! A separate recoverable effect demonstrates process-local monitoring custody.
#[path = "resource_monitor/authority.rs"]
mod authority;
#[path = "resource_monitor/declaration.rs"]
mod declaration;
#[path = "resource_monitor/handler.rs"]
mod handler;
#[path = "resource_monitor/journey.rs"]
mod journey;

pub use worth_query_testkit_certification_operation::facade::certification::native_workflow_operations::capability::resources::provider as producer;
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

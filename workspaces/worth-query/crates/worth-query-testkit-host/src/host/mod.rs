//! One integration executable; scenario support is shared by its cost-class courts.
#[allow(dead_code, unused_imports)]
#[path = "temporal_conditional_operation/adapters.rs"]
pub mod adapters;
#[allow(dead_code, unused_imports)]
#[path = "application_entry_current_read.rs"]
pub mod application_entry_current_read;
#[allow(dead_code, unused_imports)]
#[path = "application_runtime_surface.rs"]
pub mod application_runtime_surface;
#[allow(dead_code, unused_imports)]
#[path = "canonical_graph_progression.rs"]
pub mod canonical_graph_progression;
#[allow(dead_code, unused_imports)]
#[path = "charged_bytes_derive.rs"]
pub mod charged_bytes_derive;
#[path = "support/checkpoint_capture.rs"]
pub mod checkpoint_capture;
#[allow(dead_code, unused_imports)]
#[path = "conditional_authoring_surface.rs"]
pub mod conditional_authoring_surface;
#[allow(dead_code, unused_imports)]
#[path = "temporal_conditional_operation/contract.rs"]
pub mod contract;
#[allow(dead_code, unused_imports)]
#[path = "installed_contract_inspection.rs"]
pub mod installed_contract_inspection;
#[allow(dead_code, unused_imports)]
#[path = "support/process_deadline.rs"]
pub mod process_deadline;
#[allow(dead_code, unused_imports)]
#[path = "temporal_conditional_operation/schema.rs"]
pub mod schema;
#[allow(dead_code, unused_imports)]
#[path = "temporal_conditional_operation.rs"]
pub mod temporal_conditional_operation;
#[allow(dead_code, unused_imports)]
#[path = "support/execution_policy.rs"]
pub mod test_request_policy;
#[allow(dead_code, unused_imports)]
#[path = "temporal_conditional_operation/world.rs"]
pub mod world;
pub use worth_query_execution::facade;

pub mod certification_support;

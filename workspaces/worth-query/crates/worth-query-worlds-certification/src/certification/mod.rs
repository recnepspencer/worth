//! One integration executable; scenario support is shared by its cost-class courts.
pub use crate::facade::host::adapters;
#[allow(dead_code, unused_imports)]
#[path = "application_graph/canonical_identity.rs"]
pub mod canonical_identity;
#[allow(dead_code, unused_imports)]
#[path = "application_graph/canonical_transition.rs"]
pub mod canonical_transition;
pub use crate::facade::host::checkpoint_capture;
pub use crate::facade::host::contract;
pub use crate::facade::host::temporal_conditional_operation::courtroom_support;
#[allow(dead_code, unused_imports)]
#[path = "granular_invalidation/delivery_convergence.rs"]
pub mod delivery_convergence;
pub use document_world_resources as world_resources;
#[allow(dead_code, unused_imports)]
pub use worth_query_worlds_document_retention::facade::document_retention_model;
#[allow(
    dead_code,
    unused_imports,
    reason = "Shared reference application ports are exercised by independent consumer courts"
)]
#[allow(dead_code, unused_imports)]
pub use worth_query_worlds_document_retention::facade::document_world_resources;
#[allow(dead_code, unused_imports)]
#[path = "granular_invalidation/financial_runtime_world.rs"]
pub mod financial_runtime_world;
pub use crate::facade::host::world;
pub use world as host_world;
#[allow(dead_code, unused_imports)]
#[path = "application_graph.rs"]
pub mod application_graph;

#[allow(dead_code, unused_imports)]
#[path = "granular_invalidation.rs"]
pub mod granular_invalidation;
#[allow(dead_code, unused_imports)]
#[path = "native_graph_input_guards.rs"]
pub mod native_graph_input_guards;
#[allow(dead_code, unused_imports)]
#[path = "native_operation_bodies.rs"]
pub mod native_operation_bodies;
#[allow(dead_code, unused_imports)]
#[path = "native_workflow_operations.rs"]
pub mod native_workflow_operations;
#[allow(
    dead_code,
    unused_imports,
    reason = "Shared reference application ports are exercised by independent consumer courts"
)]
pub use document_retention_model::operator_identity;
#[allow(dead_code, unused_imports)]
#[path = "native_graph_input_guards/peer_audit.rs"]
pub mod peer_audit;

#[allow(dead_code, unused_imports)]
#[path = "support/process_deadline.rs"]
pub mod process_deadline;
pub use crate::facade::host::temporal_conditional_operation::product_query_support;
#[allow(dead_code, unused_imports)]
#[path = "program_example_authority.rs"]
pub mod program_example_authority;
#[allow(dead_code, unused_imports)]
#[path = "runtime_world_branching/query_probe.rs"]
pub mod query_probe;
#[allow(dead_code, unused_imports)]
#[path = "granular_invalidation/query_runtime_world.rs"]
pub mod query_runtime_world;
#[allow(dead_code, unused_imports)]
#[path = "application_graph/relation_laws.rs"]
pub mod relation_laws;
#[allow(dead_code, unused_imports)]
#[path = "granular_invalidation/runtime_composition.rs"]
pub mod runtime_composition;
#[allow(dead_code, unused_imports)]
#[path = "runtime_world_branching.rs"]
pub mod runtime_world_branching;

pub use crate::facade::host::schema;

#[allow(dead_code, unused_imports)]
#[path = "granular_invalidation/shared_lifecycle.rs"]
pub mod shared_lifecycle;
#[allow(dead_code, unused_imports)]
#[path = "application_graph/source_only.rs"]
pub mod source_only;
#[allow(dead_code, unused_imports)]
#[path = "granular_invalidation/structural_slopes.rs"]
pub mod structural_slopes;
#[allow(dead_code, unused_imports)]
#[path = "temporal_conditional_oracle_parity.rs"]
pub mod temporal_conditional_oracle_parity;
#[allow(dead_code, unused_imports)]
#[path = "support/execution_policy.rs"]
pub mod test_request_policy;

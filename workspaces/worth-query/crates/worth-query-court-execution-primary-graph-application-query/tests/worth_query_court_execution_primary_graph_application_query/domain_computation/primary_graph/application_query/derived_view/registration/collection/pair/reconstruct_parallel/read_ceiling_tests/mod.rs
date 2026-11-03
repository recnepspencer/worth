//! Refused increments follow the map and ordinary Query-read laws independently.
pub use std::collections::BTreeMap;
pub use worth_query_execution::facade::domain_computation::WorthQueryManagedDerivedViewDenial as Denial;
pub use worth_execution::ExecutionMap;
pub use worth_execution::ExecutionRequest;
pub use worth_foundational::PartitionIdentity;
pub use worth_query_execution::facade::WorthQueryDerivedPairReadPlans;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::derived_view::reconstruction_denial as denial;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::reconstruction::owner_stage;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::*;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAccessContext;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
pub use worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls;
pub use std::num::NonZeroUsize;
pub use std::time::Duration;
pub use worth_execution::ExecutionWorkCeiling;
pub use worth_execution::MapOutcome;
pub use worth_execution::MapStop;
pub use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
pub use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
pub use worth_query_installation::facade::ApplicationScalarValueBinding;
pub use worth_relational::facade::transactions::AspectFieldPatch;
pub use worth_relational::facade::transactions::EntityMutationIntent;
pub use worth_relational::facade::transactions::MutationIntent;
pub use worth_relational::facade::transactions::UpdateEntityFieldsIntent;
pub use worth_relational::facade::transactions::WorkerIntentBatch;

pub fn current_controls(
    scope: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> WorthQueryProductQueryControls<'_> {
    WorthQueryProductQueryControls::new(NonZeroUsize::new(10).unwrap(), scope)
}

#[path = "map_ceiling/mod.rs"]
pub mod map_ceiling;
#[path = "native_concurrency/mod.rs"]
pub mod native_concurrency;
#[path = "parallel_ceiling/mod.rs"]
pub mod parallel_ceiling;
#[path = "pin_custody_tests/mod.rs"]
pub mod pin_custody_tests;
#[path = "pinning_reasons/mod.rs"]
pub mod pinning_reasons;
#[path = "reconstruction_parity/mod.rs"]
pub mod reconstruction_parity;

//! Query execution proofs grouped by their product frontier.
#![recursion_limit = "256"]
// Allocation-count courts measure through the one binary-wide stats allocator.
#[global_allocator]
static TEST_ALLOCATOR: &stats_alloc::StatsAlloc<std::alloc::System> =
    &stats_alloc::INSTRUMENTED_SYSTEM;
#[path = "worth_query_court_execution_primary_graph_application_contribution/domain_computation/mod.rs"]
pub mod domain_computation;
#[path = "worth_query_court_execution_primary_graph_application_contribution/process/mod.rs"]
pub mod process;
#[path = "worth_query_court_execution_primary_graph_application_contribution/publication_boundary/mod.rs"]
pub mod publication_boundary;

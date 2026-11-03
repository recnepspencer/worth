//! Native fork selection preserves captured generations and real local absence.
pub use std::sync::Arc;
pub use worth_query_execution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence;
pub use worth_query_execution::domain_computation::primary_graph::output_lineage::ProductCoordinate;
pub use worth_query_execution::domain_computation::primary_graph::output_lineage::retained_computation::RecordedComputation;
pub use worth_query_execution::domain_computation::primary_graph::output_lineage::semantic_source::SemanticSource;
pub use worth_query_execution::equality_type_id::EqualityTypeId as TypeId;
pub use worth_query_execution::domain_computation::primary_graph::output_lineage::WorthQueryApplicationOutputLineage;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::sealed_run_for_lineage_test;

#[path = "fork_selection/mod.rs"]
pub mod fork_selection;

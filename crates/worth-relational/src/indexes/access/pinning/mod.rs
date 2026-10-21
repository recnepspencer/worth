//! Installed entries borrow the live owner only to select immutable custody.
//! Sibling read algorithms cannot access the owner field.
use super::{DerivedIndexDefinitionLookup, PreparedEntityFieldLookup};
use crate::runtime::RelationalRuntime;
#[path = "../admitted_generation.rs"]
mod admitted_generation;
mod field_preparation;
mod inspection;
#[cfg(test)]
#[path = "../query_index_parity.rs"]
mod query_index_parity;
mod related_ordering;
mod relation_join;
mod snapshot_field;
#[cfg(test)]
use super::execution::execute_index_backed_query_from_generation;
#[cfg(test)]
use super::routing::{admissible_access_path, should_verify_sampled_parity};
#[cfg(test)]
use crate::query::data::{
    query_index_parity_basis_digest, IndexParityMode, IndexParityVerifiedQueryOutcome,
    QueryAccessContract, QueryAccessPath, SnapshotPinnedQueryPlan,
};

pub struct IndexAccess<'runtime> {
    runtime: &'runtime RelationalRuntime,
}
impl<'runtime> IndexAccess<'runtime> {
    pub(super) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }
}

#[cfg(test)]
fn record_index_execution_shape(
    runtime: &RelationalRuntime,
    execution: &crate::query::data::QueryExecutionOutcome,
) {
    let target_count = execution.complexity.target_count.max(1);
    let scope_units = execution.complexity.touched_partitions.max(1);
    runtime.performance_access().count_query_packet_shape(
        execution.complexity.packet_count.max(1),
        target_count,
        target_count,
        scope_units,
    );
    runtime.performance_access().count_query_serial_strategy();
}

#[cfg(test)]
fn record_index_execution_counters(
    runtime: &RelationalRuntime,
    execution: &crate::query::data::QueryExecutionOutcome,
) {
    record_index_execution_shape(runtime, execution);
    runtime.performance_access().count_query_emissions(
        execution.result.entities.len(),
        execution.result.relations.len(),
    );
}

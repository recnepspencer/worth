//! Observes the persistent names of the rows one kernel read materialized,
//! at the read's own basis and under the runtime the read already holds.

use std::collections::BTreeSet;

use worth_query_execution_naming::facade::{ExecutionNameReader, NameReadDependencies};
use worth_relational::facade::identity::EntityId;

use super::super::projection::WorthQueryApplicationProjectionNode;
use super::super::WorthQueryAdmittedApplicationQueryPlan;
use super::RawNonLiveKernelOutcome;

/// Reads the name every materialized row's record carries, in one pass over
/// the name index at the plan's selected basis, and lends them to the
/// projection that follows.
///
/// When the name index refuses at the basis the names stay unobserved and
/// each name slot refuses `PersistentNamesUnavailable`; the read itself and
/// rows that project no name are unaffected. A projected name adds no source
/// dependency: a record's name changes only when its lineage ends, which
/// ends the record its row's footprint already tracks.
///
/// Complexity: O(materialized rows) expected index probes.
pub fn observe_persistent_names<S, Q, P, R, A, I, C>(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    plan: &WorthQueryAdmittedApplicationQueryPlan<'_, S, Q, P, R, A, I, C>,
    kernel: &mut RawNonLiveKernelOutcome,
) {
    let mut entities = BTreeSet::new();
    collect(kernel.raw.rows.raw_rows(), &mut entities);
    let observation = plan.basis.relational_observation();
    let reader = ExecutionNameReader::at_observation(runtime, execution, &observation);
    kernel.persistent_names = reader
        .observed_entity_names(entities, &mut NameReadDependencies::new())
        .ok();
}

fn collect(rows: &[WorthQueryApplicationProjectionNode], entities: &mut BTreeSet<EntityId>) {
    for row in rows {
        entities.insert(row.entity_id());
        for relation in row.relations() {
            collect(relation.rows(), entities);
        }
    }
}

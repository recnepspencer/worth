//! Observes the persistent names of the rows one kernel read materialized,
//! at the read's own basis and under the runtime the read already holds.

use std::collections::BTreeSet;

use worth_query_execution_naming::facade::{ExecutionNameReader, NameReadDependencies};
use worth_relational::facade::identity::EntityId;

use crate::domain_computation::primary_graph::application_query::observed_source::member_groups::WorthQueryObservedMemberGroups;

use super::super::projection::WorthQueryApplicationProjectionNode;
use super::super::WorthQueryAdmittedApplicationQueryPlan;
use super::RawNonLiveKernelOutcome;

/// Reads the name every materialized row's record carries, in one pass over
/// the name index at the plan's selected basis, and lends them to the
/// projection that follows.
///
/// When the name index refuses at the basis the names stay unobserved and
/// each name slot refuses `PersistentNamesUnavailable`; the read itself and
/// rows that project no name are unaffected. A projected record name adds
/// no source dependency: a record's name changes only when its lineage ends,
/// which ends the record its row's footprint already tracks. A projected
/// group name does (`groups`): regrouping a member changes no record, so each
/// row's footprint keeps every named member's complete group membership,
/// which every consumer of the observed source re-verifies.
///
/// Complexity: O(materialized rows) expected index probes.
pub fn observe_persistent_names<S, Q, P, R, A, I, C>(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    plan: &WorthQueryAdmittedApplicationQueryPlan<'_, S, Q, P, R, A, I, C>,
    kernel: &mut RawNonLiveKernelOutcome,
    groups: bool,
) {
    let mut entities = BTreeSet::new();
    collect(kernel.raw.rows.raw_rows(), &mut entities);
    let observation = plan.basis.relational_observation();
    let reader = ExecutionNameReader::at_observation(runtime, execution, &observation);
    kernel.persistent_names = reader
        .observed_entity_names(entities, &mut NameReadDependencies::new())
        .ok();
    let (true, Some(names)) = (groups, kernel.persistent_names.as_ref()) else {
        return;
    };
    for footprint in &mut kernel.raw.source_footprints {
        let read = std::iter::once(footprint.root).chain(footprint.entities.iter().copied());
        let mut members: Vec<WorthQueryObservedMemberGroups> = read
            .filter_map(|entity| names.member_groups_of(entity))
            .map(|(member, kind, groups)| WorthQueryObservedMemberGroups {
                member,
                kind: kind.to_owned(),
                groups,
            })
            .collect();
        members.sort();
        members.dedup();
        footprint.member_groups = members;
    }
}

fn collect(rows: &[WorthQueryApplicationProjectionNode], entities: &mut BTreeSet<EntityId>) {
    for row in rows {
        entities.insert(row.entity_id());
        for relation in row.relations() {
            collect(relation.rows(), entities);
        }
    }
}

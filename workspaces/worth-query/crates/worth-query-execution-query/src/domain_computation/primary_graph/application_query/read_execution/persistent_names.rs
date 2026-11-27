//! Observes the persistent names of the rows one kernel read materialized,
//! at the read's own basis and under the runtime the read already holds.

use std::collections::BTreeSet;

use worth_query_execution_naming::facade::{ExecutionNameReader, NameReadDependencies};
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::lineage::PersistentNameSubjectKind;

use crate::domain_computation::primary_graph::application_attempt::fact::persistent_name_history::{
    PersistentNameHistoryAnswer, PersistentNameHistoryQuestion, PersistentNameHistoryRead,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact as Fact;

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
/// group name does: regrouping a member changes no record, so every named
/// member's group membership is kept as a `PersistentNameHistory` fact
/// (`persistent_name_facts`) for the consumer that keeps the result current.
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
    let members = kernel
        .persistent_names
        .iter()
        .flat_map(|names| names.member_groups());
    kernel.persistent_name_facts = members
        .map(|(member, kind, groups)| Fact::PersistentNameHistory {
            read: PersistentNameHistoryRead {
                kind: kind.to_owned(),
                subject: PersistentNameSubjectKind::Entity,
                question: PersistentNameHistoryQuestion::MemberGroups(member),
            },
            answer: PersistentNameHistoryAnswer::MemberGroups(groups),
        })
        .collect();
}

fn collect(rows: &[WorthQueryApplicationProjectionNode], entities: &mut BTreeSet<EntityId>) {
    for row in rows {
        entities.insert(row.entity_id());
        for relation in row.relations() {
            collect(relation.rows(), entities);
        }
    }
}

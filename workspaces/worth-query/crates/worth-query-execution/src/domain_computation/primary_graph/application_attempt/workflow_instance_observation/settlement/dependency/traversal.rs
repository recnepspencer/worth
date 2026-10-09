//! Actual pinned adjacency traversal, independent of a fact-count ceiling.
use crate::domain_computation::primary_graph::application_attempt::check_request_live;
use crate::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedRelation;
use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial;
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAdjacencyDirection, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::runtime::{RelationalAdjacencyDirection, RelationalAdjacencyVisit};
use worth_relational::facade::storage::RecordLifecycleState;

pub(super) fn observe(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    evidence: EntityId,
    policy: &WorthQueryRequestScope,
) -> Result<(Vec<EntityId>, WorthQueryApplicationObservedFact), WorthQueryApplicationAttemptDenial>
{
    let subject = "workflow evidence dependency relation";
    check_request_live(policy, subject)?;
    let view = runtime
        .read_truth()
        .project_snapshot(snapshot)
        .ok_or_else(|| super::denial(subject))?;
    let mut relations = Vec::new();
    let mut examined = 0usize;
    view.try_visit_adjacency_ids(
        evidence,
        layout.evidence_dependency_relation,
        RelationalAdjacencyDirection::Outgoing,
        |visit| {
            check_request_live(policy, subject)?;
            if let RelationalAdjacencyVisit::Relation(id) = visit {
                examined = examined
                    .checked_add(1)
                    .ok_or_else(|| StoreDenial::Representability.into_attempt_denial(subject))?;
                let Some(relation) = view
                    .exact_relation_metadata(id)
                    .map_err(|_| super::denial(subject))?
                else {
                    return Ok(());
                };
                if relation.kind_id == layout.evidence_dependency_relation
                    && relation.source == evidence
                    && relation.lifecycle == RecordLifecycleState::Live
                {
                    relations.push(WorthQueryApplicationObservedRelation {
                        relation_id: relation.relation_id,
                        from: relation.source,
                        to: relation.target,
                    });
                }
            }
            Ok::<(), WorthQueryApplicationAttemptDenial>(())
        },
    )?;
    // Match the existing frontier owner's endpoint/identity order exactly.
    relations.sort_by_key(|relation| {
        (
            relation.from.partition_value(),
            relation.from.local_slot_value(),
            relation.to.partition_value(),
            relation.to.local_slot_value(),
            relation.relation_id.partition_value(),
            relation.relation_id.local_slot_value(),
        )
    });
    // The scalar re-comparison owner charges examined records plus endpoints;
    // its extra anchor list is admitted by that owner's existing checked core.
    let comparison_work = examined
        .checked_add(relations.len())
        .and_then(|work| work.checked_add(1))
        .ok_or_else(|| StoreDenial::Representability.into_attempt_denial(subject))?;
    let dependencies = relations.iter().map(|relation| relation.to).collect();
    check_request_live(policy, subject)?;
    Ok((
        dependencies,
        WorthQueryApplicationObservedFact::Adjacency {
            relation_kind: layout.evidence_dependency_relation,
            anchor: evidence,
            direction: WorthQueryApplicationAdjacencyDirection::Outgoing,
            maximum_work_units: comparison_work,
            relations,
        },
    ))
}

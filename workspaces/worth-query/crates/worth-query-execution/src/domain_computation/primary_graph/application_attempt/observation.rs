use std::sync::Arc;

use worth_relational::facade::query::{
    DeterministicQueryPlanKey, PlannedQueryPacket, QueryAccessContract, QueryExecutionShape,
    QueryLocalityClass, QueryOrderingContract, QueryScope, ReductionDiscipline,
};
use worth_relational::facade::runtime::{ProjectionAspectRequirement, ProjectionAspectScope};
use worth_relational::facade::storage::RecordLifecycleState;

use super::{WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind};

mod predecode_admission;
pub(in crate::domain_computation::primary_graph) use predecode_admission::observe_field_value_borrowed;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryApplicationFieldObservation {
    Present(worth_foundational::facade::AspectValue),
    Absent,
}

pub(in crate::domain_computation::primary_graph) fn observe_field(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity_id: worth_relational::facade::identity::EntityId,
    kind: worth_relational::facade::identity::KindId,
    locator: &worth_foundational::facade::AspectFieldLocator,
) -> Option<WorthQueryApplicationFieldObservation> {
    let field = locator.field_path().fields().first()?.clone();
    let scope = ProjectionAspectScope::from_requirements([ProjectionAspectRequirement::fields(
        locator.aspect().aspect_key().clone(),
        [field.clone()],
    )]);
    runtime
        .read_truth()
        .project_snapshot(snapshot)?
        .entity_record_with_projection_scope(entity_id, scope, |record| {
            (record.kind_id() == kind && record.lifecycle() == RecordLifecycleState::Live).then(
                || {
                    record
                        .aspect_field_value(locator.aspect().aspect_key(), &field)
                        .cloned()
                        .map_or(
                            WorthQueryApplicationFieldObservation::Absent,
                            WorthQueryApplicationFieldObservation::Present,
                        )
                },
            )
        })
}

pub(in crate::domain_computation::primary_graph) fn observe_field_value(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity_id: worth_relational::facade::identity::EntityId,
    kind: worth_relational::facade::identity::KindId,
    locator: &worth_foundational::facade::AspectFieldLocator,
) -> Option<worth_foundational::facade::AspectValue> {
    match observe_field(runtime, snapshot, entity_id, kind, locator)? {
        WorthQueryApplicationFieldObservation::Present(value) => Some(value),
        WorthQueryApplicationFieldObservation::Absent => None,
    }
}

pub(super) fn exact_relations(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    relation_kind: worth_relational::facade::identity::KindId,
    from: worth_relational::facade::identity::EntityId,
    to: worth_relational::facade::identity::EntityId,
) -> Result<Vec<worth_relational::facade::identity::RelationId>, WorthQueryApplicationAttemptDenial>
{
    let context = runtime
        .read_truth()
        .query_plan_context(snapshot)
        .ok_or_else(|| missing_fact("relation snapshot"))?;
    let packet = PlannedQueryPacket {
        label: "application-decision-relation".to_string(),
        context_id: context,
        scope: QueryScope::OutgoingNeighborhood {
            seeds: Arc::from([from]),
            relation_kind_scope: Some(Arc::from([relation_kind])),
        },
        locality: QueryLocalityClass::CrossPartitionTraversal,
        ordering: QueryOrderingContract::CanonicalTraversalOrder,
        access_contract: QueryAccessContract::AuthoritativeStorageOnly,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: relation_plan_key(from, to, relation_kind),
        target_count_hint: 1,
    };
    let plan = runtime
        .read_truth()
        .plan_query_packet(snapshot, packet)
        .ok_or_else(|| missing_fact("relation plan"))?;
    let outcome = runtime
        .read_truth()
        .execute_query_plan(plan)
        .ok_or_else(|| missing_fact("relation execution"))?;
    Ok(outcome
        .result
        .relations
        .iter()
        .filter(|record| {
            record.kind.kind_id == relation_kind
                && record.source == from
                && record.target == to
                && record.lifecycle == RecordLifecycleState::Live
        })
        .map(|record| record.relation_id)
        .collect())
}

fn relation_plan_key(
    from: worth_relational::facade::identity::EntityId,
    to: worth_relational::facade::identity::EntityId,
    relation: worth_relational::facade::identity::KindId,
) -> DeterministicQueryPlanKey {
    DeterministicQueryPlanKey(
        ((from.local_slot_value() as u128) << 64)
            | ((to.local_slot_value() as u128) << 32)
            | relation.as_u32() as u128,
    )
}

fn missing_fact(subject: &str) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
        subject,
    )
}

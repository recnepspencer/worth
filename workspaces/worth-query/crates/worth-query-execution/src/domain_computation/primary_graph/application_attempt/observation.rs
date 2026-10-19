use std::sync::Arc;

use worth_relational::facade::query::{
    DeterministicQueryPlanKey, PlannedQueryPacket, QueryAccessContract, QueryExecutionShape,
    QueryLocalityClass, QueryOrderingContract, QueryScope, ReductionDiscipline,
};
use worth_relational::facade::runtime::{ProjectionAspectRequirement, ProjectionAspectScope};
use worth_relational::facade::storage::RecordLifecycleState;

use super::{WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind};

mod predecode_admission;
#[cfg(test)]
mod read_rendezvous;
#[cfg(test)]
pub(in crate::domain_computation::primary_graph) use read_rendezvous::ReadRendezvous;
#[cfg(test)]
std::thread_local! {
    static NATIVE_FIELD_CONTACTS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
#[cfg(test)]
pub(in crate::domain_computation::primary_graph) fn take_native_field_contacts() -> usize {
    NATIVE_FIELD_CONTACTS.with(|count| count.replace(0))
}
pub(in crate::domain_computation::primary_graph) use predecode_admission::{
    observe_aspect_absence, observe_field_observation_borrowed_admitted,
    observe_field_value_borrowed, observe_field_value_borrowed_admitted,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryApplicationFieldObservation {
    Present(worth_foundational::facade::AspectValue),
    Absent,
    AspectAbsent,
    UnsupportedAspectShape,
}
impl From<worth_relational::facade::runtime::RelationalProjectedFieldObservation<'_>>
    for WorthQueryApplicationFieldObservation
{
    fn from(
        value: worth_relational::facade::runtime::RelationalProjectedFieldObservation<'_>,
    ) -> Self {
        use worth_relational::facade::runtime::RelationalProjectedFieldObservation as Raw;
        match value {
            Raw::Present(value) => Self::Present(value.clone()),
            Raw::AspectAbsent => Self::AspectAbsent,
            Raw::FieldAbsent => Self::Absent,
            Raw::UnsupportedAspectShape => Self::UnsupportedAspectShape,
        }
    }
}

pub(in crate::domain_computation::primary_graph) fn observe_field(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity_id: worth_relational::facade::identity::EntityId,
    kind: worth_relational::facade::identity::KindId,
    locator: &worth_foundational::facade::AspectFieldLocator,
) -> Result<
    Option<WorthQueryApplicationFieldObservation>,
    worth_relational::facade::transactions::RelationalExecutionDenialCause,
> {
    #[cfg(test)]
    NATIVE_FIELD_CONTACTS.with(|count| count.set(count.get() + 1));
    let Some(field) = locator.field_path().fields().first().cloned() else {
        return Ok(None);
    };
    let scope = ProjectionAspectScope::from_requirements([ProjectionAspectRequirement::fields(
        locator.aspect().aspect_key().clone(),
        [field.clone()],
    )]);
    let Some(view) = runtime.read_truth().project_snapshot(snapshot) else {
        return Ok(None);
    };
    view.entity_record_with_projection_scope(execution, entity_id, scope, |record| {
        (record.kind_id() == kind && record.lifecycle() == RecordLifecycleState::Live).then(|| {
            let value = record
                .observe_aspect_field(locator.aspect().aspect_key(), &field)
                .expect("the requested field belongs to its exact projection");
            #[cfg(test)]
            read_rendezvous::reached();
            WorthQueryApplicationFieldObservation::from(value)
        })
    })
}

pub(in crate::domain_computation::primary_graph) fn observe_field_value(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity_id: worth_relational::facade::identity::EntityId,
    kind: worth_relational::facade::identity::KindId,
    locator: &worth_foundational::facade::AspectFieldLocator,
) -> Result<
    Option<worth_foundational::facade::AspectValue>,
    worth_relational::facade::transactions::RelationalExecutionDenialCause,
> {
    match observe_field(runtime, execution, snapshot, entity_id, kind, locator)? {
        Some(WorthQueryApplicationFieldObservation::Present(value)) => Ok(Some(value)),
        Some(
            WorthQueryApplicationFieldObservation::Absent
            | WorthQueryApplicationFieldObservation::AspectAbsent
            | WorthQueryApplicationFieldObservation::UnsupportedAspectShape,
        )
        | None => Ok(None),
    }
}

pub(super) fn exact_relations(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
        .execute_query_plan(plan, execution)
        .map_err(|stop| {
            WorthQueryApplicationAttemptDenial::query_read_denial(stop, "relation execution")
        })?
        .ok_or_else(|| missing_fact("relation execution"))?
        .outcome;
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

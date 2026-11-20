use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
use std::collections::BTreeMap;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_relational::facade::identity::{EntityId, PartitionId};

/// Historical identity, not a grant or an execution right.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkflowAdmittedActor(EntityId);

impl WorkflowAdmittedActor {
    pub(in crate::domain_computation::primary_graph) fn from_admission<S, O, I, C>(
        admitted: &WorthQueryAdmittedApplicationOperation<S, O, I, C>,
    ) -> Self {
        let principal = admitted.operation_scope_binding().principal();
        Self(EntityId::new(
            PartitionId::new(principal.partition_id()),
            principal.local_slot(),
            principal.generation(),
        ))
    }
    pub(in crate::domain_computation::primary_graph) fn from_performed_receipt(
        receipt: &crate::domain_computation::primary_graph::application_attempt::workflow_transition_program::ValidatedWorkflowOperationReceipt<'_>,
    ) -> Self {
        Self(receipt.principal())
    }
    pub(in crate::domain_computation) const fn entity(self) -> EntityId {
        self.0
    }
    /// Recorded coordinates come from the actual admitted/receipted principal.
    /// They are DATA, never a grant for a later request or another runtime.
    pub(in crate::domain_computation::primary_graph) fn ending_actor(
        self,
    ) -> worth_proof::AdmittedWorkflowEndingActor<
        Self,
        worth_relational::facade::runtime::RelationalHomeProgramPlanningSource,
    > {
        worth_proof::AdmittedWorkflowEndingActor::from_admitted_actor(self)
    }
    pub(super) fn ending_provenance(
        self,
    ) -> worth_foundational::facade::WorkflowEndingActorProvenance {
        let actor = self.entity();
        use worth_foundational::facade::{EntityId, PartitionId, WorkflowEndingActorProvenance};
        WorkflowEndingActorProvenance::Principal(EntityId::new(
            PartitionId(actor.partition_value()),
            actor.local_slot_value(),
            actor.generation_value(),
        ))
    }
    pub(in crate::domain_computation::primary_graph) fn append_fields(
        self,
        locators: &WorkflowActorFields,
        fields: &mut BTreeMap<AspectFieldLocator, AspectValue>,
    ) {
        fields.insert(
            locators.partition.clone(),
            AspectValue::UInt64(self.0.partition_value_u64()),
        );
        fields.insert(
            locators.slot.clone(),
            AspectValue::UInt64(self.0.local_slot_value()),
        );
        fields.insert(
            locators.generation.clone(),
            AspectValue::UInt64(u64::from(self.0.generation_value())),
        );
    }
}
pub use worth_query_execution_layout::facade::WorkflowActorFields;

pub(in crate::domain_computation::primary_graph) fn read_adoption_actor_fields(
    locators: &WorkflowActorFields,
    record: &super::super::adoption::WorkflowAdoptionEntity<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    entity: EntityId,
) -> Result<Option<EntityId>, super::super::adoption::WorkflowAdoptionReadDenial> {
    let invalid =
        || super::super::adoption::WorkflowAdoptionReadDenial::UnreadableEntity { entity };
    decode_actor_components([
        record.optional_u64(execution, &locators.partition)?,
        record.optional_u64(execution, &locators.slot)?,
        record.optional_u64(execution, &locators.generation)?,
    ])
    .map_err(|_| invalid())
}

pub(in crate::domain_computation) fn observe_actor_fields(
    locators: &WorkflowActorFields,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: worth_relational::facade::identity::KindId,
    facts: &mut Vec<crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact>,
) -> Result<
    Option<EntityId>,
    crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
> {
    use crate::domain_computation::primary_graph::application_attempt::observe_field_value;
    use crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact as Fact;
    let malformed = actor_record_corruption;
    let mut components = [None; 3];
    for (index, locator) in [&locators.partition, &locators.slot, &locators.generation]
        .into_iter()
        .enumerate()
    {
        match observe_field_value(runtime, execution, snapshot, entity, kind, locator)? {
            Some(value @ AspectValue::UInt64(_)) => {
                let AspectValue::UInt64(component) = &value else {
                    unreachable!()
                };
                components[index] = Some(*component);
                facts.push(Fact::Field {
                    entity_id: entity,
                    kind,
                    locator: locator.clone(),
                    value,
                });
            }
            None => {
                facts.push(Fact::AbsentField {
                    entity_id: entity,
                    kind,
                    locator: locator.clone(),
                });
            }
            _ => return Err(malformed()),
        }
    }
    decode_actor_components(components).map_err(|_| malformed())
}

/// One decoder for current native records and passive corruption inspection.
/// All absent is the typed not-yet state; a partial triplet is corrupt.
pub(in crate::domain_computation) fn decode_actor_components(
    components: [Option<u64>; 3],
) -> Result<Option<EntityId>, ()> {
    match components {
        [None, None, None] => Ok(None),
        [Some(partition), Some(slot), Some(generation)] => Ok(Some(EntityId::new(
            PartitionId::new(u32::try_from(partition).map_err(|_| ())?),
            slot,
            u32::try_from(generation).map_err(|_| ())?,
        ))),
        _ => Err(()),
    }
}

pub(in crate::domain_computation) fn validate_actor_applicability(
    actor: Option<EntityId>,
    required: bool,
) -> Result<
    Option<EntityId>,
    crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
> {
    if actor.is_some() == required {
        Ok(actor)
    } else {
        Err(actor_record_corruption())
    }
}

/// Current native actor corruption is distinct from foreign occurrence affinity.
pub(in crate::domain_computation) fn actor_record_corruption(
) -> crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial {
    use crate::domain_computation::primary_graph::{
        WorthQueryApplicationAttemptDenial as Denial,
        WorthQueryApplicationAttemptDenialKind as Kind,
    };
    Denial::new(
        Kind::InvalidAuthoritativeValue,
        "current workflow actor record is malformed",
    )
}

mod court_doors_for_test;
pub use court_doors_for_test::observe_actor_fields_for_test;

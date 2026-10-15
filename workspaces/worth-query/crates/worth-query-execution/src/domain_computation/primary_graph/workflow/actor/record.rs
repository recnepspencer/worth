use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
use std::collections::BTreeMap;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_relational::facade::identity::{EntityId, PartitionId};

/// Historical identity, not a grant or an execution right.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) struct WorkflowAdmittedActor(EntityId);

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
#[derive(Clone, Debug)]
pub(in crate::domain_computation) struct WorkflowActorFields {
    pub(in crate::domain_computation) partition: AspectFieldLocator,
    pub(in crate::domain_computation) slot: AspectFieldLocator,
    pub(in crate::domain_computation) generation: AspectFieldLocator,
}
impl WorkflowActorFields {
    pub(in crate::domain_computation::primary_graph) fn read_adoption(
        &self,
        record: &super::super::adoption::WorkflowAdoptionEntity<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: EntityId,
    ) -> Result<Option<EntityId>, super::super::adoption::WorkflowAdoptionReadDenial> {
        let invalid =
            || super::super::adoption::WorkflowAdoptionReadDenial::UnreadableEntity { entity };
        match [
            record.optional_u64(execution, &self.partition)?,
            record.optional_u64(execution, &self.slot)?,
            record.optional_u64(execution, &self.generation)?,
        ] {
            [None, None, None] => Ok(None),
            [Some(partition), Some(slot), Some(generation)] => Ok(Some(EntityId::new(
                PartitionId::new(u32::try_from(partition).map_err(|_| invalid())?),
                slot,
                u32::try_from(generation).map_err(|_| invalid())?,
            ))),
            _ => Err(invalid()),
        }
    }
    pub(in crate::domain_computation) fn observe(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        entity: EntityId,
        kind: worth_relational::facade::identity::KindId,
        facts: &mut Vec<
            crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact,
        >,
    ) -> Result<
        Option<EntityId>,
        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
    > {
        use crate::domain_computation::primary_graph::application_attempt::observe_field_value;
        use crate::domain_computation::primary_graph::{
            WorthQueryApplicationAttemptDenial as Denial,
            WorthQueryApplicationAttemptDenialKind as Kind,
            WorthQueryApplicationObservedFact as Fact,
        };
        let malformed = || {
            Denial::new(
                Kind::InvalidAuthoritativeValue,
                "current workflow actor record is malformed",
            )
        };
        let mut components = [None; 3];
        for (index, locator) in [&self.partition, &self.slot, &self.generation]
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
        match components.as_slice() {
            [None, None, None] => Ok(None),
            [Some(partition), Some(slot), Some(generation)] => Ok(Some(EntityId::new(
                PartitionId::new(u32::try_from(*partition).map_err(|_| malformed())?),
                *slot,
                u32::try_from(*generation).map_err(|_| malformed())?,
            ))),
            _ => Err(malformed()),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn planned(
        aspect: &str,
        role: &str,
    ) -> Result<
        Self,
        crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial,
    > {
        use crate::domain_computation::primary_graph::schema_layout::planned_field_locator;
        Ok(Self {
            partition: planned_field_locator(aspect, &format!("{role}-partition"))?,
            slot: planned_field_locator(aspect, &format!("{role}-slot"))?,
            generation: planned_field_locator(aspect, &format!("{role}-generation"))?,
        })
    }
}

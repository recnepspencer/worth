//! Closed certification inspection of original immutable committed actor fields.
//! These values never construct native selection evidence or capability access.
use super::*;
use worth_query_declaration::facade::application_capability::ApplicationWorkflowRecordedRole;
use worth_relational::facade::identity::PartitionId;

fn actor_record_corruption(
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

fn decode_actor_components(components: [Option<u64>; 3]) -> Result<Option<EntityId>, ()> {
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

fn validate_actor_applicability(
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

impl WorthQueryApplicationCommittedChanges {
    /// Observes the actual committed actor triplet on a known original record.
    #[doc(hidden)]
    pub fn workflow_actor_fields_for_test(
        &self,
        entity: EntityId,
        role: ApplicationWorkflowRecordedRole,
    ) -> Result<
        Option<EntityId>,
        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
    > {
        use ApplicationWorkflowRecordedRole::*;
        let (aspect, role) = match role {
            Proposer => ("workflow-proposal", "proposer"),
            Advancer => ("workflow-transition", "advancer"),
            Executor => ("workflow-transition", "executor"),
            Approver => ("workflow-approval", "approver"),
        };
        let fields = worth_query_execution_layout::facade::WorkflowActorFields::planned(aspect, role)
            .map_err(|_| actor_record_corruption())?;
        let values = [&fields.partition, &fields.slot, &fields.generation]
            .map(|locator| self.committed_field_value(entity, locator));
        let mut components = [None; 3];
        for (index, value) in values.into_iter().enumerate() {
            components[index] = match value {
                None => None,
                Some(AspectValue::UInt64(value)) => Some(value),
                _ => return Err(actor_record_corruption()),
            };
        }
        decode_actor_components(components).map_err(|_| actor_record_corruption())
    }

    /// Rehydrates the actual committed native actor evidence under the caller's lease.
    /// No bytes, decoded proof or permission leaves this closed certification door.
    #[doc(hidden)]
    pub fn retain_workflow_actor_evidence_for_test<R: crate::ports::WorkflowActorEvidenceRuntimePort>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &R,
        receipt: &super::super::WorthQueryApplicationCommitReceipt,
        approval: EntityId,
    ) -> Result<(), crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial>
    {
        runtime.retain_workflow_actor_evidence_for_test(self, execution, receipt, approval)
    }

    /// Passive corruption twin over an original committed triplet. This never
    /// writes a snapshot or returns evidence usable for native admission.
    #[doc(hidden)]
    pub fn workflow_actor_omission_for_test(
        &self,
        entity: EntityId,
        role: ApplicationWorkflowRecordedRole,
        omit_all: bool,
    ) -> Result<
        Option<EntityId>,
        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
    > {
        let original = self
            .workflow_actor_fields_for_test(entity, role)
            .map_err(|_| actor_record_corruption())?;
        let mut components = original.map_or([None; 3], |actor| {
            [
                Some(actor.partition_value_u64()),
                Some(actor.local_slot_value()),
                Some(u64::from(actor.generation_value())),
            ]
        });
        if omit_all {
            components = [None; 3];
        } else {
            components[0] = None;
        }
        let actor = decode_actor_components(components).map_err(|_| actor_record_corruption())?;
        let required = match role {
            ApplicationWorkflowRecordedRole::Executor => {
                let locator =
                    crate::domain_computation::primary_graph::schema_layout::planned_field_locator(
                        "workflow-transition",
                        "operation-receipt-identity",
                    )
                    .map_err(|_| actor_record_corruption())?;
                self.committed_field_value(entity, &locator).is_some()
            }
            _ => true,
        };
        validate_actor_applicability(actor, required)
    }
}

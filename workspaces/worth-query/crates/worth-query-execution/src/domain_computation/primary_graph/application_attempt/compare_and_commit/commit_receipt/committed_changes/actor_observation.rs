//! Closed certification inspection of original immutable committed actor fields.
//! These values never construct native selection evidence or capability access.
use super::*;
use worth_query_declaration::facade::application_capability::ApplicationWorkflowRecordedRole;

impl WorthQueryApplicationCommittedChanges {
    /// Observes the actual committed actor triplet on a known original record.
    #[doc(hidden)]
    pub fn workflow_actor_fields_for_test(
        &self,
        entity: EntityId,
        role: ApplicationWorkflowRecordedRole,
    ) -> Result<Option<EntityId>, ()> {
        use ApplicationWorkflowRecordedRole::*;
        let (aspect, role) = match role {
            Proposer => ("workflow-proposal", "proposer"),
            Advancer => ("workflow-transition", "advancer"),
            Executor => ("workflow-transition", "executor"),
            Approver => ("workflow-approval", "approver"),
        };
        let fields = crate::domain_computation::primary_graph::workflow::actor::record::WorkflowActorFields::planned(aspect, role).map_err(|_| ())?;
        let values = [&fields.partition, &fields.slot, &fields.generation]
            .map(|locator| self.committed_field_value(entity, locator));
        let mut components = [None; 3];
        for (index, value) in values.into_iter().enumerate() {
            components[index] = match value {
                None => None,
                Some(AspectValue::UInt64(value)) => Some(value),
                _ => return Err(()),
            };
        }
        crate::domain_computation::primary_graph::workflow::actor::record::decode_actor_components(
            components,
        )
    }
}

impl WorthQueryApplicationCommittedChanges {
    /// Rehydrates the actual committed native actor evidence under the caller's lease.
    /// No bytes, decoded proof or permission leaves this closed certification door.
    #[doc(hidden)]
    pub fn retain_workflow_actor_evidence_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        approval: EntityId,
    ) -> Result<(), crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial>
    {
        use crate::domain_computation::authorization::{
            WorkflowApprovalRequestDecodingDenial as Decode, WorthQueryRetainedCapabilityRequest,
        };
        use crate::domain_computation::primary_graph::{
            WorthQueryApplicationAttemptDenial as Denial,
            WorthQueryApplicationAttemptDenialKind as Kind,
        };
        let invalid = || {
            Denial::new(
                Kind::InvalidAuthoritativeValue,
                "committed approval request is invalid",
            )
        };
        let locator =
            crate::domain_computation::primary_graph::schema_layout::planned_field_locator(
                "workflow-approval",
                "authorization-request",
            )
            .map_err(|_| invalid())?;
        let Some(AspectValue::String(worth_foundational::facade::InternedString::Raw(encoded))) =
            self.committed_field_value(approval, &locator)
        else {
            return Err(invalid());
        };
        let _retained = WorthQueryRetainedCapabilityRequest::decode_workflow_approval_request(
            execution, &encoded,
        )
        .map_err(|cause| match cause {
            Decode::InvalidRequest => invalid(),
            Decode::Actor(cause) => *cause,
        })?;
        Ok(())
    }
}

impl WorthQueryApplicationCommittedChanges {
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
        let invalid = crate::domain_computation::primary_graph::workflow::actor::record::actor_record_corruption;
        let original = self
            .workflow_actor_fields_for_test(entity, role)
            .map_err(|_| invalid())?;
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
        let actor = crate::domain_computation::primary_graph::workflow::actor::record::decode_actor_components(components)
            .map_err(|_| invalid())?;
        let required = match role {
            ApplicationWorkflowRecordedRole::Executor => {
                let locator =
                    crate::domain_computation::primary_graph::schema_layout::planned_field_locator(
                        "workflow-transition",
                        "operation-receipt-identity",
                    )
                    .map_err(|_| invalid())?;
                self.committed_field_value(entity, &locator).is_some()
            }
            _ => true,
        };
        crate::domain_computation::primary_graph::workflow::actor::record::validate_actor_applicability(actor, required)
    }
}

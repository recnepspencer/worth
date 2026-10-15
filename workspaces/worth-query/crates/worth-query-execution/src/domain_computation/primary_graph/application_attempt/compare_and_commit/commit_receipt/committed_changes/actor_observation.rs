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
        match values {
            [None, None, None] => Ok(None),
            [Some(AspectValue::UInt64(partition)), Some(AspectValue::UInt64(slot)), Some(AspectValue::UInt64(generation))] => {
                Ok(Some(EntityId::new(
                    worth_relational::facade::identity::PartitionId::new(
                        u32::try_from(partition).map_err(|_| ())?,
                    ),
                    slot,
                    u32::try_from(generation).map_err(|_| ())?,
                )))
            }
            _ => Err(()),
        }
    }
}

impl WorthQueryApplicationCommittedChanges {
    /// Rehydrates the actual committed native actor evidence under the caller's lease.
    /// No bytes, decoded proof or permission leaves this closed certification door.
    #[doc(hidden)]
    pub fn retain_workflow_actor_evidence_for_test(&self,
        execution: &worth_execution::ExecutionRequest<'_, '_>, approval: EntityId)
        -> Result<(), crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial> {
        use crate::domain_computation::primary_graph::{WorthQueryApplicationAttemptDenial as Denial,
            WorthQueryApplicationAttemptDenialKind as Kind};
        use crate::domain_computation::authorization::{WorthQueryRetainedCapabilityRequest,
            WorkflowApprovalRequestDecodingDenial as Decode};
        let invalid = || Denial::new(Kind::InvalidAuthoritativeValue, "committed approval request is invalid");
        let locator = crate::domain_computation::primary_graph::schema_layout::planned_field_locator(
            "workflow-approval", "authorization-request").map_err(|_| invalid())?;
        let Some(AspectValue::String(worth_foundational::facade::InternedString::Raw(encoded))) =
            self.committed_field_value(approval, &locator) else { return Err(invalid()); };
        let _retained = WorthQueryRetainedCapabilityRequest::decode_workflow_approval_request(execution, &encoded)
            .map_err(|cause| match cause { Decode::InvalidRequest => invalid(), Decode::Actor(cause) => *cause })?;
        Ok(())
    }
}

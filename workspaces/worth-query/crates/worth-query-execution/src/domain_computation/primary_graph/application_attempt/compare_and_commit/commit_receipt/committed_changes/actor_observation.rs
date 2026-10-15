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
    ) -> Result<
        Option<EntityId>,
        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
    > {
        let invalid = crate::domain_computation::primary_graph::workflow::actor::record::actor_record_corruption;
        use ApplicationWorkflowRecordedRole::*;
        let (aspect, role) = match role {
            Proposer => ("workflow-proposal", "proposer"),
            Advancer => ("workflow-transition", "advancer"),
            Executor => ("workflow-transition", "executor"),
            Approver => ("workflow-approval", "approver"),
        };
        let fields = crate::domain_computation::primary_graph::workflow::actor::record::WorkflowActorFields::planned(aspect, role).map_err(|_| invalid())?;
        let values = [&fields.partition, &fields.slot, &fields.generation]
            .map(|locator| self.committed_field_value(entity, locator));
        let mut components = [None; 3];
        for (index, value) in values.into_iter().enumerate() {
            components[index] = match value {
                None => None,
                Some(AspectValue::UInt64(value)) => Some(value),
                _ => return Err(invalid()),
            };
        }
        crate::domain_computation::primary_graph::workflow::actor::record::decode_actor_components(
            components,
        )
        .map_err(|_| invalid())
    }
}

impl WorthQueryApplicationCommittedChanges {
    /// Rehydrates the actual committed native actor evidence under the caller's lease.
    /// No bytes, decoded proof or permission leaves this closed certification door.
    #[doc(hidden)]
    pub fn retain_workflow_actor_evidence_for_test<
        Schema: worth_query_installation::facade::ApplicationSchema,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        receipt: &super::super::WorthQueryApplicationCommitReceipt,
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
        // The receipt must own this exact artifact; its native runtime identity
        // and canonical commit select the retained snapshot, never current head.
        if !Arc::ptr_eq(&self.envelope, &receipt.committed_changes().envelope) {
            return Err(invalid());
        }
        let Some(AspectValue::String(worth_foundational::facade::InternedString::Symbol(symbol))) =
            self.committed_field_value(approval, &locator)
        else {
            return Err(invalid());
        };
        runtime
            .primary_provider
            .graph
            .with_runtime(execution, |native| {
                use worth_relational::facade::runtime::{
                    RelationalPinnedReadAdmissionDenial as PinStop,
                    RelationalRetainedCommitSnapshotDenialKind as SnapshotStop,
                };
                let snapshot = native
                    .snapshots()
                    .retained_snapshot_for_commit(
                        receipt.provider_runtime_instance_id(),
                        self.commit_reference(),
                    )
                    .map_err(|cause| match cause.kind() {
                        SnapshotStop::Execution(cause) => {
                            Denial::relational_read_stopped(cause, "committed approval snapshot")
                        }
                        _ => invalid(),
                    })?;
                let pin = native
                    .read_truth()
                    .pin_read_capability(snapshot.snapshot_handle(), &[], execution)
                    .map_err(|cause| match cause {
                        PinStop::Request(cause) => {
                            Denial::request_read_stopped(cause, "committed approval symbols")
                        }
                        _ => invalid(),
                    })?;
                let encoded = pin
                    .symbols()
                    .retained_text(worth_relational::facade::symbols::Symbol(symbol.0))
                    .filter(|text| text.is_managed())
                    .ok_or_else(invalid)?;
                // Borrow the native admitted backing for the entire decode. Only
                // the decoder's real retained wire/DTO asks the original payer.
                let _retained =
                    WorthQueryRetainedCapabilityRequest::decode_workflow_approval_request(
                        execution,
                        encoded.as_str(),
                    )
                    .map_err(|cause| match cause {
                        Decode::InvalidRequest => invalid(),
                        Decode::Actor(cause) => *cause,
                    })?;
                Ok(())
            })
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

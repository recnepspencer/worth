use worth_query_declaration::facade::{
    application_capability::{
        ApplicationCapabilityDelegationRequest, ApplicationCapabilityRequest,
    },
    application_schema::{ApplicationOperationMarkerIdentity, TypedMutationPreconditions},
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationCapability,
    WorthQueryInstalledApplicationOperation,
};

use super::super::{
    operation_progression::{
        progress_capability_operation, WorthQueryCapabilityOperationProgression,
    },
    WorthQueryAdmittedApplicationCapabilityAccess,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationIdempotencyResolution, WorthQueryApplicationIdempotencyResolutionDenial,
    WorthQueryPrimaryGraphApplicationRuntime,
};

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// A denied fresh delegation may inspect one exact prior commit. A missing
    /// or drifted record never leaves this method as executable authority.
    pub fn replay_capability_delegation_after_fresh_denial<
        CommandCapability,
        TargetCapability,
        TargetOperation,
        TargetInput,
        Operation,
        Input,
    >(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        access: WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            CommandCapability,
            Operation,
            Input,
        >,
        target: &WorthQueryInstalledApplicationCapability<
            Schema,
            TargetCapability,
            TargetOperation,
            TargetInput,
        >,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        preconditions: TypedMutationPreconditions<
            Schema,
            Operation,
            <Input as ApplicationCapabilityRequest<Schema, CommandCapability>>::Scope,
        >,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        Option<WorthQueryApplicationCommitOutcome>,
        WorthQueryApplicationIdempotencyResolutionDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Input: ApplicationCapabilityRequest<Schema, CommandCapability>
            + ApplicationCapabilityDelegationRequest<Schema, TargetCapability>
            + Clone
            + Send
            + Sync
            + 'static,
    {
        let execution = match phase.execution_request_for(&self.product_runtime) {
            Ok(execution) => execution,
            Err(_) => return Err(crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial::from_authorization(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial::new(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::ForeignPhase, "phase-custody"))),
        };
        self.replay_capability_delegation_after_fresh_denial_with_request(
            &execution,
            access,
            target,
            operation,
            preconditions,
            idempotency,
        )
    }

    pub(in crate::domain_computation) fn replay_capability_delegation_after_fresh_denial_with_request<
        CommandCapability,
        TargetCapability,
        TargetOperation,
        TargetInput,
        Operation,
        Input,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        access: WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            CommandCapability,
            Operation,
            Input,
        >,
        target: &WorthQueryInstalledApplicationCapability<
            Schema,
            TargetCapability,
            TargetOperation,
            TargetInput,
        >,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        preconditions: TypedMutationPreconditions<
            Schema,
            Operation,
            <Input as ApplicationCapabilityRequest<Schema, CommandCapability>>::Scope,
        >,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        Option<WorthQueryApplicationCommitOutcome>,
        WorthQueryApplicationIdempotencyResolutionDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Input: ApplicationCapabilityRequest<Schema, CommandCapability>
            + ApplicationCapabilityDelegationRequest<Schema, TargetCapability>
            + Clone
            + Send
            + Sync
            + 'static,
    {
        let Some(installed) = self.authorization.capability_plan(target) else {
            return Ok(None);
        };
        if super::validate_activation_operation(installed, operation).is_err() {
            return Ok(None);
        }
        let Ok(proposed) = access.capability_input().delegation_request() else {
            return Ok(None);
        };
        let resolved = match access.with_exact_observation(execution, self, |observation| {
            observation.resolve_delegation_replay_target(execution, target, &proposed)
        }) {
            Some(Ok(resolved)) => resolved,
            Some(Err(denial)) if super::super::denial::is_request_stop(denial.kind()) => {
                return Err(
                    WorthQueryApplicationIdempotencyResolutionDenial::from_authorization(denial),
                )
            }
            Some(Err(_)) | None => return Ok(None),
        };
        let Ok(prepared) = super::binding::bind_activation(
            self, installed, &proposed, resolved, target, operation,
        ) else {
            return Ok(None);
        };
        let admission = match progress_capability_operation(
            execution,
            self,
            access,
            operation,
            preconditions,
            WorthQueryCapabilityOperationProgression::DelegationActivation,
        ) {
            Ok(admission) => admission,
            Err(denial) if super::super::denial::is_request_stop(denial.kind()) => {
                return Err(
                    WorthQueryApplicationIdempotencyResolutionDenial::from_authorization(denial),
                )
            }
            Err(_) => return Ok(None),
        };
        let Ok(admission) = prepared.finish(admission) else {
            return Ok(None);
        };
        let resolution = self
            .resolve_admitted_application_idempotency_with_request(
                execution,
                &admission,
                idempotency,
            )?
            .into_resolution();
        Ok(match resolution {
            WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(receipt) => Some(
                WorthQueryApplicationCommitOutcome::AlreadyCommitted(receipt),
            ),
            WorthQueryApplicationIdempotencyResolution::IntentDrift
            | WorthQueryApplicationIdempotencyResolution::Unseen => None,
        })
    }
}

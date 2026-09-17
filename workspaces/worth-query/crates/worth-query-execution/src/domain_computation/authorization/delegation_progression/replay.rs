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
        let Some(Ok(resolved)) = access.with_exact_observation(self, |observation| {
            observation.resolve_delegation_replay_target(target, &proposed)
        }) else {
            return Ok(None);
        };
        let Ok(prepared) = super::binding::bind_activation(
            self, installed, &proposed, resolved, target, operation,
        ) else {
            return Ok(None);
        };
        let Ok(admission) = progress_capability_operation(
            self,
            access,
            operation,
            preconditions,
            WorthQueryCapabilityOperationProgression::DelegationActivation,
        ) else {
            return Ok(None);
        };
        let Ok(admission) = prepared.finish(admission) else {
            return Ok(None);
        };
        let resolution = self
            .resolve_admitted_application_idempotency(&admission, idempotency)?
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

use crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::primary_graph::application_attempt::IdempotencyResolutionScopedOps;
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
};

pub trait ReplayOps<Schema: ApplicationSchema>:
    crate::domain_computation::authorization::host::AuthorizationHost<Schema>
{
    fn replay_capability_delegation_after_fresh_denial<
        CommandCapability,
        TargetCapability,
        TargetOperation,
        TargetInput,
        Operation,
        Input,
    >(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
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
        let execution = match phase.execution_request_for(self.product_runtime()) {
            Ok(execution) => execution,
            Err(_) => return Err(crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial::from_authorization(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial::new(crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ForeignPhase), "phase-custody"))),
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
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::authorization::host::AuthorizationHost<Schema> + ?Sized,
    > ReplayOps<Schema> for HostRuntime
{
}

pub(in crate::domain_computation) trait ReplayScopedOps<Schema: ApplicationSchema>:
    crate::domain_computation::authorization::host::AuthorizationHost<Schema>
{
    fn replay_capability_delegation_after_fresh_denial_with_request<
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
        let Some(installed) = self.authorization().capability_plan(target) else {
            return Ok(None);
        };
        if super::validate_activation_operation(installed, operation).is_err() {
            return Ok(None);
        }
        let Ok(proposed) = access.capability_input().delegation_request() else {
            return Ok(None);
        };
        let resolved = match access.with_exact_observation(execution, self.authorization_host(), |observation| {
            observation.resolve_delegation_replay_target(execution, target, &proposed)
        }) {
            Some(Ok(resolved)) => resolved,
            Some(Err(denial)) => match denial.kind() {
                crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind::Stopped(_) => {
                return Err(
                    WorthQueryApplicationIdempotencyResolutionDenial::from_authorization(denial),
                )
                }
                crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind::Refused(_) => return Ok(None),
            },
            None => return Ok(None),
        };
        let Ok(prepared) = super::binding::bind_activation(
            self.authorization_host(),
            installed,
            &proposed,
            resolved,
            target,
            operation,
        ) else {
            return Ok(None);
        };
        let admission = match progress_capability_operation(
            execution,
            self.authorization_host(),
            access,
            operation,
            preconditions,
            WorthQueryCapabilityOperationProgression::DelegationActivation,
        ) {
            Ok(admission) => admission,
            Err(denial) => match denial.kind() {
                crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind::Stopped(_) => {
                return Err(
                    WorthQueryApplicationIdempotencyResolutionDenial::from_authorization(denial),
                )
                }
                crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind::Refused(_) => return Ok(None),
            },
        };
        let Ok(admission) = prepared.finish(admission) else {
            return Ok(None);
        };
        let resolution = self
            .application_runtime()
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

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::authorization::host::AuthorizationHost<Schema> + ?Sized,
    > ReplayScopedOps<Schema> for HostRuntime
{
}

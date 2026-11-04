//! Fresh lifecycle disclosure may observe a key without minting a stage owner.
use super::operation_role::installed_lifecycle_owner;
use crate::domain_computation::authorization::{
    operation_progression::{
        progress_capability_operation, WorthQueryCapabilityOperationProgression,
    },
    WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind,
    WorthQueryOperationAuthorizationRefusalKind as Refusal,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationIdempotencyResolution,
    WorthQueryApplicationIdempotencyResolutionDenial as Denial,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_schema::{ApplicationOperationMarkerIdentity, TypedMutationPreconditions},
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationOperation,
};

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Resolves original meaning after fresh capability authorization. No operation
    /// admission or successor-stage authority escapes this observation entrance.
    pub fn resolve_capability_lifecycle_idempotency<Capability, Operation, Input>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        preconditions: TypedMutationPreconditions<Schema, Operation, Input::Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<WorthQueryApplicationIdempotencyResolution, Denial>
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Input: ApplicationCapabilityRequest<Schema, Capability> + Clone + Send + Sync + 'static,
    {
        let authorize = |kind| {
            Denial::from_authorization(WorthQueryOperationAuthorizationDenial::new(
                WorthQueryOperationAuthorizationDenialKind::Refused(kind),
                operation.operation(),
            ))
        };
        let execution = phase
            .execution_request_for(&self.product_runtime)
            .map_err(|_| authorize(Refusal::ForeignPhase))?;
        let (_, _, role) = self
            .authorization
            .elevation_lifecycle_operation::<Schema, Operation>(
                operation.operation(),
                operation.input_type(),
            )
            .map_err(|()| authorize(Refusal::StaleInstalledOperation))?
            .ok_or_else(|| authorize(Refusal::ElevationLifecycleRoleMismatch))?;
        installed_lifecycle_owner(
            self,
            access.installed_capability_identity(),
            operation,
            role,
        )
        .map_err(Denial::from_authorization)?;
        // The admission stays local to this read. It has no predecessor binding
        // and cannot become a lifecycle program or a returned stage witness.
        let admission = progress_capability_operation(
            &execution,
            self,
            access,
            operation,
            preconditions,
            WorthQueryCapabilityOperationProgression::ElevationLifecycle,
        )
        .map_err(Denial::from_authorization)?;
        self.resolve_admitted_application_idempotency_with_request(
            &execution,
            &admission,
            idempotency,
        )
        .map(|read| read.into_resolution())
    }
}

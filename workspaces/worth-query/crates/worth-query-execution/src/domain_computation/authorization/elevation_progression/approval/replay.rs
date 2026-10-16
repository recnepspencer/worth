use super::*;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    /// Only returns an exact committed replay. A fresh approval cannot leave
    /// this path as an admitted operation when its original support is stale.
    pub fn replay_elevation_approval_after_fresh_denial<Capability, Operation, Input>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        requested: WorthQueryRequestedElevation,
        access: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        preconditions: TypedMutationPreconditions<
            Schema,
            Operation,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        WorthQueryElevationApprovalOutcome,
        (
            Option<WorthQueryApplicationIdempotencyResolutionDenial>,
            WorthQueryRequestedElevation,
        ),
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Input: ApplicationCapabilityRequest<Schema, Capability> + Clone + Send + Sync + 'static,
    {
        let execution = match phase.execution_request_for(&self.product_runtime) {
            Ok(execution) => execution,
            Err(_) => {
                return Err((
                    Some(
                        WorthQueryApplicationIdempotencyResolutionDenial::from_authorization(
                            WorthQueryOperationAuthorizationDenial::new(
                                WorthQueryOperationAuthorizationDenialKind::Refused(
                                    AuthorizationRefusal::ForeignPhase,
                                ),
                                "phase-custody",
                            ),
                        ),
                    ),
                    requested,
                ));
            }
        };
        self.replay_elevation_approval_after_fresh_denial_with_request(
            &execution,
            requested,
            access,
            operation,
            preconditions,
            idempotency,
        )
    }

    pub(in crate::domain_computation) fn replay_elevation_approval_after_fresh_denial_with_request<
        Capability,
        Operation,
        Input,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        requested: WorthQueryRequestedElevation,
        access: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        preconditions: TypedMutationPreconditions<
            Schema,
            Operation,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        WorthQueryElevationApprovalOutcome,
        (
            Option<WorthQueryApplicationIdempotencyResolutionDenial>,
            WorthQueryRequestedElevation,
        ),
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Input: ApplicationCapabilityRequest<Schema, Capability> + Clone + Send + Sync + 'static,
    {
        let draft = match bind_approval(execution, self, &requested, &access, operation) {
            Ok(draft) => draft,
            Err(denial) => match denial.kind() {
                crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Stopped(_) => {
                return Err((
                    Some(
                        WorthQueryApplicationIdempotencyResolutionDenial::from_authorization(
                            denial,
                        ),
                    ),
                    requested,
                ))
                }
                crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(_) => return Err((None, requested)),
            },
        };
        let admission = match progress_capability_operation(
            execution,
            self,
            access,
            operation,
            preconditions,
            WorthQueryCapabilityOperationProgression::ElevationLifecycle,
        ) {
            Ok(admission) => admission,
            Err(denial) => match denial.kind() {
                crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Stopped(_) => {
                return Err((
                    Some(
                        WorthQueryApplicationIdempotencyResolutionDenial::from_authorization(
                            denial,
                        ),
                    ),
                    requested,
                ))
                }
                crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(_) => return Err((None, requested)),
            },
        };
        let mut admission = admission
            .bind_elevation_approval(draft.bind(requested))
            .map_err(|(_, binding)| (None, binding.into_requested()))?;
        match self.resolve_admitted_elevation_approval_replay_with_request(
            execution,
            &mut admission,
            idempotency,
        ) {
            Ok(Some(outcome)) => Ok(outcome),
            Ok(None) => Err((
                None,
                admission
                    .take_elevation_approval_binding()
                    .expect("the replay probe retains its approval binding")
                    .into_requested(),
            )),
            Err((denial, requested)) => Err((Some(denial), requested)),
        }
    }
}

//! Explicit borrowed preconditions use the same native capability progression.
use super::*;
impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn authorize_borrowed_capability_operation<Capability, Operation, Input>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        access: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        preconditions: &TypedMutationPreconditions<
            Schema,
            Operation,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
    ) -> Result<
        WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
        WorthQueryOperationAuthorizationDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Input: ApplicationCapabilityRequest<Schema, Capability>,
    {
        let phase_request = match phase.execution_request_for(&self.product_runtime) {
            Ok(request) => request,
            Err(_) => return Err(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial::new(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ForeignPhase), "phase-custody")),
        };
        let execution = &phase_request;
        self.authorize_capability_operation_with_request(
            execution,
            access,
            operation,
            preconditions,
        )
    }
}

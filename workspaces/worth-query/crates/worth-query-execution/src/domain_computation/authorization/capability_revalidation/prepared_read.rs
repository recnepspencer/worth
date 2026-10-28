//! Capability metadata preselection uses the same actual future Product session.
use super::*;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation) fn refresh_capability_authorization_for_prepared_read(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        authorization: &mut WorthQueryRetainedCapabilityAuthorization,
        reserved: &crate::domain_computation::provider_session::WorthQueryPreparedProductReadSessionIdentity,
        product: &crate::basis::WorthQueryProductObservationLease,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        self.refresh_capability_authorization(
            execution,
            authorization,
            reserved.identity(),
            product,
        )
    }
}

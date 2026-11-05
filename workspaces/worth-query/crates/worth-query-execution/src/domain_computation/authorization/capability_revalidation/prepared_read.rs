//! Capability metadata preselection uses the same actual future Product session.
use super::*;

pub(in crate::domain_computation) trait PreparedReadScopedOps<Schema: ApplicationSchema>:
    crate::domain_computation::authorization::host::AuthorizationHost<Schema>
{
    fn refresh_capability_authorization_for_prepared_read(
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

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::authorization::host::AuthorizationHost<Schema> + ?Sized,
    > PreparedReadScopedOps<Schema> for HostRuntime
{
}

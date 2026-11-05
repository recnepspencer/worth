//! Sealed test doors over this module's private members.
use super::*;

impl<Schema, Capability, Operation, Input>
    WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>
where
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    #[doc(hidden)]
    pub fn graph_work_managed_run_identity_for_test(
        &self,
    ) -> crate::domain_computation::vocabulary::provider_session::session_identity::WorthQueryGraphWorkManagedRunIdentity{
        Self::graph_work_managed_run_identity(self)
    }
}

impl<Schema, Capability, Operation, Input>
    WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>
where
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    #[doc(hidden)]
    pub fn graph_work_session_identity_for_test(
        &self,
    ) -> crate::domain_computation::vocabulary::provider_session::session_identity::WorthQueryGraphWorkSessionIdentity{
        Self::graph_work_session_identity(self)
    }
}

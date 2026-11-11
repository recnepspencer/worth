//! The original admission owner and request decide current authority.
use super::WorthQueryAdmittedOperation;
use crate::domain_computation::authorization::{
    WorthQueryAdmissionLapse, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind,
    WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal,
};
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
impl WorthQueryAdmittedOperation {
    /// Ownership precedes lapse and every read: nothing about a foreign
    /// admission is judged before it is known to be this runtime's.
    pub(in crate::domain_computation) fn validate_runtime_ownership(
        &self,
        runtime_authority: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &ApplicationSchemaBindingIdentity,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        if self.data.runtime_authority != runtime_authority {
            return Err(WorthQueryOperationAuthorizationDenial::new(
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    AuthorizationRefusal::ForeignRuntime,
                ),
                &self.data.operation,
            ));
        }
        if &self.data.binding_identity != binding_identity {
            return Err(WorthQueryOperationAuthorizationDenial::new(
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    AuthorizationRefusal::StaleInstalledSchema,
                ),
                &self.data.operation,
            ));
        }
        Ok(())
    }

    /// Revalidates the time and cancellation authority carried from the exact
    /// authentication request that minted this operation admission.
    ///
    /// Later governed phases must call this rather than accepting a detached
    /// timestamp or caller assertion.
    pub(in crate::domain_computation) fn validate_current_authority(
        &self,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        match self.current_authority_lapse() {
            Some(lapse) => Err(WorthQueryOperationAuthorizationDenial::new(
                lapse.denial_kind(),
                &self.data.operation,
            )),
            None => Ok(()),
        }
    }

    /// The exact cause when the request authority that minted this admission
    /// has lapsed, or `None` while it is still current.
    pub(in crate::domain_computation) fn current_authority_lapse(
        &self,
    ) -> Option<WorthQueryAdmissionLapse> {
        crate::domain_computation::authorization::admitted_request::request_lapse(
            &self.data.request_scope,
            self.data.authentication_valid_until,
        )
    }

    pub(in crate::domain_computation) fn retain_admitted_request(
        &self,
    ) -> crate::domain_computation::authorization::admitted_request::AdmittedRequestCurrentness
    {
        crate::domain_computation::authorization::admitted_request::AdmittedRequestCurrentness {
            request: self.data.request_scope.clone(),
            authentication_valid_until: self.data.authentication_valid_until,
        }
    }

    pub(in crate::domain_computation) fn publication_request(
        &self,
    ) -> &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope {
        &self.data.request_scope
    }
}

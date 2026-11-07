use super::{ErasedAuthorizationBasis, WorthQueryAdmittedOperation};
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::authorization::{
    WorthQueryCapabilityRevocationBinding, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind,
};

impl WorthQueryAdmittedOperation {
    pub(super) fn bind_capability_revocation(
        mut self,
        binding: WorthQueryCapabilityRevocationBinding,
        work: worth_query_installation::facade::WorthQueryCanonicalWorkEvidence,
    ) -> Result<Self, WorthQueryOperationAuthorizationDenial> {
        if !matches!(
            self.data.authorization_basis,
            ErasedAuthorizationBasis::Capability
        ) {
            return Err(WorthQueryOperationAuthorizationDenial::new(
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    AuthorizationRefusal::InconsistentDecision,
                ),
                &self.data.operation,
            ));
        }
        self.data.canonical_work = self.data.canonical_work.with_execution_work(work);
        self.data.authorization_basis = ErasedAuthorizationBasis::CapabilityRevocation { binding };
        Ok(self)
    }

    pub(in crate::domain_computation) fn capability_revocation_binding(
        &self,
    ) -> Option<&WorthQueryCapabilityRevocationBinding> {
        match &self.data.authorization_basis {
            ErasedAuthorizationBasis::CapabilityRevocation { binding } => Some(binding),
            _ => None,
        }
    }
}

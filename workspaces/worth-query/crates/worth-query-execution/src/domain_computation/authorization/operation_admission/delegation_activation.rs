use super::{ErasedAuthorizationBasis, WorthQueryAdmittedOperation};
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::authorization::{
    WorthQueryDelegationActivationBinding, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind,
};

impl WorthQueryAdmittedOperation {
    pub(in crate::domain_computation) fn retain_delegation_proposal_canonical_work(
        &mut self,
        work: worth_query_installation::facade::WorthQueryCanonicalWorkEvidence,
    ) {
        self.data.canonical_work = self.data.canonical_work.with_execution_work(work);
    }

    pub(super) fn bind_delegation_activation(
        mut self,
        binding: WorthQueryDelegationActivationBinding,
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
        self.data.authorization_basis = ErasedAuthorizationBasis::DelegationActivation { binding };
        Ok(self)
    }

    pub(in crate::domain_computation) fn delegation_activation_binding(
        &self,
    ) -> Option<&WorthQueryDelegationActivationBinding> {
        match &self.data.authorization_basis {
            ErasedAuthorizationBasis::DelegationActivation { binding } => Some(binding),
            _ => None,
        }
    }
}

use super::{WorthQueryAdmittedApplicationOperation, WorthQueryOperationAuthorizationBasis};
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::authorization::{
    WorthQueryCapabilityRevocationBinding, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation) fn bind_capability_revocation(
        mut self,
        binding: WorthQueryCapabilityRevocationBinding,
        work: worth_query_installation::facade::WorthQueryCanonicalWorkEvidence,
    ) -> Result<Self, WorthQueryOperationAuthorizationDenial> {
        let basis = std::mem::replace(
            &mut self.authorization_basis,
            WorthQueryOperationAuthorizationBasis::Conventional,
        );
        let WorthQueryOperationAuthorizationBasis::Capability { input } = basis else {
            return Err(WorthQueryOperationAuthorizationDenial::new(
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    AuthorizationRefusal::InconsistentDecision,
                ),
                &self.operation,
            ));
        };
        self.canonical_work = self.canonical_work.with_execution_work(work);
        self.authorization_basis =
            WorthQueryOperationAuthorizationBasis::CapabilityRevocation { input, binding };
        Ok(self)
    }

    pub(in crate::domain_computation) fn capability_revocation_binding(
        &self,
    ) -> Option<&WorthQueryCapabilityRevocationBinding> {
        match &self.authorization_basis {
            WorthQueryOperationAuthorizationBasis::CapabilityRevocation { binding, .. } => {
                Some(binding)
            }
            _ => None,
        }
    }
}

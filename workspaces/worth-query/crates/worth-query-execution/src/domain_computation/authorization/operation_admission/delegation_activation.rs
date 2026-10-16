use super::authorization_basis::LifecycleBinding;
use super::{WorthQueryAdmittedApplicationOperation, WorthQueryOperationAuthorizationBasis};
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::authorization::{
    WorthQueryDelegationActivationBinding, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation) fn retain_delegation_proposal_canonical_work(
        &mut self,
        work: worth_query_installation::facade::WorthQueryCanonicalWorkEvidence,
    ) {
        self.canonical_work = self.canonical_work.with_execution_work(work);
    }

    pub(in crate::domain_computation) fn bind_delegation_activation(
        mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: WorthQueryDelegationActivationBinding,
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
        let binding = LifecycleBinding::retain(execution, binding, &self.operation)
            .map_err(|(denial, _binding)| denial)?;
        self.authorization_basis =
            WorthQueryOperationAuthorizationBasis::DelegationActivation { input, binding };
        Ok(self)
    }

    pub(in crate::domain_computation) fn delegation_activation_binding(
        &self,
    ) -> Option<&WorthQueryDelegationActivationBinding> {
        match &self.authorization_basis {
            WorthQueryOperationAuthorizationBasis::DelegationActivation { binding, .. } => {
                Some(binding.get())
            }
            _ => None,
        }
    }
}

use super::{ErasedAuthorizationBasis, WorthQueryAdmittedOperation};
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::authorization::{
    WorthQueryMandatoryReviewBinding, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind,
};

impl WorthQueryAdmittedOperation {
    pub(super) fn bind_mandatory_review(
        mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: WorthQueryMandatoryReviewBinding,
    ) -> Result<
        Self,
        (
            WorthQueryOperationAuthorizationDenial,
            WorthQueryMandatoryReviewBinding,
        ),
    > {
        if !matches!(
            self.data.authorization_basis,
            ErasedAuthorizationBasis::Capability
        ) {
            return Err((
                WorthQueryOperationAuthorizationDenial::new(
                    WorthQueryOperationAuthorizationDenialKind::Refused(
                        AuthorizationRefusal::InconsistentDecision,
                    ),
                    &self.data.operation,
                ),
                binding,
            ));
        };
        let memory = match execution
            .reserve_retained_memory(std::mem::size_of::<WorthQueryMandatoryReviewBinding>() as u64)
        {
            Ok(memory) => memory,
            Err(stop) => {
                return Err((
                    crate::domain_computation::authorization::denial::relational_read_denial(
                        stop.into(),
                        &self.data.operation,
                    ),
                    binding,
                ))
            }
        };
        self.data.authorization_basis = ErasedAuthorizationBasis::MandatoryReview {
            binding: Box::new(binding),
            _binding_memory: memory,
        };
        Ok(self)
    }

    pub(in crate::domain_computation) fn mandatory_review_binding(
        &self,
    ) -> Option<&WorthQueryMandatoryReviewBinding> {
        match &self.data.authorization_basis {
            ErasedAuthorizationBasis::MandatoryReview { binding, .. } => Some(binding),
            _ => None,
        }
    }

    pub(in crate::domain_computation) fn take_mandatory_review_binding(
        &mut self,
    ) -> Option<WorthQueryMandatoryReviewBinding> {
        match std::mem::replace(
            &mut self.data.authorization_basis,
            ErasedAuthorizationBasis::Capability,
        ) {
            ErasedAuthorizationBasis::MandatoryReview {
                binding,
                _binding_memory: memory,
            } => {
                let binding = *binding;
                drop(memory);
                Some(binding)
            }
            other => {
                self.data.authorization_basis = other;
                None
            }
        }
    }
}

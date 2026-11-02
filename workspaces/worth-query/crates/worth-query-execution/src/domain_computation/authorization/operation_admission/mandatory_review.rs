use super::{ErasedAuthorizationBasis, WorthQueryAdmittedOperation};
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::authorization::{
    WorthQueryMandatoryReviewBinding, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind,
};

impl WorthQueryAdmittedOperation {
    pub(super) fn bind_mandatory_review(
        mut self,
        binding: WorthQueryMandatoryReviewBinding,
    ) -> Result<
        Self,
        (
            WorthQueryOperationAuthorizationDenial,
            WorthQueryMandatoryReviewBinding,
        ),
    > {
        if !matches!(
            self.authorization_basis,
            ErasedAuthorizationBasis::Capability
        ) {
            return Err((
                WorthQueryOperationAuthorizationDenial::new(
                    WorthQueryOperationAuthorizationDenialKind::Refused(
                        AuthorizationRefusal::InconsistentDecision,
                    ),
                    &self.operation,
                ),
                binding,
            ));
        }
        self.authorization_basis = ErasedAuthorizationBasis::MandatoryReview { binding };
        Ok(self)
    }

    pub(in crate::domain_computation) fn mandatory_review_binding(
        &self,
    ) -> Option<&WorthQueryMandatoryReviewBinding> {
        match &self.authorization_basis {
            ErasedAuthorizationBasis::MandatoryReview { binding } => Some(binding),
            _ => None,
        }
    }

    pub(in crate::domain_computation) fn take_mandatory_review_binding(
        &mut self,
    ) -> Option<WorthQueryMandatoryReviewBinding> {
        match std::mem::replace(
            &mut self.authorization_basis,
            ErasedAuthorizationBasis::Capability,
        ) {
            ErasedAuthorizationBasis::MandatoryReview { binding } => Some(binding),
            other => {
                self.authorization_basis = other;
                None
            }
        }
    }
}

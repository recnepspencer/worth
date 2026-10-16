use super::authorization_basis::LifecycleBinding;
use super::{WorthQueryAdmittedApplicationOperation, WorthQueryOperationAuthorizationBasis};
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::authorization::{
    WorthQueryMandatoryReviewBinding, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation) fn bind_mandatory_review(
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
        let basis = std::mem::replace(
            &mut self.authorization_basis,
            WorthQueryOperationAuthorizationBasis::Conventional,
        );
        let WorthQueryOperationAuthorizationBasis::Capability { input } = basis else {
            return Err((
                WorthQueryOperationAuthorizationDenial::new(
                    WorthQueryOperationAuthorizationDenialKind::Refused(
                        AuthorizationRefusal::InconsistentDecision,
                    ),
                    &self.operation,
                ),
                binding,
            ));
        };
        let binding = LifecycleBinding::retain(execution, binding, &self.operation)?;
        self.authorization_basis =
            WorthQueryOperationAuthorizationBasis::MandatoryReview { input, binding };
        Ok(self)
    }

    pub(in crate::domain_computation) fn mandatory_review_binding(
        &self,
    ) -> Option<&WorthQueryMandatoryReviewBinding> {
        match &self.authorization_basis {
            WorthQueryOperationAuthorizationBasis::MandatoryReview { binding, .. } => {
                Some(binding.get())
            }
            _ => None,
        }
    }

    pub(in crate::domain_computation) fn take_mandatory_review_binding(
        &mut self,
    ) -> Option<WorthQueryMandatoryReviewBinding> {
        let basis = std::mem::replace(
            &mut self.authorization_basis,
            WorthQueryOperationAuthorizationBasis::Conventional,
        );
        match basis {
            WorthQueryOperationAuthorizationBasis::MandatoryReview { input, binding } => {
                self.authorization_basis =
                    WorthQueryOperationAuthorizationBasis::Capability { input };
                Some(binding.into_inner())
            }
            other => {
                self.authorization_basis = other;
                None
            }
        }
    }
}

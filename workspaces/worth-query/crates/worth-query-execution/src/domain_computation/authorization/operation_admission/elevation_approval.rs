use super::authorization_basis::LifecycleBinding;
use super::{WorthQueryAdmittedApplicationOperation, WorthQueryOperationAuthorizationBasis};
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::authorization::{
    WorthQueryElevationApprovalBinding, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation) fn bind_elevation_approval(
        mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: WorthQueryElevationApprovalBinding,
    ) -> Result<
        Self,
        (
            WorthQueryOperationAuthorizationDenial,
            WorthQueryElevationApprovalBinding,
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
            WorthQueryOperationAuthorizationBasis::ElevationApproval { input, binding };
        Ok(self)
    }

    pub(in crate::domain_computation) fn elevation_approval_binding(
        &self,
    ) -> Option<&WorthQueryElevationApprovalBinding> {
        match &self.authorization_basis {
            WorthQueryOperationAuthorizationBasis::ElevationApproval { binding, .. } => {
                Some(binding.get())
            }
            _ => None,
        }
    }

    pub(in crate::domain_computation) fn take_elevation_approval_binding(
        &mut self,
    ) -> Option<WorthQueryElevationApprovalBinding> {
        let basis = std::mem::replace(
            &mut self.authorization_basis,
            WorthQueryOperationAuthorizationBasis::Conventional,
        );
        match basis {
            WorthQueryOperationAuthorizationBasis::ElevationApproval { input, binding } => {
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

use super::{ErasedAuthorizationBasis, WorthQueryAdmittedOperation};
use crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use {crate::domain_computation::authorization::WorthQueryElevationApprovalBinding, crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial, crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind};

impl WorthQueryAdmittedOperation {
    pub(super) fn bind_elevation_approval(
        mut self,
        binding: WorthQueryElevationApprovalBinding,
    ) -> Result<
        Self,
        (
            WorthQueryOperationAuthorizationDenial,
            WorthQueryElevationApprovalBinding,
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
        self.authorization_basis = ErasedAuthorizationBasis::ElevationApproval { binding };
        Ok(self)
    }

    pub(in crate::domain_computation) fn elevation_approval_binding(
        &self,
    ) -> Option<&WorthQueryElevationApprovalBinding> {
        match &self.authorization_basis {
            ErasedAuthorizationBasis::ElevationApproval { binding } => Some(binding),
            _ => None,
        }
    }

    pub(in crate::domain_computation) fn take_elevation_approval_binding(
        &mut self,
    ) -> Option<WorthQueryElevationApprovalBinding> {
        match std::mem::replace(
            &mut self.authorization_basis,
            ErasedAuthorizationBasis::Capability,
        ) {
            ErasedAuthorizationBasis::ElevationApproval { binding } => Some(binding),
            other => {
                self.authorization_basis = other;
                None
            }
        }
    }
}

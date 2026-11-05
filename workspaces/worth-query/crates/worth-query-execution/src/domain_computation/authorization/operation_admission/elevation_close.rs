use super::{ErasedAuthorizationBasis, WorthQueryAdmittedOperation};
use crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use {crate::domain_computation::authorization::WorthQueryElevationCloseBinding, crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial, crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind};

impl WorthQueryAdmittedOperation {
    pub(super) fn bind_elevation_close(
        mut self,
        binding: WorthQueryElevationCloseBinding,
    ) -> Result<
        Self,
        (
            WorthQueryOperationAuthorizationDenial,
            WorthQueryElevationCloseBinding,
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
        self.authorization_basis = ErasedAuthorizationBasis::ElevationClose { binding };
        Ok(self)
    }

    pub(in crate::domain_computation) fn elevation_close_binding(
        &self,
    ) -> Option<&WorthQueryElevationCloseBinding> {
        match &self.authorization_basis {
            ErasedAuthorizationBasis::ElevationClose { binding } => Some(binding),
            _ => None,
        }
    }

    pub(in crate::domain_computation) fn take_elevation_close_binding(
        &mut self,
    ) -> Option<WorthQueryElevationCloseBinding> {
        match std::mem::replace(
            &mut self.authorization_basis,
            ErasedAuthorizationBasis::Capability,
        ) {
            ErasedAuthorizationBasis::ElevationClose { binding } => Some(binding),
            other => {
                self.authorization_basis = other;
                None
            }
        }
    }
}

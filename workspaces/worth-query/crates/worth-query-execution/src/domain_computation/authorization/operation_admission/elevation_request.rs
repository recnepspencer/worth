use super::{ErasedAuthorizationBasis, WorthQueryAdmittedOperation};
use crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use {crate::domain_computation::authorization::WorthQueryElevationRequestBinding, crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial, crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind};

impl WorthQueryAdmittedOperation {
    pub(super) fn bind_elevation_request(
        mut self,
        binding: WorthQueryElevationRequestBinding,
    ) -> Result<Self, WorthQueryOperationAuthorizationDenial> {
        if !matches!(
            self.authorization_basis,
            ErasedAuthorizationBasis::Capability
        ) {
            return Err(WorthQueryOperationAuthorizationDenial::new(
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    AuthorizationRefusal::InconsistentDecision,
                ),
                &self.operation,
            ));
        }
        self.authorization_basis = ErasedAuthorizationBasis::ElevationRequest { binding };
        Ok(self)
    }

    pub(in crate::domain_computation) fn elevation_request_binding(
        &self,
    ) -> Option<&WorthQueryElevationRequestBinding> {
        match &self.authorization_basis {
            ErasedAuthorizationBasis::ElevationRequest { binding } => Some(binding),
            _ => None,
        }
    }

    pub(in crate::domain_computation) fn take_elevation_request_binding(
        &mut self,
    ) -> Option<WorthQueryElevationRequestBinding> {
        match std::mem::replace(
            &mut self.authorization_basis,
            ErasedAuthorizationBasis::Capability,
        ) {
            ErasedAuthorizationBasis::ElevationRequest { binding } => Some(binding),
            other => {
                self.authorization_basis = other;
                None
            }
        }
    }
}

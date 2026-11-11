use super::{ErasedAuthorizationBasis, WorthQueryAdmittedOperation};
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::authorization::{
    WorthQueryElevationRequestBinding, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind,
};

impl WorthQueryAdmittedOperation {
    pub(super) fn bind_elevation_request(
        mut self,
        binding: WorthQueryElevationRequestBinding,
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
        self.data.authorization_basis = ErasedAuthorizationBasis::ElevationRequest { binding };
        Ok(self)
    }

    pub(in crate::domain_computation) fn elevation_request_binding(
        &self,
    ) -> Option<&WorthQueryElevationRequestBinding> {
        match &self.data.authorization_basis {
            ErasedAuthorizationBasis::ElevationRequest { binding } => Some(binding),
            _ => None,
        }
    }

    pub(in crate::domain_computation) fn take_elevation_request_binding(
        &mut self,
    ) -> Option<WorthQueryElevationRequestBinding> {
        match std::mem::replace(
            &mut self.data.authorization_basis,
            ErasedAuthorizationBasis::Capability,
        ) {
            ErasedAuthorizationBasis::ElevationRequest { binding } => Some(binding),
            other => {
                self.data.authorization_basis = other;
                None
            }
        }
    }
}

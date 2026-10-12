use super::{WorthQueryAdmittedApplicationOperation, WorthQueryOperationAuthorizationBasis};
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::authorization::{
    WorthQueryElevationRequestBinding, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation) fn bind_elevation_request(
        mut self,
        binding: WorthQueryElevationRequestBinding,
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
        self.authorization_basis =
            WorthQueryOperationAuthorizationBasis::ElevationRequest { input, binding };
        Ok(self)
    }

    pub(in crate::domain_computation) fn elevation_request_binding(
        &self,
    ) -> Option<&WorthQueryElevationRequestBinding> {
        match &self.authorization_basis {
            WorthQueryOperationAuthorizationBasis::ElevationRequest { binding, .. } => {
                Some(binding)
            }
            _ => None,
        }
    }

    pub(in crate::domain_computation) fn take_elevation_request_binding(
        &mut self,
    ) -> Option<WorthQueryElevationRequestBinding> {
        let basis = std::mem::replace(
            &mut self.authorization_basis,
            WorthQueryOperationAuthorizationBasis::Conventional,
        );
        match basis {
            WorthQueryOperationAuthorizationBasis::ElevationRequest { input, binding } => {
                self.authorization_basis =
                    WorthQueryOperationAuthorizationBasis::Capability { input };
                Some(binding)
            }
            other => {
                self.authorization_basis = other;
                None
            }
        }
    }
}

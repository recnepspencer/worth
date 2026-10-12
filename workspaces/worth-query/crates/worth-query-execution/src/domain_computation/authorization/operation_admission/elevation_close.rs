use super::{WorthQueryAdmittedApplicationOperation, WorthQueryOperationAuthorizationBasis};
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::authorization::{
    WorthQueryElevationCloseBinding, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation) fn bind_elevation_close(
        mut self,
        binding: WorthQueryElevationCloseBinding,
    ) -> Result<
        Self,
        (
            WorthQueryOperationAuthorizationDenial,
            WorthQueryElevationCloseBinding,
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
        self.authorization_basis =
            WorthQueryOperationAuthorizationBasis::ElevationClose { input, binding };
        Ok(self)
    }

    pub(in crate::domain_computation) fn elevation_close_binding(
        &self,
    ) -> Option<&WorthQueryElevationCloseBinding> {
        match &self.authorization_basis {
            WorthQueryOperationAuthorizationBasis::ElevationClose { binding, .. } => Some(binding),
            _ => None,
        }
    }

    pub(in crate::domain_computation) fn take_elevation_close_binding(
        &mut self,
    ) -> Option<WorthQueryElevationCloseBinding> {
        let basis = std::mem::replace(
            &mut self.authorization_basis,
            WorthQueryOperationAuthorizationBasis::Conventional,
        );
        match basis {
            WorthQueryOperationAuthorizationBasis::ElevationClose { input, binding } => {
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

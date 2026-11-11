use super::{ErasedAuthorizationBasis, WorthQueryAdmittedOperation};
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::authorization::{
    WorthQueryElevationCloseBinding, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind,
};

impl WorthQueryAdmittedOperation {
    pub(super) fn bind_elevation_close(
        mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: WorthQueryElevationCloseBinding,
    ) -> Result<
        Self,
        (
            WorthQueryOperationAuthorizationDenial,
            WorthQueryElevationCloseBinding,
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
            .reserve_retained_memory(std::mem::size_of::<WorthQueryElevationCloseBinding>() as u64)
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
        self.data.authorization_basis = ErasedAuthorizationBasis::ElevationClose {
            binding: Box::new(binding),
            _binding_memory: memory,
        };
        Ok(self)
    }

    pub(in crate::domain_computation) fn elevation_close_binding(
        &self,
    ) -> Option<&WorthQueryElevationCloseBinding> {
        match &self.data.authorization_basis {
            ErasedAuthorizationBasis::ElevationClose { binding, .. } => Some(binding),
            _ => None,
        }
    }

    pub(in crate::domain_computation) fn take_elevation_close_binding(
        &mut self,
    ) -> Option<WorthQueryElevationCloseBinding> {
        match std::mem::replace(
            &mut self.data.authorization_basis,
            ErasedAuthorizationBasis::Capability,
        ) {
            ErasedAuthorizationBasis::ElevationClose {
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

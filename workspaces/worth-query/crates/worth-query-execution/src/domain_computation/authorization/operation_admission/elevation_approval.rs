use super::{ErasedAuthorizationBasis, WorthQueryAdmittedOperation};
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::authorization::{
    WorthQueryElevationApprovalBinding, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind,
};

impl WorthQueryAdmittedOperation {
    pub(super) fn bind_elevation_approval(
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
        let memory = match execution.reserve_retained_memory(std::mem::size_of::<
            WorthQueryElevationApprovalBinding,
        >() as u64)
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
        self.data.authorization_basis = ErasedAuthorizationBasis::ElevationApproval {
            binding: Box::new(binding),
            _binding_memory: memory,
        };
        Ok(self)
    }

    pub(in crate::domain_computation) fn elevation_approval_binding(
        &self,
    ) -> Option<&WorthQueryElevationApprovalBinding> {
        match &self.data.authorization_basis {
            ErasedAuthorizationBasis::ElevationApproval { binding, .. } => Some(binding),
            _ => None,
        }
    }

    pub(in crate::domain_computation) fn take_elevation_approval_binding(
        &mut self,
    ) -> Option<WorthQueryElevationApprovalBinding> {
        match std::mem::replace(
            &mut self.data.authorization_basis,
            ErasedAuthorizationBasis::Capability,
        ) {
            ErasedAuthorizationBasis::ElevationApproval {
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

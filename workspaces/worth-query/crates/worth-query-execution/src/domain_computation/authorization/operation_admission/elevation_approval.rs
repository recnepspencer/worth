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
        let memory = match execution.reserve_retained_memory(std::mem::size_of::<
            WorthQueryElevationApprovalBinding,
        >() as u64)
        {
            Ok(memory) => memory,
            Err(stop) => {
                return Err((
                    crate::domain_computation::authorization::denial::relational_read_denial(
                        stop.into(),
                        &self.operation,
                    ),
                    binding,
                ))
            }
        };
        self.authorization_basis = WorthQueryOperationAuthorizationBasis::ElevationApproval {
            input,
            binding: Box::new(binding),
            _binding_memory: memory,
        };
        Ok(self)
    }

    pub(in crate::domain_computation) fn elevation_approval_binding(
        &self,
    ) -> Option<&WorthQueryElevationApprovalBinding> {
        match &self.authorization_basis {
            WorthQueryOperationAuthorizationBasis::ElevationApproval { binding, .. } => {
                Some(binding)
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
            WorthQueryOperationAuthorizationBasis::ElevationApproval {
                input,
                binding,
                _binding_memory: memory,
            } => {
                self.authorization_basis =
                    WorthQueryOperationAuthorizationBasis::Capability { input };
                let binding = {
                    let backing = binding;
                    *backing
                };
                drop(memory);
                Some(binding)
            }
            other => {
                self.authorization_basis = other;
                None
            }
        }
    }
}

//! Current recovery re-admission axes compared through `worth-proof` binding law.
//!
//! Historical receipt facts live in the recovery handle. Re-admission presents
//! only current operation truth, so callers cannot replace the receipt or its
//! installed aftermath with reconstructed objects.

use worth_proof::Binding;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::history::BranchId;

use crate::domain_computation::authorization::{
    WorthQueryAdmittedApplicationOperation, WorthQueryOperationScopeBinding,
};

use super::super::recovery_handle::{
    WorthQueryRecoveryHandleBinding, WorthQueryRecoveryHandleDenial,
    WorthQueryRecoveryHandleDenialKind,
};

worth_proof::binding_axes! {
    /// Current operation facts that must still match a live recovery handle.
    pub(crate) struct WorthQueryRecoveryBindingCurrentTruth {
        schema_identity: [u8; 32] => SchemaIdentity,
        branch: BranchId => Branch,
        application_binding_generation: u64 => ApplicationBindingGeneration,
        installed_operation: [u8; 32] => InstalledOperation,
        governed_input_identity: Option<[u8; 32]> => GovernedInputIdentity,
        principal_scope: WorthQueryOperationScopeBinding => PrincipalScope,
        installed_aftermath: [u8; 32] => InstalledAftermath,
    }
    drift pub(crate) enum WorthQueryRecoveryBindingDrift;
}

impl WorthQueryRecoveryBindingCurrentTruth {
    pub(crate) fn from_admitted<Schema, Operation, Input, Scope>(
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<Self, WorthQueryRecoveryHandleDenial>
    where
        Schema: ApplicationSchema,
    {
        let aftermath = admission
            .allowed_graph_contract()
            .aftermath()
            .ok_or_else(|| {
                WorthQueryRecoveryHandleDenial::new(
                    WorthQueryRecoveryHandleDenialKind::RecoveryNotAdmitted,
                )
            })?;
        Ok(Self {
            schema_identity: *admission
                .operation_scope_binding()
                .binding_identity()
                .schema_identity()
                .bytes(),
            branch: admission.graph_work_branch().clone(),
            application_binding_generation: admission.binding_identity().generation(),
            installed_operation: *admission.installed_operation_identity_bytes(),
            governed_input_identity: admission.governed_input_identity().copied(),
            principal_scope: admission.operation_scope_binding().clone(),
            installed_aftermath: *aftermath.identity().digest().bytes(),
        })
    }

    fn from_handle_binding(binding: &WorthQueryRecoveryHandleBinding) -> Self {
        Self {
            schema_identity: *binding.schema_identity(),
            branch: binding.branch().clone(),
            application_binding_generation: binding.application_binding_generation(),
            installed_operation: *binding.installed_operation(),
            governed_input_identity: binding
                .retained_governed_input()
                .and_then(|input| input.governed_identity()),
            principal_scope: binding.principal_scope().clone(),
            installed_aftermath: *binding.installed_aftermath_identity().digest().bytes(),
        }
    }

    pub(crate) fn check(
        &self,
        binding: &WorthQueryRecoveryHandleBinding,
    ) -> Result<(), WorthQueryRecoveryHandleDenial> {
        let presented = Self::from_handle_binding(binding);
        Binding::new(self.clone())
            .ensure_matches(&Binding::new(presented))
            .map_err(|drift| map_binding_drift(drift, self, binding))
    }

    #[cfg(test)]
    pub(crate) fn axis_probe(binding: &WorthQueryRecoveryHandleBinding) -> Self {
        Self::from_handle_binding(binding)
    }
}

fn map_binding_drift(
    drift: WorthQueryRecoveryBindingDrift,
    current: &WorthQueryRecoveryBindingCurrentTruth,
    binding: &WorthQueryRecoveryHandleBinding,
) -> WorthQueryRecoveryHandleDenial {
    use WorthQueryRecoveryBindingDrift as Drift;
    use WorthQueryRecoveryHandleDenialKind as Denial;

    let kind = match drift {
        Drift::SchemaIdentity => Denial::SchemaMismatch,
        Drift::Branch => {
            if binding.application_binding_generation() == current.application_binding_generation {
                Denial::ForeignBranchEqualOrdinal
            } else {
                Denial::BranchMismatch
            }
        }
        Drift::ApplicationBindingGeneration => Denial::ApplicationBindingGenerationMismatch,
        Drift::InstalledOperation => Denial::OperationMismatch,
        Drift::GovernedInputIdentity => Denial::GovernedInputMismatch,
        Drift::PrincipalScope => Denial::ForeignPrincipal,
        Drift::InstalledAftermath => Denial::CompatibilityGenerationMismatch,
    };
    WorthQueryRecoveryHandleDenial::new(kind)
}

pub(super) fn check_binding_axes<Schema, Operation, Input, Scope>(
    binding: &WorthQueryRecoveryHandleBinding,
    admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
) -> Result<(), WorthQueryRecoveryHandleDenial>
where
    Schema: ApplicationSchema,
{
    WorthQueryRecoveryBindingCurrentTruth::from_admitted(admission)?.check(binding)
}

#[cfg(test)]
#[path = "binding_axis_tests.rs"]
mod tests;

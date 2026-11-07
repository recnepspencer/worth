//! Carry the exact root or Home target inventory through one preparation.
use super::RelationalPreparationRuntime;
use crate::branch::AdmittedRelationalBranchBasis;
use crate::runtime::AdmittedRelationalHomeProgramTarget;
use crate::transactions::data::TransactionCommitError;
use std::sync::Arc;

impl RelationalPreparationRuntime {
    pub(crate) fn select_home_contracts(
        &self,
        basis: &AdmittedRelationalBranchBasis,
        target: Option<&Arc<AdmittedRelationalHomeProgramTarget>>,
    ) -> Result<Self, TransactionCommitError> {
        let selected = match target {
            Some(target) => Some(&target.meaning),
            None => basis.inner.root.execution_contracts.get(),
        };
        let mut runtime = self.clone();
        if let Some(target) = target {
            if !target.receiver_epoch.same_epoch(&self.configuration_epoch) {
                return Err(required_contracts());
            }
        }
        runtime.schema_contract_runtime = self.configuration_epoch.schema_contract_runtime.clone();
        runtime.execution_contracts = None;
        runtime.execution_registry = Some(match selected {
            Some(selected) => selected.schema.registry_arc(),
            None => basis.inner.root.schema_authority().registry_arc(),
        });
        runtime.home_target = target.cloned();
        match selected {
            Some(selected) => {
                if selected.runtime != self.runtime_instance_id() {
                    return Err(required_contracts());
                }
                if target.is_none()
                    && !selected.schema.has_same_executable_meaning_except_basis(
                        basis.inner.root.schema_authority().registry(),
                    )
                {
                    return Err(required_contracts());
                }
                runtime.schema_contract_runtime = selected.contracts.clone();
                runtime.execution_contracts = Some(selected.clone());
            }
            None => {
                // D16 governs a medium bound by the Home owner. Ordinary Native
                // roots keep interpreting their own admitted schema inventory.
                let home = self
                    .durability_admission
                    .requires_home_program_contracts()
                    .map_err(|cause| {
                        TransactionCommitError::publication_denied(
                            crate::mvcc::RelationalPublicationDenial::HomeProgramTransition(cause),
                        )
                    })?;
                if home
                    && !basis
                        .inner
                        .root
                        .schema_authority()
                        .has_same_executable_meaning_except_basis(&self.config.schema.registry)
                {
                    return Err(required_contracts());
                }
            }
        }
        Ok(runtime)
    }
}

fn required_contracts() -> TransactionCommitError {
    TransactionCommitError::publication_denied(
        crate::mvcc::RelationalPublicationDenial::HomeProgramContractsRequired,
    )
}

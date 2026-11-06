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
        let construction_contracts = self
            .execution_contracts
            .as_ref()
            .map_or(&self.schema_contract_runtime, |selected| {
                &selected.configuration_contracts
            });
        runtime.schema_contract_runtime = construction_contracts.clone();
        runtime.execution_contracts = None;
        runtime.execution_registry = Some(match selected {
            Some(selected) => selected.schema.registry_arc(),
            None => basis.inner.root.schema_authority().registry_arc(),
        });
        runtime.home_target = target.cloned();
        match selected {
            Some(selected) => {
                if selected.runtime != self.runtime_instance_id()
                    || !Arc::ptr_eq(&selected.configuration, &self.config)
                    || !Arc::ptr_eq(&selected.configuration_contracts, construction_contracts)
                {
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

//! Verify recorded successor meaning against the receiver's actual Home seal.
use super::*;
impl RelationalRuntime {
    /// Admit the actual installed preparation and its issued receipt. Matching
    /// catalog bytes from another preparation cannot substitute its owners.
    pub fn admit_installed_home_program_target(
        &self,
        target: &PreparedRelationalHomeProgramContracts,
        receipt: &super::super::RelationalInitialSchemaInstallationReceipt,
        binding: &worth_proof::AdmittedHomeProgramTransitionSuccessor<
            worth_foundational::facade::HomeProgramTransitionInstallationBinding,
        >,
    ) -> Result<(), RelationalHomeProgramContractDenial> {
        use RelationalHomeProgramContractDenial as Denial;
        self.admit_home_program_transition_installation(binding)?;
        let installed = self.configuration_binding().snapshot();
        let configuration = target
            .configuration
            .as_ref()
            .ok_or(Denial::ReceiverConfigurationRequired)?;
        if target.meaning.runtime != self.runtime_instance_id() {
            return Err(Denial::ForeignRuntime);
        }
        if !Arc::ptr_eq(&installed.config, configuration)
            || !Arc::ptr_eq(
                &installed.schema_contract_runtime,
                &target.meaning.contracts,
            )
            || !receipt.admits_home_target(
                self.runtime_instance_id(),
                &target.meaning.schema,
                &target.meaning.contracts,
            )
        {
            return Err(Denial::TargetCatalogMismatch);
        }
        Ok(())
    }

    pub fn admit_home_program_transition_installation(
        &self,
        binding: &worth_proof::AdmittedHomeProgramTransitionSuccessor<
            worth_foundational::facade::HomeProgramTransitionInstallationBinding,
        >,
    ) -> Result<(), RelationalHomeProgramContractDenial> {
        use RelationalHomeProgramContractDenial as Denial;
        self.durability
            .medium()
            .ok_or(Denial::UnboundHome)?
            .home_transition_authority()
            .ok_or(Denial::UnboundHome)?
            .admit_recorded_successor(binding)
            .map_err(Denial::Admission)
    }
}

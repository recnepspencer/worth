//! Hold the actual final source and receiver epoch across the binding effect.
use super::*;
use crate::branch::RelationalBranchBasisAdmissionIdentity;
use crate::runtime::RelationalInitialSchemaInstallationReceipt;

#[derive(Debug)]
pub enum RelationalHomeProgramPromotionRefused<BindingRefusal> {
    Preparation(RelationalHomeProgramContractDenial),
    Binding(BindingRefusal),
    Admission(crate::durability::medium::RelationalHomeProgramWriteDenial),
}

impl RelationalRuntime {
    /// All allocation and refusal preconditions precede the caller's Store
    /// binding write. The caller returns its genuine Home acknowledgement.
    /// Source movement and receiver replacement are excluded until the pair
    /// is installed and its original writer fence is released, in that order.
    pub fn promote_home_program_after_binding<BindingRefusal>(
        &self,
        request: &ExecutionRequest<'_, '_>,
        target: &PreparedRelationalHomeProgramContracts,
        source: &AdmittedRelationalBranchBasis,
        completed: &worth_proof::CompletedHomeProgramTransitionSteps<
            RelationalBranchBasisAdmissionIdentity,
        >,
        publish_binding: impl FnOnce() -> Result<
            worth_proof::HomeProgramTransitionBindingCommitted,
            BindingRefusal,
        >,
    ) -> Result<
        RelationalInitialSchemaInstallationReceipt,
        RelationalHomeProgramPromotionRefused<BindingRefusal>,
    > {
        use RelationalHomeProgramContractDenial as Denial;
        use RelationalHomeProgramPromotionRefused as Refusal;
        request
            .consult()
            .map_err(|cause| Refusal::Preparation(Denial::Request(cause)))?;
        let owner = self.owner_binding();
        let _operation = owner
            .admit()
            .ok_or(Refusal::Preparation(Denial::OwnerUnavailable))?;
        let configuration = target
            .configuration
            .as_ref()
            .ok_or(Refusal::Preparation(Denial::ReceiverConfigurationRequired))?;
        if target.meaning.runtime != self.runtime_instance_id()
            || source.identity().runtime_instance_id() != self.runtime_instance_id()
        {
            return Err(Refusal::Preparation(Denial::ForeignRuntime));
        }
        if !source
            .inner
            .root
            .execution_contracts
            .get()
            .is_some_and(|meaning| Arc::ptr_eq(meaning, &target.meaning))
        {
            return Err(Refusal::Preparation(Denial::TargetCatalogMismatch));
        }
        let medium = self
            .durability
            .medium()
            .ok_or(Refusal::Preparation(Denial::UnboundHome))?;
        medium
            .admit_completed_home_steps(completed, source.admission_identity())
            .map_err(Refusal::Admission)?;
        let receipt = RelationalInitialSchemaInstallationReceipt::prepare_home_target(
            self.runtime_instance_id(),
            &target.meaning.schema,
            &target.meaning.contracts,
            request,
        )
        .map_err(Refusal::Preparation)?;
        let binding = self.configuration_binding();
        let mut epoch = binding
            .try_home_promotion()
            .ok_or(Refusal::Preparation(Denial::PromotionContended))?;
        if !epoch.same_epoch(&target.receiver_epoch) {
            return Err(Refusal::Preparation(Denial::SourceContractChanged));
        }
        let _coordination = source.inner.publication_cell.coordination().enter();
        let state = source.inner.publication_cell.enter_state();
        if !state.matches_current_basis(source) {
            return Err(Refusal::Preparation(Denial::SourceBasisMismatch));
        }
        request
            .consult()
            .map_err(|cause| Refusal::Preparation(Denial::Request(cause)))?;
        let acknowledgement = publish_binding().map_err(Refusal::Binding)?;
        medium
            .install_acknowledged_home_receiver(
                acknowledgement,
                completed.plan_commitment(),
                || {
                    epoch.config = configuration.clone();
                    epoch.schema_contract_runtime = target.meaning.contracts.clone();
                },
            )
            .map_err(Refusal::Admission)?;
        Ok(receipt)
    }
}

//! Bind a candidate's native publication inputs and its exact Home inventory.

use super::RelationalPreparationRuntime;
use crate::authority::commit::pipeline::publication_execution::{
    PublicationPhaseOwners, PublicationRootCapture,
};
use crate::branch::root::PreparedRelationalBranchRootCapture;
use crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;

impl RelationalPreparationRuntime {
    fn root_capture<'a>(
        &'a self,
        symbols: &'a RuntimeSymbolTable,
    ) -> PublicationRootCapture<'a, impl FnOnce(&PreparedRelationalBranchRootCapture) + 'a> {
        let inventory = self.execution_contracts.as_ref();
        PublicationRootCapture {
            history: &self.history,
            runtime_identity: self.runtime_instance_id(),
            symbols,
            planned_schema: inventory.map(|contracts| &contracts.schema),
            storage: &self.config.storage,
            attach_execution_inventory: move |root: &PreparedRelationalBranchRootCapture| {
                if let Some(contracts) = inventory {
                    crate::runtime::home_program_transition::install_root_execution_contracts(
                        root.root(),
                        contracts,
                    )
                    .expect("prepared Home root has not bound any executor");
                }
            },
        }
    }

    pub(crate) fn publication_phase_owners<'a>(
        &'a self,
        symbols: &'a RuntimeSymbolTable,
    ) -> PublicationPhaseOwners<'a, impl FnOnce(&PreparedRelationalBranchRootCapture) + 'a> {
        PublicationPhaseOwners {
            root: self.root_capture(symbols),
            work: self.commit_work_budget.as_ref(),
            diagnostics: self.preparation_diagnostics(),
            lineage: self.lineage_preparation_authority(),
            fallback_schema: &self.config.schema.registry,
            invariant_generation: self.schema_contract_runtime.custom_invariant_generation,
            max_patch_records: self.config.publication.policy.max_patch_records_per_commit,
            mirror: &self.mirror_residency,
            indexes: &self.index_definitions,
        }
    }

    #[cfg(test)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare_commit_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commit_id: crate::history::data::CommitId,
        commit_reference: crate::history::data::RelationalCommitReceipt,
        binding: &crate::branch::basis::AdmittedRelationalBranchBasis,
        selected_branch_state: &crate::branch::root_selection::SelectedRelationalBranchState,
        published_partition_delta: crate::storage::RelationalPublishedPartitionDelta,
        envelope: Arc<crate::history::data::CanonicalCommitEnvelope>,
        schema_registry: &crate::schema::data::RelationalSchemaRegistry,
        output_positions: Option<crate::history::data::RequiredOutputPublicationPositions>,
    ) -> Result<
        crate::mvcc::publication::authority::PreparedRelationalPublication,
        crate::mvcc::publication::PublicationPreparationDenial,
    > {
        self.root_capture(&self.services.symbols)
            .prepare_commit_publication(
                execution,
                commit_id,
                commit_reference,
                binding,
                selected_branch_state,
                published_partition_delta,
                envelope,
                schema_registry,
                output_positions,
            )
    }
}

impl<'a> From<&'a super::RelationalRuntime>
    for crate::mvcc::publication::authority::RecoveredPublicationInstall<'a>
{
    fn from(runtime: &'a super::RelationalRuntime) -> Self {
        Self {
            history: &runtime.history,
        }
    }
}

#[cfg(test)]
use std::sync::Arc;

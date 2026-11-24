//! Read-only demand authority for the mirror's impending publication set.
use super::StorageSubsystem;
use crate::storage::substrate::owned_state::RuntimeOwnedState;

#[derive(Debug)]
pub struct MirrorResidencyDemandBinding {
    partitions: RuntimeOwnedState<super::CurrentMirror>,
    main_branch: crate::history::data::coordinates::BranchId,
}

impl Clone for MirrorResidencyDemandBinding {
    fn clone(&self) -> Self {
        Self {
            partitions: self.partitions.share(),
            main_branch: self.main_branch.clone(),
        }
    }
}

impl StorageSubsystem {
    pub fn residency_demand_binding(
        &self,
        main_branch: crate::history::data::coordinates::BranchId,
    ) -> MirrorResidencyDemandBinding {
        MirrorResidencyDemandBinding {
            partitions: self.partitions.share(),
            main_branch,
        }
    }
}

impl MirrorResidencyDemandBinding {
    pub fn is_main_branch(&self, branch: &crate::history::data::coordinates::BranchId) -> bool {
        branch == &self.main_branch
    }

    pub fn edition(&self) -> super::PartitionEdition {
        self.partitions.read().edition.clone()
    }

    pub fn admit_current_target(
        &self,
        id: crate::identity::data::PartitionId,
    ) -> Result<
        Option<super::ResidentMirrorTarget>,
        crate::storage::residency::RelationalResidencyDemandFailure,
    > {
        Ok(self.partitions.write().admit_current_target(id)?)
    }

    pub fn demand_head(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        symbols: &crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
        branch: &crate::history::data::coordinates::BranchId,
        ids: impl IntoIterator<Item = crate::identity::data::PartitionId>,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        if branch != &self.main_branch {
            return Ok(());
        }
        let edition = self.partitions.read().edition.clone();
        let source = edition.with_symbol_admission(symbols);
        for id in ids {
            crate::storage::overlay::CurrentPartitionAccess::demand_head_partition(
                &source, execution, id,
            )?;
        }
        Ok(())
    }
    pub fn demand_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        symbols: &crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
        branch: &crate::history::data::coordinates::BranchId,
        ids: impl IntoIterator<Item = crate::identity::data::PartitionId>,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        if branch != &self.main_branch {
            return Ok(());
        }
        let edition = self.partitions.read().edition.clone();
        for id in ids {
            edition.demand_partition_with_symbols(execution, symbols, id)?;
        }
        Ok(())
    }
}

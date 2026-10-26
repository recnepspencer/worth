//! Read-only demand authority for the mirror's impending publication set.
use super::{PartitionEdition, StorageSubsystem};
use crate::runtime::state::subsystems::RuntimeOwnedState;

#[derive(Debug)]
pub(crate) struct MirrorResidencyDemandBinding {
    partitions: RuntimeOwnedState<PartitionEdition>,
    main_branch: crate::history::data::BranchId,
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
    pub(crate) fn residency_demand_binding(
        &self,
        main_branch: crate::history::data::BranchId,
    ) -> MirrorResidencyDemandBinding {
        MirrorResidencyDemandBinding {
            partitions: self.partitions.share(),
            main_branch,
        }
    }
}

impl MirrorResidencyDemandBinding {
    pub(crate) fn demand(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        branch: &crate::history::data::BranchId,
        ids: impl IntoIterator<Item = crate::identity::data::PartitionId>,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        if branch != &self.main_branch {
            return Ok(());
        }
        let edition = self.partitions.read().clone();
        for id in ids {
            edition.demand_partition(execution, id)?;
        }
        Ok(())
    }
}

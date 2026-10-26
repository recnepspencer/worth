//! Read-only demand authority for the mirror's impending publication set.
use super::StorageSubsystem;
use crate::runtime::state::subsystems::RuntimeOwnedState;

#[derive(Debug)]
pub(crate) struct MirrorResidencyDemandBinding {
    partitions: RuntimeOwnedState<super::CurrentMirror>,
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
    pub(crate) fn prepare_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        symbols: &crate::runtime::RuntimeSymbolTable,
        branch: &crate::history::data::BranchId,
        root: &crate::branch::RelationalBranchRoot,
        clone_mode: crate::storage::overlay::PartitionCloneMode,
        committed: std::collections::BTreeMap<
            crate::identity::data::PartitionId,
            (
                crate::storage::overlay::PartitionState,
                crate::storage::overlay::PartitionMutationJournal,
            ),
        >,
    ) -> Result<
        crate::storage::PreparedMirrorPublication,
        crate::storage::residency::RelationalResidencyDemandFailure,
    > {
        crate::execution::check_request(execution)
            .map_err(crate::storage::residency::RelationalResidencyDemandFailure::Request)?;
        let mut prepared = crate::storage::PreparedMirrorPublication::empty();
        if branch != &self.main_branch {
            return Ok(prepared);
        }
        let edition = self.partitions.read().edition.clone();
        for (id, (partition, journal)) in committed {
            edition.demand_partition_with_symbols(execution, symbols, id)?;
            let target = self.partitions.write().admit_target(id)?;
            match target {
                Some(target) => prepared.prepare_existing(target, clone_mode, partition, journal),
                None => {
                    let lease = root.resident_full_region_witness(id)?.ok_or(
                        crate::storage::data::RelationalStorageUnavailable {
                            subject: crate::storage::data::RelationalStorageSubject::Region(id),
                            cause: crate::storage::data::RelationalStorageUnavailableCause::SourceUnavailable,
                        }
                    )?;
                    prepared.prepare_new(super::ResidentMirrorTarget::new(lease, false), &journal);
                }
            }
        }
        Ok(prepared)
    }

    pub(crate) fn demand(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        symbols: &crate::runtime::RuntimeSymbolTable,
        branch: &crate::history::data::BranchId,
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

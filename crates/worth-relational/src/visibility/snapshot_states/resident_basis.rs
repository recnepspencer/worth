//! Selected root and isolated dictionary custody for per-region read demands.
use super::VisibilitySnapshotBasis;
use crate::branch::RelationalBranchRoot;
use crate::identity::data::{PartitionId, VersionId};
use crate::storage::residency::RelationalRegionHeadView;
use std::sync::Arc;
mod regions;
use regions::ReadRegionCustody;

/// Retains the exact selected root and one isolated persistent dictionary.
/// Residency is established per touched region by the operation's request.
/// This custody grants no claim that an untouched region is already Full.
#[derive(Clone, Debug)]
pub(crate) struct ResidentVisibilitySnapshotBasis {
    basis: VisibilitySnapshotBasis,
    symbols: crate::runtime::RuntimeSymbolTable,
    regions: Arc<ReadRegionCustody>,
}
impl ResidentVisibilitySnapshotBasis {
    pub(crate) fn admit(
        basis: VisibilitySnapshotBasis,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        symbols: &crate::runtime::RuntimeSymbolTable,
    ) -> Result<Self, crate::execution::RelationalExecutionDenialCause> {
        crate::execution::check_request(request)?;
        Ok(Self {
            basis,
            symbols: symbols.detached_owner_snapshot(),
            regions: Arc::default(),
        })
    }
    pub(crate) fn basis(&self) -> &VisibilitySnapshotBasis {
        &self.basis
    }
    /// One isolated dictionary owner per pin; projections retain its shared
    /// persistent maps instead of reconstructing all symbol entries per read.
    pub(crate) fn symbols(&self) -> &crate::runtime::RuntimeSymbolTable {
        &self.symbols
    }
    pub(crate) fn root(&self) -> &Arc<RelationalBranchRoot> {
        self.basis.root()
    }
    pub(crate) fn version_id(&self) -> VersionId {
        self.basis.version_id()
    }
    pub(crate) fn branch_id(&self) -> &crate::history::data::BranchId {
        self.basis.branch_id()
    }
    pub(crate) fn head_region(&self, id: PartitionId) -> Option<RelationalRegionHeadView<'_>> {
        self.regions
            .has_head(id)
            .then(|| self.root().accepted_head_region_view(id))
            .flatten()
    }
    pub(crate) fn full_partition(
        &self,
        id: PartitionId,
    ) -> Option<&crate::storage::overlay::PartitionState> {
        self.regions
            .has_full(id)
            .then(|| self.root().accepted_full_region(id))
            .flatten()
    }
    pub(crate) fn demand_regions_head(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        ids: impl IntoIterator<Item = PartitionId>,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        use crate::storage::residency::RelationalResidencyDemandFailure as Failure;
        crate::execution::check_request(request).map_err(Failure::Request)?;
        for id in ids {
            if self.regions.has_head(id) {
                continue;
            }
            if let Some(lease) = self.root().demand_region_head(request, id)? {
                lease.admit_symbols(request, self.symbols())?;
                self.regions.retain_head(id, lease);
            }
        }
        Ok(())
    }
    pub(crate) fn demand_regions_full(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        ids: impl IntoIterator<Item = PartitionId>,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        use crate::storage::residency::RelationalResidencyDemandFailure as Failure;
        crate::execution::check_request(request).map_err(Failure::Request)?;
        for id in ids {
            if self.regions.has_full(id) {
                continue;
            }
            if let Some(lease) = self.root().demand_region_full(request, id)? {
                lease.admit_symbols(request, self.symbols())?;
                self.regions.retain_full(id, lease);
            }
        }
        Ok(())
    }
}
impl crate::storage::overlay::PartitionAccess for ResidentVisibilitySnapshotBasis {
    fn get_partition(&self, id: PartitionId) -> Option<&crate::storage::overlay::PartitionState> {
        self.full_partition(id)
    }
    fn probe_full_partition(
        &self,
        id: PartitionId,
    ) -> crate::storage::residency::PartitionProbe<'_> {
        use crate::storage::residency::{PartitionHeadProbe as Head, PartitionProbe as Full};
        if let Some(partition) = self.full_partition(id) {
            return Full::Resident(partition);
        }
        match self.root().probe_head_region(id) {
            Head::Absent => Full::Absent,
            _ => Full::Cold,
        }
    }
    fn demand_full_partition(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        self.demand_regions_full(request, [id])
    }
}

impl crate::storage::overlay::CurrentPartitionAccess for ResidentVisibilitySnapshotBasis {
    fn get_current_partition(&self, id: PartitionId) -> Option<RelationalRegionHeadView<'_>> {
        self.head_region(id)
    }
    fn probe_head_partition(
        &self,
        id: PartitionId,
    ) -> crate::storage::residency::PartitionHeadProbe<'_> {
        use crate::storage::residency::PartitionHeadProbe as Probe;
        if let Some(head) = self.head_region(id) {
            return Probe::Resident(head);
        }
        match self.root().probe_head_region(id) {
            Probe::Absent => Probe::Absent,
            _ => Probe::Cold,
        }
    }
    fn demand_head_partition(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        self.demand_regions_head(request, [id])
    }
    fn partition_ids(&self) -> Vec<PartitionId> {
        self.root().partition_ids()
    }
    fn partition_ids_with_memory(
        &self,
        memory: &mut dyn crate::storage::overlay::PartitionIdMemory,
    ) -> Result<Vec<PartitionId>, crate::storage::overlay::PartitionIdsDenial> {
        crate::storage::overlay::CurrentPartitionAccess::partition_ids_with_memory(
            self.root().as_ref(),
            memory,
        )
    }
    fn partition_ids_iter(&self) -> Box<dyn Iterator<Item = PartitionId> + '_> {
        Box::new(self.root().partition_ids_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selected_basis_cannot_infallibly_become_resident_custody() {
        trait WithoutPromotion<A> {
            fn witness() {}
        }
        impl<T> WithoutPromotion<()> for T {}
        impl<T: Into<ResidentVisibilitySnapshotBasis>> WithoutPromotion<u8> for T {}
        let _ = <VisibilitySnapshotBasis as WithoutPromotion<_>>::witness;
        trait WithoutDefault<A> {
            fn witness() {}
        }
        impl<T> WithoutDefault<()> for T {}
        impl<T: Default> WithoutDefault<u8> for T {}
        let _ = <ResidentVisibilitySnapshotBasis as WithoutDefault<_>>::witness;
    }
}

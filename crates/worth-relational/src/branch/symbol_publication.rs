//! Bind the prepared Branch change to its exact resident symbol admissions.
use super::{maintenance::CommitChange, root::RelationalBranchRoot};
use crate::identity::data::PartitionId;
use crate::storage::residency::{
    RegionSymbolPublication, RelationalResidencyDemandFailure, SymbolPublicationKey,
};
use crate::symbols::data::Symbol;
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub(crate) struct SymbolPublicationAdmission {
    key: SymbolPublicationKey,
    change: Arc<CommitChange>,
    regions: Arc<BTreeMap<PartitionId, RegionSymbolPublication>>,
}
impl SymbolPublicationAdmission {
    pub(crate) fn prepare(
        root: &RelationalBranchRoot,
    ) -> Result<Self, RelationalResidencyDemandFailure> {
        let change = root
            .commit_change()
            .expect(/* Proof: This door is called only for the prepared canonical publication root; root capture installs its immutable CommitChange before returning PreparedRelationalHistoryPublication. Restored or initial uncommitted roots never enter publication preflight. */ "prepared publication root carries its change")
            .clone();
        let key = SymbolPublicationKey::new();
        let mut regions = BTreeMap::new();
        for (id, _) in change.partitions() {
            let lease = root
                .accepted_head_region_witness(id)
                .map_err(missing)?
                .ok_or_else(|| missing(id))?;
            let region = lease.symbol_publication(&key).ok_or_else(|| missing(id))?;
            regions.insert(id, region);
        }
        Ok(Self {
            key,
            change,
            regions: Arc::new(regions),
        })
    }
    pub(crate) fn for_region(
        &self,
        change: &Arc<CommitChange>,
        id: PartitionId,
    ) -> Option<RegionSymbolPublication> {
        Arc::ptr_eq(change, &self.change)
            .then(|| self.regions.get(&id).cloned())
            .flatten()
    }
    pub(crate) fn required(&self) -> impl Iterator<Item = Symbol> + '_ {
        self.regions
            .values()
            .flat_map(RegionSymbolPublication::required_symbols)
    }
    pub(crate) fn key(&self) -> &SymbolPublicationKey {
        &self.key
    }
}
fn missing(id: PartitionId) -> RelationalResidencyDemandFailure {
    RelationalResidencyDemandFailure::StorageUnavailable(crate::storage::data::RelationalStorageUnavailable {
        subject: crate::storage::data::RelationalStorageSubject::Region(id),
        cause: crate::storage::data::RelationalStorageUnavailableCause::MissingSymbolMaintenancePredecessor,
    })
}

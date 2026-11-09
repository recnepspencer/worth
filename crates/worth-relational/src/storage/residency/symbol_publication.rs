//! Acceptance scope for the symbol closure of one canonical publication.
use super::*;
use crate::identity::data::PartitionId;
use crate::storage::substrate::maintenance::CommitChange;
use crate::symbols::data::Symbol;
use std::collections::BTreeMap;
use std::sync::Arc;
#[derive(Debug, Clone)]
pub(crate) struct SymbolPublicationAdmission {
    key: SymbolPublicationKey,
    change: Arc<CommitChange>,
    regions: Arc<BTreeMap<PartitionId, Arc<RegionSymbolChanges>>>,
}
#[derive(Debug, Clone)]
pub(crate) struct RegionSymbolPublication {
    pub(super) publication: SymbolPublicationKey,
    pub(super) changes: Arc<RegionSymbolChanges>,
}
impl SymbolPublicationAdmission {
    pub(crate) fn prepare(
        root: &crate::branch::RelationalBranchRoot,
    ) -> Result<Self, RelationalResidencyDemandFailure> {
        let change = root
            .commit_change()
            .expect(/* Proof: This door is called only for the prepared canonical publication root; root capture installs its immutable CommitChange before returning PreparedRelationalHistoryPublication. Restored or initial uncommitted roots never enter publication preflight. */ "prepared publication root carries its change")
            .clone();
        let mut regions = BTreeMap::new();
        for (id, _) in change.partitions() {
            let lease = root
                .accepted_head_region_witness(id)
                .map_err(missing)?
                .ok_or_else(|| missing(id))?;
            let changes = lease
                .cell()
                .symbol_inheritance
                .get()
                .map(|basis| basis.changes.clone())
                .or_else(|| {
                    lease
                        .resident_region()
                        .symbol_requirements
                        .get()
                        .and_then(|seed| seed.changes())
                });
            regions.insert(id, changes.ok_or_else(|| missing(id))?);
        }
        Ok(Self {
            key: SymbolPublicationKey(Arc::new(())),
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
            .map(|changes| RegionSymbolPublication {
                publication: self.key.clone(),
                changes,
            })
    }
    pub(crate) fn required(&self) -> impl Iterator<Item = Symbol> + '_ {
        self.regions
            .values()
            .flat_map(|changes| changes.owned.ids.iter().copied())
    }
    pub(crate) fn key(&self) -> &SymbolPublicationKey {
        &self.key
    }
}
/// Retained cells carry only the accepted dictionary predicate, never a root
/// or canonical envelope. That authority remains with the publication scope.
#[derive(Debug, Clone)]
pub(crate) struct SymbolPublicationKey(Arc<()>);
impl SymbolPublicationKey {
    pub(crate) fn identity(&self) -> RelationalRegionSymbolAdmissionIdentity {
        RelationalRegionSymbolAdmissionIdentity::publication(&self.0)
    }
}
fn missing(id: PartitionId) -> RelationalResidencyDemandFailure {
    RelationalResidencyDemandFailure::StorageUnavailable(crate::storage::data::RelationalStorageUnavailable {
        subject: crate::storage::data::RelationalStorageSubject::Region(id),
        cause: crate::storage::data::RelationalStorageUnavailableCause::MissingSymbolMaintenancePredecessor,
    })
}

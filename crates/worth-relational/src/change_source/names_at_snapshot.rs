//! Persistent-name reads at a decision seal's snapshot: its branch's head.
use super::names_at_observation::RelationalPersistentNameReadDenial as Denial;
use crate::lineage::access::PersistentNameObservationView;
use crate::runtime::RelationalRuntime;
use crate::snapshots::data::SnapshotHandle;
use worth_relational_lineage::internal_port::lineage::access::name_observation_view::PersistentNameObservationViewOwnerPort as _;

impl RelationalRuntime {
    /// Runs `read` over `snapshot`'s branch names at the commit that
    /// published the snapshot's version, which must still be the branch's
    /// head. Refuses `EmptyBranch` for an empty or no-longer-head version,
    /// and an unsettled or unavailable index as an observation read does.
    ///
    /// Complexity: O(1) head lookup plus whatever `read` asks the view for.
    pub fn with_persistent_names_at_snapshot<T>(
        &self,
        snapshot: &SnapshotHandle,
        read: impl FnOnce(PersistentNameObservationView<'_>) -> T,
    ) -> Result<T, Denial> {
        let root = self
            .history
            .branch_cell(snapshot.branch_id())
            .and_then(|cell| cell.root());
        let receipt = root.and_then(|root| root.commit_receipt().cloned());
        let basis = receipt
            .filter(|receipt| receipt.version_id == snapshot.version_id())
            .map(|receipt| receipt.commit_id)
            .ok_or(Denial::EmptyBranch)?;
        if self
            .publication_binding()
            .has_unsettled_publication_through(basis)
        {
            return Err(Denial::SettlementInFlight { basis });
        }
        self.lineage
            .read_name_index(snapshot.branch_id(), |index| {
                read(PersistentNameObservationView::new(index, basis))
            })
            .map_err(Denial::IndexUnavailable)
    }
}

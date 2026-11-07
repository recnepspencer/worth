//! Concrete region leases accepted by one lazy read capability and its clones.
use crate::identity::data::PartitionId;
use crate::storage::residency::{Full, Head, RelationalResidencyLease};
use std::collections::BTreeMap;
use std::sync::Mutex;

enum AcceptedRegion {
    Head(RelationalResidencyLease<Head>),
    Full(RelationalResidencyLease<Full>),
}
#[derive(Default)]
pub(super) struct ReadRegionCustody(Mutex<BTreeMap<PartitionId, AcceptedRegion>>);
impl std::fmt::Debug for ReadRegionCustody {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.debug_struct("ReadRegionCustody")
            .finish_non_exhaustive()
    }
}
impl ReadRegionCustody {
    pub(super) fn has_head(&self, id: PartitionId) -> bool {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&id)
            .is_some_and(|region| match region {
                AcceptedRegion::Head(lease) => lease.head_view().partition_id() == id,
                AcceptedRegion::Full(lease) => lease.head_view().partition_id() == id,
            })
    }
    pub(super) fn has_full(&self, id: PartitionId) -> bool {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&id)
            .is_some_and(|region| match region {
                AcceptedRegion::Full(lease) => lease.head_view().partition_id() == id,
                AcceptedRegion::Head(_) => false,
            })
    }
    pub(super) fn retain_head(&self, id: PartitionId, lease: RelationalResidencyLease<Head>) {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entry(id)
            .or_insert(AcceptedRegion::Head(lease));
    }
    pub(super) fn retain_full(&self, id: PartitionId, lease: RelationalResidencyLease<Full>) {
        let mut regions = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !matches!(regions.get(&id), Some(AcceptedRegion::Full(_))) {
            regions.insert(id, AcceptedRegion::Full(lease));
        }
    }
}

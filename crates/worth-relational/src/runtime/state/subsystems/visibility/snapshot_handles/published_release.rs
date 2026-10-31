//! A fact about one original published registration, never a release obligation.
use super::super::SnapshotHandleBinding;
use super::published_registry::PublishedSnapshotHandleRegistry;
use crate::snapshots::data::{SnapshotHandle, SnapshotId, SnapshotReadPolicy};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Weak,
};

#[derive(Debug)]
pub(crate) struct PublishedSnapshotReleaseObservation {
    registry: Weak<PublishedSnapshotHandleRegistry>,
    snapshot_id: SnapshotId,
    branch_id: crate::history::data::BranchId,
    version_id: crate::identity::data::VersionId,
    read_policy: SnapshotReadPolicy,
    released: AtomicBool,
}
impl PublishedSnapshotReleaseObservation {
    pub(super) fn new(
        registry: &Arc<PublishedSnapshotHandleRegistry>,
        snapshot_id: SnapshotId,
        binding: &SnapshotHandleBinding,
    ) -> Arc<Self> {
        Arc::new(Self {
            registry: Arc::downgrade(registry),
            snapshot_id,
            branch_id: binding.branch_id.clone(),
            version_id: binding.version_id,
            read_policy: binding.read_policy,
            released: AtomicBool::new(false),
        })
    }
    pub(super) fn mark_released(&self) {
        self.released.store(true, Ordering::Release);
    }
    pub(super) fn belongs_to(&self, registry: &Arc<PublishedSnapshotHandleRegistry>) -> bool {
        Weak::ptr_eq(&self.registry, &Arc::downgrade(registry))
    }
    pub(crate) fn matches(&self, handle: &SnapshotHandle) -> bool {
        self.snapshot_id == handle.snapshot_id()
            && &self.branch_id == handle.branch_id()
            && self.version_id == handle.version_id()
            && self.read_policy == handle.read_policy()
    }
    pub(crate) fn was_released(&self) -> bool {
        self.released.load(Ordering::Acquire)
    }
}
impl PartialEq for PublishedSnapshotReleaseObservation {
    fn eq(&self, other: &Self) -> bool {
        self.snapshot_id == other.snapshot_id && Weak::ptr_eq(&self.registry, &other.registry)
    }
}
impl Eq for PublishedSnapshotReleaseObservation {}

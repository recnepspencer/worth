//! Original-registration fact and its actual removed-binding retirement custody.
use super::super::SnapshotHandleBinding;
use super::published_registry::PublishedSnapshotHandleRegistry;
use crate::snapshots::data::{SnapshotHandle, SnapshotId, SnapshotReadPolicy};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, Weak,
};

#[derive(Debug)]
pub(crate) struct PublishedSnapshotReleaseObservation {
    registry: Weak<PublishedSnapshotHandleRegistry>,
    snapshot_id: SnapshotId,
    branch_id: crate::history::data::BranchId,
    version_id: crate::identity::data::VersionId,
    read_policy: SnapshotReadPolicy,
    released: AtomicBool,
    pub(super) retirement: Mutex<PublishedSnapshotRetirement>,
}
#[derive(Debug)]
pub(super) enum PublishedSnapshotRetirement {
    Registered,
    Removed(PublishedSnapshotBindingRetirement),
    Interrupted,
    Complete,
}
#[derive(Debug)]
pub(super) struct PublishedSnapshotBindingRetirement {
    binding: Option<SnapshotHandleBinding>,
    basis: Option<crate::visibility::snapshot_states::VisibilitySnapshotBasisRetirement>,
}
impl PublishedSnapshotRetirement {
    pub(super) fn removed(binding: SnapshotHandleBinding) -> Self {
        Self::Removed(PublishedSnapshotBindingRetirement {
            binding: Some(binding),
            basis: None,
        })
    }
}
impl PublishedSnapshotBindingRetirement {
    fn advance(&mut self) -> Result<bool, ()> {
        if let Some(binding) = self.binding.take() {
            self.basis = Some(binding.basis.into_retirement());
            return Ok(false);
        }
        match self.basis.as_mut() {
            Some(basis) => basis.advance(),
            None => Ok(true),
        }
    }
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
            retirement: Mutex::new(PublishedSnapshotRetirement::Registered),
        })
    }
    pub(super) fn mark_released(&self) {
        self.released.store(true, Ordering::Release);
    }
    pub(super) fn snapshot_id(&self) -> SnapshotId {
        self.snapshot_id
    }
    pub(super) fn install_removed(&self, binding: SnapshotHandleBinding) {
        let mut phase = self
            .retirement
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        *phase = PublishedSnapshotRetirement::removed(binding);
        self.mark_released();
    }
    /// Legacy synchronous release joins the SAME actual retirement phase.
    /// This method and final Drop deliberately retain their unbudgeted contract.
    pub(crate) fn finish_retirement_blocking(&self) -> bool {
        let mut phase = self
            .retirement
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if matches!(*phase, PublishedSnapshotRetirement::Removed(_)) {
            let retired = std::mem::replace(&mut *phase, PublishedSnapshotRetirement::Interrupted);
            drop(retired);
            *phase = PublishedSnapshotRetirement::Complete;
        }
        matches!(*phase, PublishedSnapshotRetirement::Complete)
    }
    pub(crate) fn retirement_complete(&self) -> Result<Option<bool>, ()> {
        let phase = match self.retirement.try_lock() {
            Ok(guard) => guard,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Ok(None),
        };
        if matches!(*phase, PublishedSnapshotRetirement::Interrupted) {
            return Err(());
        }
        Ok(Some(matches!(
            *phase,
            PublishedSnapshotRetirement::Complete
        )))
    }
    /// Called only under an authentic Native owner borrow, after exact removal.
    pub(crate) fn advance_retirement(&self) -> Result<Option<bool>, ()> {
        let mut phase = match self.retirement.try_lock() {
            Ok(guard) => guard,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Ok(None),
        };
        match &mut *phase {
            PublishedSnapshotRetirement::Registered => Err(()),
            PublishedSnapshotRetirement::Interrupted => Err(()),
            PublishedSnapshotRetirement::Complete => Ok(Some(true)),
            PublishedSnapshotRetirement::Removed(retirement) => match retirement.advance() {
                Err(()) => Ok(None),
                Ok(false) => Ok(Some(false)),
                Ok(true) => {
                    *phase = PublishedSnapshotRetirement::Complete;
                    Ok(Some(true))
                }
            },
        }
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

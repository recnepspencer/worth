//! Exact product-branch coordination for application commit progression.
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::TryLockError;
use std::sync::Weak;
use worth_runtime_world::facade::ProductBranchIncarnation;
use worth_runtime_world::facade::ProductBranchObservation;
mod contention;
pub use contention::WorthQueryApplicationBranchCommitContentionForTest;
pub(super) struct WorthQueryApplicationBranchCommitCoordinator {
    contention: Arc<contention::ContentionControl>,
    lanes: Mutex<BTreeMap<ProductBranchIncarnation, Weak<WorthQueryApplicationBranchCommitLane>>>,
}
pub(crate) struct WorthQueryApplicationBranchCommitLane {
    contention: Arc<contention::ContentionControl>,
    occurrence: ProductBranchIncarnation,
    transition: Mutex<()>,
}
pub(crate) struct WorthQueryApplicationBranchCommitCoordination<'lane> {
    occurrence: ProductBranchIncarnation,
    _guard: MutexGuard<'lane, ()>,
}
impl WorthQueryApplicationBranchCommitCoordinator {
    pub(super) fn new() -> Self {
        Self {
            contention: Arc::default(),
            lanes: Mutex::new(BTreeMap::new()),
        }
    }
    pub(super) fn observe_contention_for_test(
        &self,
    ) -> WorthQueryApplicationBranchCommitContentionForTest {
        self.contention.arm()
    }
    pub(super) fn lane_for(
        &self,
        observation: &ProductBranchObservation,
    ) -> Arc<WorthQueryApplicationBranchCommitLane> {
        self.lane_for_occurrence(observation.lifecycle_incarnation())
    }
    pub(super) fn lane_for_occurrence(
        &self,
        occurrence: ProductBranchIncarnation,
    ) -> Arc<WorthQueryApplicationBranchCommitLane> {
        let mut lanes = self
            .lanes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(lane) = lanes.get(&occurrence).and_then(Weak::upgrade) {
            return lane;
        }
        let lane = Arc::new(WorthQueryApplicationBranchCommitLane {
            contention: Arc::clone(&self.contention),
            occurrence,
            transition: Mutex::new(()),
        });
        lanes.insert(occurrence, Arc::downgrade(&lane));
        lane
    }
    pub(super) fn retire(&self, occurrence: ProductBranchIncarnation) {
        self.lanes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&occurrence);
    }
    #[cfg(test)]
    fn entry_count(&self) -> usize {
        self.lanes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }
}
impl WorthQueryApplicationBranchCommitLane {
    pub(crate) fn enter(&self) -> WorthQueryApplicationBranchCommitCoordination<'_> {
        // Ordinary entry is the same blocking mutex. The receiver's armed
        // certification recorder only reports an actual native contention.
        let guard = if self.contention.armed() {
            match self.transition.try_lock() {
                Ok(guard) => guard,
                Err(TryLockError::Poisoned(error)) => error.into_inner(),
                Err(TryLockError::WouldBlock) => {
                    self.contention.observe(self.occurrence);
                    self.transition
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                }
            }
        } else {
            self.transition
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        };
        WorthQueryApplicationBranchCommitCoordination {
            occurrence: self.occurrence,
            _guard: guard,
        }
    }
}

impl WorthQueryApplicationBranchCommitCoordination<'_> {
    pub(in crate::domain_computation) fn admits(
        &self,
        observation: &ProductBranchObservation,
    ) -> bool {
        self.occurrence == observation.lifecycle_incarnation()
    }
}

#[cfg(test)]
mod tests;

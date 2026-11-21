//! Correction commands serialize across product branches through the existing coordinator.
use crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard, Weak};

type CorrectionLanes = Mutex<BTreeMap<[u8; 32], Weak<WorthQueryCorrectionCommitLane>>>;

#[derive(Default)]
pub struct CorrectionCommitLanes {
    lanes: Arc<CorrectionLanes>,
}
pub struct WorthQueryCorrectionCommitLane {
    key: [u8; 32],
    owner: Weak<CorrectionLanes>,
    transition: Mutex<()>,
}
impl CorrectionCommitLanes {
    pub fn for_binding(
        &self,
        binding: &WorthQueryApplicationIdempotencyBinding,
    ) -> Option<Arc<WorthQueryCorrectionCommitLane>> {
        binding
            .correction_identity()
            .map(|_| self.for_key(binding.key_identity()))
    }
    pub fn for_key(&self, key: &[u8; 32]) -> Arc<WorthQueryCorrectionCommitLane> {
        let mut lanes = self
            .lanes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(lane) = lanes.get(key).and_then(Weak::upgrade) {
            return lane;
        }
        let lane = Arc::new(WorthQueryCorrectionCommitLane {
            key: *key,
            owner: Arc::downgrade(&self.lanes),
            transition: Mutex::new(()),
        });
        lanes.insert(*key, Arc::downgrade(&lane));
        lane
    }
}
impl WorthQueryCorrectionCommitLane {
    pub fn enter(&self) -> MutexGuard<'_, ()> {
        self.transition
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
impl Drop for WorthQueryCorrectionCommitLane {
    fn drop(&mut self) {
        let Some(owner) = self.owner.upgrade() else {
            return;
        };
        let mut lanes = owner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if lanes
            .get(&self.key)
            .is_some_and(|lane| std::ptr::eq(lane.as_ptr(), self))
        {
            lanes.remove(&self.key);
        }
    }
}


//! Correction commands serialize across product branches through the existing coordinator.
use crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard, Weak};

type CorrectionLanes = Mutex<BTreeMap<[u8; 32], Weak<WorthQueryCorrectionCommitLane>>>;

#[derive(Default)]
pub(super) struct CorrectionCommitLanes {
    lanes: Arc<CorrectionLanes>,
}
pub(crate) struct WorthQueryCorrectionCommitLane {
    key: [u8; 32],
    owner: Weak<CorrectionLanes>,
    transition: Mutex<()>,
}
impl CorrectionCommitLanes {
    pub(super) fn for_binding(
        &self,
        binding: &WorthQueryApplicationIdempotencyBinding,
    ) -> Option<Arc<WorthQueryCorrectionCommitLane>> {
        binding
            .correction_identity()
            .map(|_| self.for_key(binding.key_identity()))
    }
    pub(super) fn for_key(&self, key: &[u8; 32]) -> Arc<WorthQueryCorrectionCommitLane> {
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
    pub(crate) fn enter(&self) -> MutexGuard<'_, ()> {
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

impl super::super::WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn correction_commit_lanes(
        &self,
        bindings: impl IntoIterator<Item = WorthQueryApplicationIdempotencyBinding>,
    ) -> Vec<Arc<WorthQueryCorrectionCommitLane>> {
        let lanes = bindings
            .into_iter()
            .filter_map(|binding| {
                self.correction_commit_lane(&binding)
                    .map(|lane| (*binding.key_identity(), lane))
            })
            .collect::<BTreeMap<_, _>>();
        lanes.into_values().collect()
    }

    pub(in crate::domain_computation::primary_graph) fn correction_commit_lane(
        &self,
        binding: &WorthQueryApplicationIdempotencyBinding,
    ) -> Option<Arc<WorthQueryCorrectionCommitLane>> {
        self.branch_commit_coordination
            .corrections
            .for_binding(binding)
    }
    pub(in crate::domain_computation::primary_graph::provider) fn correction_key_lane(
        &self,
        key: &[u8; 32],
    ) -> Arc<WorthQueryCorrectionCommitLane> {
        self.branch_commit_coordination.corrections.for_key(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_live_command_lane_is_shared_until_its_last_owner_releases_it() {
        let owner = CorrectionCommitLanes::default();
        let first = owner.for_key(&[4; 32]);
        let second = owner.for_key(&[4; 32]);
        assert!(Arc::ptr_eq(&first, &second));
        let guard = first.enter();
        assert!(matches!(
            second.transition.try_lock(),
            Err(std::sync::TryLockError::WouldBlock)
        ));
        assert!(!Arc::ptr_eq(&first, &owner.for_key(&[5; 32])));
        drop(guard);
        drop(first);
        assert_eq!(owner.lanes.lock().unwrap().len(), 1);
        let third = owner.for_key(&[4; 32]);
        assert!(Arc::ptr_eq(&second, &third));
        drop(second);
        drop(third);
        assert!(
            owner.lanes.lock().unwrap().is_empty(),
            "idle client keys retain no lane entries"
        );
    }
}

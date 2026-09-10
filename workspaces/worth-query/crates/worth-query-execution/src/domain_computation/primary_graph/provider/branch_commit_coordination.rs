//! Exact product-branch coordination for application commit progression.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard, Weak};

use worth_runtime_world::facade::{ProductBranchIncarnation, ProductBranchObservation};

#[derive(Default)]
pub(super) struct WorthQueryApplicationBranchCommitCoordinator {
    lanes: Mutex<BTreeMap<ProductBranchIncarnation, Weak<WorthQueryApplicationBranchCommitLane>>>,
}

pub(in crate::domain_computation) struct WorthQueryApplicationBranchCommitLane {
    occurrence: ProductBranchIncarnation,
    transition: Mutex<()>,
}

pub(in crate::domain_computation) struct WorthQueryApplicationBranchCommitCoordination<'lane> {
    occurrence: ProductBranchIncarnation,
    _guard: MutexGuard<'lane, ()>,
}

impl WorthQueryApplicationBranchCommitCoordinator {
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
}

impl WorthQueryApplicationBranchCommitLane {
    pub(in crate::domain_computation) fn enter(
        &self,
    ) -> WorthQueryApplicationBranchCommitCoordination<'_> {
        WorthQueryApplicationBranchCommitCoordination {
            occurrence: self.occurrence,
            _guard: self
                .transition
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
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

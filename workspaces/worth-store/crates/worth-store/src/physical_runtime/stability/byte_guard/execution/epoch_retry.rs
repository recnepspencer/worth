use worth_store_physical_isolation::{ManifestEpoch, PhysicalReadPlanRetryPosture, RootEpoch};

use super::StablePhysicalReadExecutionCounters;

#[derive(Debug, Clone, Copy)]
pub struct EpochRetryReceipt {
    admitted_root_epoch: RootEpoch,
    observed_root_epoch: RootEpoch,
    admitted_manifest_epoch: ManifestEpoch,
    observed_manifest_epoch: ManifestEpoch,
    retry_posture: PhysicalReadPlanRetryPosture,
    counters: StablePhysicalReadExecutionCounters,
}

impl EpochRetryReceipt {
    pub(crate) const fn new(
        admitted_root_epoch: RootEpoch,
        observed_root_epoch: RootEpoch,
        admitted_manifest_epoch: ManifestEpoch,
        observed_manifest_epoch: ManifestEpoch,
        retry_posture: PhysicalReadPlanRetryPosture,
        counters: StablePhysicalReadExecutionCounters,
    ) -> Self {
        Self {
            admitted_root_epoch,
            observed_root_epoch,
            admitted_manifest_epoch,
            observed_manifest_epoch,
            retry_posture,
            counters,
        }
    }

    pub const fn admitted_root_epoch(self) -> RootEpoch {
        self.admitted_root_epoch
    }

    pub const fn observed_root_epoch(self) -> RootEpoch {
        self.observed_root_epoch
    }

    pub const fn admitted_manifest_epoch(self) -> ManifestEpoch {
        self.admitted_manifest_epoch
    }

    pub const fn observed_manifest_epoch(self) -> ManifestEpoch {
        self.observed_manifest_epoch
    }

    pub const fn retry_posture(self) -> PhysicalReadPlanRetryPosture {
        self.retry_posture
    }

    pub const fn counters(self) -> StablePhysicalReadExecutionCounters {
        self.counters
    }
}

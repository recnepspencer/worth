use worth_proof::{CanonicalVec, NonEmpty};

use super::IntegrityRepairRegion;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrityRepairClassificationDenial {
    EmptyRegions,
    DuplicateRegion,
    AllocationFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::workflow::repair) struct IntegrityRepairClassificationPlan {
    pub(in crate::workflow::repair) fingerprint: [u8; 32],
    pub(in crate::workflow::repair) regions: CanonicalVec<IntegrityRepairRegion>,
    pub(in crate::workflow::repair) non_empty: NonEmpty<[u8; 32]>,
}

impl IntegrityRepairClassificationPlan {
    pub(in crate::workflow::repair) const fn fingerprint(&self) -> [u8; 32] {
        self.fingerprint
    }

    pub(in crate::workflow::repair) fn regions(&self) -> &[IntegrityRepairRegion] {
        self.regions.as_slice()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntegrityRepairClassificationReceipt {
    pub(in crate::workflow::repair) plan_fingerprint: [u8; 32],
    pub(in crate::workflow::repair) classified_regions: u64,
    pub(in crate::workflow::repair) quarantined_regions: u64,
}

impl IntegrityRepairClassificationReceipt {
    pub const fn plan_fingerprint(self) -> [u8; 32] {
        self.plan_fingerprint
    }

    pub const fn classified_regions(self) -> u64 {
        self.classified_regions
    }

    pub const fn quarantined_regions(self) -> u64 {
        self.quarantined_regions
    }
}

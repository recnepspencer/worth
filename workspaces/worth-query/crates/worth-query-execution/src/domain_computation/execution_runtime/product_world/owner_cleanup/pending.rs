//! Existing cleanup ordering over either original backing owner.
use worth_runtime_world::facade::{FundedOwnerRetirementWork, OwnerRetirementWork};

pub(super) enum PendingOwnerWork {
    Legacy(Vec<OwnerRetirementWork>),
    Funded(FundedOwnerRetirementWork),
}

impl PendingOwnerWork {
    pub(super) fn as_slice(&self) -> &[OwnerRetirementWork] {
        match self {
            Self::Legacy(work) => work,
            Self::Funded(work) => work.as_slice(),
        }
    }

    pub(super) fn removal_work(&self) -> u64 {
        match self {
            Self::Legacy(_) => 1,
            Self::Funded(work) => u64::try_from(work.as_slice().len()).unwrap_or(u64::MAX),
        }
    }

    pub(super) fn remove_first(&mut self) {
        match self {
            Self::Legacy(work) => {
                work.remove(0);
            }
            Self::Funded(work) => {
                work.remove(0);
            }
        }
    }
}

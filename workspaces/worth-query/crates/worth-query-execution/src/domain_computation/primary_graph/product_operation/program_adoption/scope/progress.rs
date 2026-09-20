use worth_runtime_world::facade::NoEffectCompositePublication;

use super::super::{WorthQueryPerformedBranchAdoption, WorthQueryUnpublishedBranchAdoption};
use crate::basis::WorthQueryProductBranch;

pub enum WorthQueryBranchSetAdoptionProgress {
    Performed {
        branch: WorthQueryProductBranch,
        adoption: WorthQueryPerformedBranchAdoption,
    },
    NoEffect {
        branch: WorthQueryProductBranch,
        no_effect: NoEffectCompositePublication,
    },
    ProductUnpublished {
        branch: WorthQueryProductBranch,
        adoption: WorthQueryUnpublishedBranchAdoption,
    },
}

impl WorthQueryBranchSetAdoptionProgress {
    pub const fn branch(&self) -> WorthQueryProductBranch {
        match self {
            Self::Performed { branch, .. }
            | Self::NoEffect { branch, .. }
            | Self::ProductUnpublished { branch, .. } => *branch,
        }
    }

    pub const fn performed(&self) -> Option<&WorthQueryPerformedBranchAdoption> {
        match self {
            Self::Performed { adoption, .. } => Some(adoption),
            Self::NoEffect { .. } | Self::ProductUnpublished { .. } => None,
        }
    }

    pub const fn no_effect(&self) -> Option<&NoEffectCompositePublication> {
        match self {
            Self::NoEffect { no_effect, .. } => Some(no_effect),
            Self::Performed { .. } | Self::ProductUnpublished { .. } => None,
        }
    }

    pub const fn unpublished(&self) -> Option<&WorthQueryUnpublishedBranchAdoption> {
        match self {
            Self::ProductUnpublished { adoption, .. } => Some(adoption),
            Self::Performed { .. } | Self::NoEffect { .. } => None,
        }
    }

    pub(super) const fn blocks_advance(&self) -> bool {
        !matches!(self, Self::Performed { .. })
    }

    pub(super) const fn requires_recovery(&self) -> bool {
        matches!(self, Self::ProductUnpublished { .. })
    }
}

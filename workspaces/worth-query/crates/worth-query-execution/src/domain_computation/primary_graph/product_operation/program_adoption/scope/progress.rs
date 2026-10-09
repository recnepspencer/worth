use worth_runtime_world::facade::NoEffectCompositePublication;

use super::super::{WorthQueryPerformedBranchAdoption, WorthQueryUnpublishedBranchAdoption};
use crate::basis::WorthQueryProductBranch;

/// The disposition of one branch already attempted in a branch-set adoption.
pub enum WorthQueryBranchSetAdoptionProgress {
    /// The host call was refused before any owner effects.
    ExecutionDenied {
        branch: WorthQueryProductBranch,
        cause: crate::domain_computation::primary_graph::WorthQueryAdvancementDenial,
    },
    /// The branch's adoption is published.
    Performed {
        branch: WorthQueryProductBranch,
        adoption: WorthQueryPerformedBranchAdoption,
    },
    /// The branch's publication had no effect: nothing was published.
    NoEffect {
        branch: WorthQueryProductBranch,
        no_effect: NoEffectCompositePublication,
    },
    /// Some owners moved, but the branch's product head did not. Recover
    /// publication.
    ProductUnpublished {
        branch: WorthQueryProductBranch,
        adoption: WorthQueryUnpublishedBranchAdoption,
    },
}

impl WorthQueryBranchSetAdoptionProgress {
    pub const fn branch(&self) -> WorthQueryProductBranch {
        match self {
            Self::ExecutionDenied { branch, .. }
            | Self::Performed { branch, .. }
            | Self::NoEffect { branch, .. }
            | Self::ProductUnpublished { branch, .. } => *branch,
        }
    }

    pub const fn performed(&self) -> Option<&WorthQueryPerformedBranchAdoption> {
        match self {
            Self::Performed { adoption, .. } => Some(adoption),
            Self::ExecutionDenied { .. }
            | Self::NoEffect { .. }
            | Self::ProductUnpublished { .. } => None,
        }
    }

    pub const fn no_effect(&self) -> Option<&NoEffectCompositePublication> {
        match self {
            Self::NoEffect { no_effect, .. } => Some(no_effect),
            Self::ExecutionDenied { .. }
            | Self::Performed { .. }
            | Self::ProductUnpublished { .. } => None,
        }
    }

    pub const fn unpublished(&self) -> Option<&WorthQueryUnpublishedBranchAdoption> {
        match self {
            Self::ProductUnpublished { adoption, .. } => Some(adoption),
            Self::ExecutionDenied { .. } | Self::Performed { .. } | Self::NoEffect { .. } => None,
        }
    }
}

//! One actual release family, with its required effective roster.
use super::*;

pub(in crate::physical_runtime) enum RecoveredReleaseCustody {
    HeadV2(VerifiedSelectedReleaseHeadCustodyV2),
    NoRelease(RecoveredNoReleaseCustody),
    PendingWal {
        claim: VerifiedPendingWalReleaseCustody,
        effective: VerifiedEffectiveReleaseHeadRosterV14,
    },
    OrderedHistorical {
        effective: VerifiedEffectiveReleaseHeadRosterV14,
        claim: VerifiedOrderedHistoricalReleaseCustody,
    },
}

#[cfg(feature = "recovery-runtime-owner")]
impl RecoveredReleaseCustody {
    pub(super) fn selected_checkpoint(
        &self,
    ) -> Option<&worth_store_physical_integrity::VerifiedCheckpointFacts> {
        match self {
            Self::HeadV2(claim) => Some(claim.checkpoint()),
            Self::NoRelease(claim) => claim.selected().map(|selected| selected.checkpoint()),
            Self::PendingWal { claim, .. } => Some(claim.checkpoint()),
            Self::OrderedHistorical { claim, .. } => Some(claim.checkpoint()),
        }
    }

    pub(super) const fn is_historical(&self) -> bool {
        match self {
            Self::HeadV2(_) => false,
            Self::NoRelease(_) => false,
            Self::PendingWal { .. } => false,
            Self::OrderedHistorical { .. } => true,
        }
    }
}

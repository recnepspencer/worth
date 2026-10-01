//! Release custody across planning and published-root rebind. Tier custody is
//! orthogonal and remains owned by its independent phase contract.

use worth_store_recovery_physics::{
    VerifiedEffectiveReleaseHeadRosterV14, VerifiedOrderedHistoricalReleaseCustody,
    VerifiedPendingWalReleaseCustody, VerifiedSelectedNoReleaseCustody,
    VerifiedSelectedReleaseHeadCustodyV2, VerifiedSelectedReleaseHeadReplayV14,
};

/// Planning may hold pre-publication pending storage, but cannot present it as
/// a final effective roster before the exact published root is admitted.
pub(crate) enum PlanningCustody {
    Unresolved,
    NoCheckpoint,
    NoRelease(VerifiedSelectedNoReleaseCustody),
    SourceHeads(VerifiedSelectedReleaseHeadCustodyV2),
    PendingPrepared {
        claim: VerifiedPendingWalReleaseCustody,
        replay: VerifiedSelectedReleaseHeadReplayV14,
    },
    OrderedCompleted {
        claim: VerifiedOrderedHistoricalReleaseCustody,
        effective_heads: VerifiedEffectiveReleaseHeadRosterV14,
    },
}

/// Only complete custody states cross into Store's post-reopen construction.
pub(crate) enum CustodyState {
    NoCheckpoint,
    NoRelease(VerifiedSelectedNoReleaseCustody),
    SourceHeads(VerifiedSelectedReleaseHeadCustodyV2),
    Pending {
        claim: VerifiedPendingWalReleaseCustody,
        replay: VerifiedSelectedReleaseHeadReplayV14,
        effective_heads: VerifiedEffectiveReleaseHeadRosterV14,
    },
    OrderedCompleted {
        claim: VerifiedOrderedHistoricalReleaseCustody,
        effective_heads: VerifiedEffectiveReleaseHeadRosterV14,
    },
}

//! Trusted genesis custody, isolated from media: the loaded root decides
//! whether a never-checkpointed Store may open on it, and the retained WAL
//! decides whether the candidate is admitted.
use super::*;
use crate::physical_runtime::durability::{
    CheckpointCustodyOrigin, RetainedWalHistory, RetainedWalReleaseEvidence,
};

fn root(generation: u64, anchor: Option<[u8; 32]>) -> DurablePhysicalRootManifest {
    DurablePhysicalRootManifest::builder(generation, 1, 2, 1)
        .tier_epoch_anchor(anchor)
        .admit()
        .expect("test root")
}

/// Joins the root's genesis candidate to the retained WAL as reopen does.
fn reopen(
    root: &DurablePhysicalRootManifest,
    tier_epoch_start: Option<u64>,
    released_drop: bool,
    history: RetainedWalHistory,
) -> CheckpointCustodyOrigin {
    genesis(root, tier_epoch_start)
        .map_or(
            CheckpointCustodyCandidate::ReopenRequiresC8,
            CheckpointCustodyCandidate::CleanReopen,
        )
        .admit(RetainedWalReleaseEvidence::new(released_drop, history))
}

fn trusted(first_root: bool) -> CheckpointCustodyOrigin {
    CheckpointCustodyOrigin::CleanReopen(CleanReopenCheckpointCustody::TrustedGenesis {
        first_root,
    })
}

#[test]
fn only_the_whole_history_without_a_tier_opens_on_trusted_genesis() {
    use RetainedWalHistory::{Empty, FromOrigin, Suffix};
    let c8 = CheckpointCustodyOrigin::ReopenRequiresC8;
    let first = root(1, None);
    let later = root(5, None);
    assert_eq!(reopen(&first, None, false, Empty), trusted(true));
    assert_eq!(reopen(&first, None, false, FromOrigin), trusted(true));
    assert_eq!(reopen(&later, None, false, FromOrigin), trusted(false));
    // A later root with no retained WAL lost its history.
    assert_eq!(reopen(&later, None, false, Empty), c8);
    assert_eq!(reopen(&later, None, false, Suffix), c8);
    assert_eq!(reopen(&first, None, false, Suffix), c8);
    assert_eq!(reopen(&first, None, true, Empty), c8);
    assert_eq!(reopen(&later, None, true, FromOrigin), c8);
    // A tier anchor or tier start never opens on genesis custody, whatever
    // the retained WAL holds.
    for history in [Empty, FromOrigin] {
        assert_eq!(reopen(&root(1, Some([3; 32])), None, false, history), c8);
        assert_eq!(reopen(&first, Some(1), false, history), c8);
    }
    assert_eq!(genesis(&root(1, Some([3; 32])), None), None);
    assert_eq!(genesis(&first, Some(1)), None);
}

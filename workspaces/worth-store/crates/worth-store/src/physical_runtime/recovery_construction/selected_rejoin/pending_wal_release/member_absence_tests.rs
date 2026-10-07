use worth_store_wal::{LogSequenceNumber, WalLsnRange};

use super::any_member_leaves;
use crate::physical_runtime::recovery_construction::selected_rejoin::SelectedMediaRejoinDenial as Denial;

const CUTOFF: u64 = 16;

fn range(start: u64) -> WalLsnRange {
    WalLsnRange::new(
        LogSequenceNumber::new(start),
        LogSequenceNumber::new(start + 1),
    )
    .unwrap()
}

/// Each fake member's redo byte is the source generation its decode reports.
fn leaves(members: &[(u64, [u8; 1])], source: u64) -> Result<bool, Denial> {
    any_member_leaves(
        members
            .iter()
            .map(|(start, redo)| (range(*start), &redo[..])),
        CUTOFF,
        source,
        |redo, _, bound, limits| {
            assert_eq!(bound, 1);
            assert_eq!(limits.total_entries, 3);
            Ok(u64::from(redo[0]))
        },
    )
}

#[test]
fn only_a_replayed_member_leaving_the_source_counts() {
    assert!(matches!(leaves(&[(17, [15]), (18, [16])], 14), Ok(false)));
    assert!(matches!(leaves(&[(17, [15]), (18, [14])], 14), Ok(true)));
    assert!(matches!(leaves(&[(CUTOFF, [14])], 14), Ok(true)));
    // Checkpoint-covered members are retired history, not replay members.
    assert!(matches!(leaves(&[(CUTOFF - 1, [14])], 14), Ok(false)));
}

#[test]
fn an_unreadable_member_denies_instead_of_proving_absence() {
    let denied = any_member_leaves(
        [(range(17), &[0_u8][..])].into_iter(),
        CUTOFF,
        14,
        |_, _, _, _| Err(Denial::WalFate),
    );
    assert!(matches!(denied, Err(Denial::WalFate)));
}

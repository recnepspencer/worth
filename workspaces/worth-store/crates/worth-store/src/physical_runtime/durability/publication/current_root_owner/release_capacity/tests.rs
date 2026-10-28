use super::{ReleaseLedgerState, SelectedReleaseCustodyLedger};
use crate::physical_runtime::durability::CheckpointCustodyOrigin;

#[test]
fn reopened_ledger_cannot_inherit_trusted_genesis_empty_state() {
    assert!(matches!(
        ReleaseLedgerState::from_origin(CheckpointCustodyOrigin::ReopenRequiresC8),
        ReleaseLedgerState::Unavailable
    ));
    assert!(matches!(
        ReleaseLedgerState::from_origin(CheckpointCustodyOrigin::FreshGenesis),
        ReleaseLedgerState::Selected(_)
    ));
}

#[test]
fn release_preflight_reserves_tier_and_ratchet_space_with_hard_bounds() {
    let ledger = SelectedReleaseCustodyLedger::trusted_genesis();
    assert!(ledger.admits_worst_case(true, 2, 1024));
    assert!(!ledger.admits_worst_case(true, 64, 1024));
    assert!(ledger.admits_worst_case(false, 2, 65_536));
    assert!(!ledger.admits_worst_case(true, 2, 65_536));
    assert!(!ledger.admits_worst_case(false, 1, 1024));
}

#[test]
fn release_preflight_replaces_the_marker_at_the_full_roster_boundary() {
    use worth_store_physical_format::{
        MAX_CHECKPOINT_CERTIFICATE_RECORDS, RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES,
        RELEASE_CHECKPOINT_BATCH_WIRE_BYTES,
    };
    let batch =
        RELEASE_CHECKPOINT_BATCH_WIRE_BYTES as u32 + super::CERTIFICATE_FRAME_OVERHEAD as u32;
    let accumulator = RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES as u32
        + super::CERTIFICATE_FRAME_OVERHEAD as u32;
    for batches in 0..MAX_CHECKPOINT_CERTIFICATE_RECORDS as u16 {
        let mut ledger = SelectedReleaseCustodyLedger::trusted_genesis();
        ledger.used_records = batches + 1;
        ledger.used_bytes = u32::from(batches) * batch + accumulator;
        assert_eq!(
            ledger.admits_worst_case(false, 2, batch + accumulator),
            u64::from(batches) + 2 <= MAX_CHECKPOINT_CERTIFICATE_RECORDS,
            "{batches} existing Batches, one incoming Batch and one Accumulator"
        );
        assert_eq!(
            ledger.admits_worst_case(true, 2, batch + accumulator),
            u64::from(batches) + 3 <= MAX_CHECKPOINT_CERTIFICATE_RECORDS,
            "{batches} existing Batches plus Batch, Accumulator and TierEpoch"
        );
    }
}

#[test]
fn release_preflight_reuses_marker_bytes_without_exceeding_the_byte_limit() {
    use worth_store_physical_format::{
        MAX_CHECKPOINT_CERTIFICATE_BYTES, RELEASE_CHECKPOINT_NO_RELEASE_WIRE_BYTES,
    };
    let mut ledger = SelectedReleaseCustodyLedger::trusted_genesis();
    ledger.used_records = 1;
    ledger.used_bytes =
        RELEASE_CHECKPOINT_NO_RELEASE_WIRE_BYTES as u32 + super::CERTIFICATE_FRAME_OVERHEAD as u32;
    let limit = MAX_CHECKPOINT_CERTIFICATE_BYTES as u32;
    assert!(ledger.admits_worst_case(false, 2, limit));
    assert!(!ledger.admits_worst_case(false, 2, limit + 1));
    assert!(!ledger.admits_worst_case(true, 2, limit));
}

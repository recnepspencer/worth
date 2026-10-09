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

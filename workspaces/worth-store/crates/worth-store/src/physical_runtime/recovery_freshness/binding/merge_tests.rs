use super::{conflicting_terminal_fates, StoreRecoveryOperationFate};

#[test]
fn conflicting_terminal_fates_are_rejected_in_both_arrival_orders() {
    assert!(conflicting_terminal_fates(
        StoreRecoveryOperationFate::AcknowledgedDurable,
        StoreRecoveryOperationFate::ProvenNoEffect,
    ));
    assert!(conflicting_terminal_fates(
        StoreRecoveryOperationFate::ProvenNoEffect,
        StoreRecoveryOperationFate::AcknowledgedDurable,
    ));
}

#[test]
fn an_indeterminate_wal_observation_can_be_replaced_by_one_terminal_fate() {
    assert!(!conflicting_terminal_fates(
        StoreRecoveryOperationFate::Indeterminate,
        StoreRecoveryOperationFate::ProvenNoEffect,
    ));
    assert!(!conflicting_terminal_fates(
        StoreRecoveryOperationFate::Indeterminate,
        StoreRecoveryOperationFate::AcknowledgedDurable,
    ));
}

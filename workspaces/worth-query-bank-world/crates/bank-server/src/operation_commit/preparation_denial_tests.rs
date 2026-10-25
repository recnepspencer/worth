//! The native storage refusal keeps its category across the Bank facade.
use super::*;

#[test]
fn storage_unavailability_keeps_its_bank_preparation_category() {
    assert_eq!(
        application_attempt_kind(WorthQueryApplicationAttemptDenialKind::StorageUnavailable),
        BankApplicationAttemptDenialKind::StorageUnavailable,
    );
}

//! Idempotency refusals distinguish recoverable capacity from lost evidence.
use super::*;

#[test]
fn a_committed_key_asks_for_a_refresh_and_an_unverifiable_one_for_the_operator() {
    use bank_server::BankEstateIdempotencyResolutionDenial as Idempotency;
    assert_eq!(
        estate_denial(BankEstateProgressionDenial::Idempotency(
            Idempotency::CommittedReceiptNotRetained
        )),
        BankHttpDenial::new(Denial::Stale, Next::Refresh)
    );
    assert_eq!(
        estate_denial(BankEstateProgressionDenial::Idempotency(
            Idempotency::RecordedIntentUnverifiable
        )),
        BankHttpDenial::new(Denial::InternalDenied, Next::ContactOperator)
    );
}

#[test]
fn idempotency_capacity_is_retried_and_spent_identity_asks_for_the_operator() {
    use bank_server::BankEstateIdempotencyResolutionDenial as Idempotency;
    for (cause, denial, next) in [
        (
            Idempotency::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots: 2,
            },
            Denial::Unavailable,
            Next::Retry,
        ),
        (
            Idempotency::RetentionCapacityExhausted,
            Denial::Unavailable,
            Next::Retry,
        ),
        (
            Idempotency::ProviderUnavailable,
            Denial::Unavailable,
            Next::Retry,
        ),
        (
            Idempotency::RetentionIdentityExhausted,
            Denial::Unavailable,
            Next::ContactOperator,
        ),
        (
            Idempotency::SnapshotIdentityExhausted,
            Denial::Unavailable,
            Next::ContactOperator,
        ),
        (
            Idempotency::ForeignAdmission,
            Denial::InternalDenied,
            Next::ContactOperator,
        ),
    ] {
        assert_eq!(
            estate_denial(BankEstateProgressionDenial::Idempotency(cause)),
            BankHttpDenial::new(denial, next),
            "{cause:?}"
        );
    }
}

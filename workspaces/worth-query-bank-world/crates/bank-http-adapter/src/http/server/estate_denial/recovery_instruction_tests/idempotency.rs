//! Idempotency refusals distinguish recoverable capacity from lost evidence.
use super::*;

#[test]
fn an_unverifiable_recorded_intent_asks_for_the_operator() {
    use bank_server::BankEstateIdempotencyResolutionDenial as Idempotency;
    // The historical receipt-loss refusal was retired by 98b39a07bd:
    // committed retries carry owner-read original data, never a missing receipt.
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

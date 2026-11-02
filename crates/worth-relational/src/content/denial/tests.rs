use super::{RelationalContentDenial as Denial, RelationalContentRejection as Rejection};
use crate::durability::data::RelationalRecoveryStop;
use crate::execution::RelationalExecutionDenialCause;

#[test]
fn semantic_content_causes_keep_literal_payloads_and_execution_keeps_its_original_cause() {
    let backing = crate::storage::data::RelationalStorageUnavailableCause::LimitExceeded {
        length: 91,
        limit: 13,
    };
    let identity = worth_foundational::facade::ContentRefId([0x5f; 32]);
    for (denial, expected) in [
        (Denial::MissingContent, Rejection::MissingContent),
        (
            Denial::BackingUnavailable {
                identity,
                cause: backing,
            },
            Rejection::BackingUnavailable {
                identity,
                cause: backing,
            },
        ),
        (
            Denial::FormatBindingMissing,
            Rejection::FormatBindingMissing,
        ),
        (Denial::FormatMismatch, Rejection::FormatMismatch),
        (Denial::UnsupportedVersion, Rejection::UnsupportedVersion),
        (Denial::InvalidFormat, Rejection::InvalidFormat),
        (Denial::NonCanonicalContent, Rejection::NonCanonicalContent),
        (Denial::IntegrityMismatch, Rejection::IntegrityMismatch),
        (
            Denial::ContentIdentityConflict,
            Rejection::ContentIdentityConflict,
        ),
        (
            Denial::EncodingInconsistency,
            Rejection::EncodingInconsistency,
        ),
        (Denial::InputTypeMismatch, Rejection::InputTypeMismatch),
        (
            Denial::DuplicateContentFormat,
            Rejection::DuplicateContentFormat,
        ),
        (
            Denial::SizeLimit {
                observed: 97,
                maximum: 31,
            },
            Rejection::SizeLimit {
                observed: 97,
                maximum: 31,
            },
        ),
        (Denial::ArithmeticOverflow, Rejection::ArithmeticOverflow),
    ] {
        let actual = denial.into_execution_cause();
        assert_eq!(actual, RelationalExecutionDenialCause::Content(expected));
        assert_eq!(
            RelationalRecoveryStop::from(actual),
            RelationalRecoveryStop::Content(denial)
        );
    }
    let original = RelationalExecutionDenialCause::PolicyMemoryExhausted {
        requested: 101,
        admitted: 19,
        ancestor: 3,
    };
    assert_eq!(Denial::Execution(original).into_execution_cause(), original);
}

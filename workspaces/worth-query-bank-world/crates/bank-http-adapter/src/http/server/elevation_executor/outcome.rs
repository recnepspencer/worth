use super::super::super::protocol::{
    BankHttpDenial, BankHttpDenialKind, BankHttpElevationApprovalOutcome,
    BankHttpElevationRequestOutcome, BankHttpElevationRevocationOutcome,
    BankHttpMandatoryReviewOutcome, BankHttpNextAction,
};

pub(super) fn request_denied(
    request_id: Option<String>,
    denial: BankHttpDenial,
) -> BankHttpElevationRequestOutcome {
    BankHttpElevationRequestOutcome::Denied { request_id, denial }
}

pub(super) fn approval_denied(
    request_id: Option<String>,
    denial: BankHttpDenial,
) -> BankHttpElevationApprovalOutcome {
    BankHttpElevationApprovalOutcome::Denied { request_id, denial }
}

pub(super) fn revocation_denied(
    request_id: Option<String>,
    denial: BankHttpDenial,
) -> BankHttpElevationRevocationOutcome {
    BankHttpElevationRevocationOutcome::Denied { request_id, denial }
}

pub(super) fn review_denied(
    request_id: Option<String>,
    denial: BankHttpDenial,
) -> BankHttpMandatoryReviewOutcome {
    BankHttpMandatoryReviewOutcome::Denied { request_id, denial }
}

pub(super) const fn malformed() -> BankHttpDenial {
    BankHttpDenial::new(
        BankHttpDenialKind::MalformedRequest,
        BankHttpNextAction::CorrectRequest,
    )
}

pub(super) const fn stale() -> BankHttpDenial {
    BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh)
}

pub(super) const fn idempotency_drift() -> BankHttpDenial {
    BankHttpDenial::new(
        BankHttpDenialKind::Stale,
        BankHttpNextAction::CorrectRequest,
    )
}

pub(super) const fn cancelled() -> BankHttpDenial {
    BankHttpDenial::new(BankHttpDenialKind::Cancelled, BankHttpNextAction::Retry)
}

pub(super) const fn saturated() -> BankHttpDenial {
    BankHttpDenial::new(BankHttpDenialKind::Saturated, BankHttpNextAction::Retry)
}

pub(super) const fn unavailable() -> BankHttpDenial {
    BankHttpDenial::new(BankHttpDenialKind::Unavailable, BankHttpNextAction::Retry)
}

pub(super) const fn indeterminate() -> BankHttpDenial {
    BankHttpDenial::new(
        BankHttpDenialKind::Unavailable,
        BankHttpNextAction::ContactOperator,
    )
}

pub(super) const fn deadline_exceeded() -> BankHttpDenial {
    BankHttpDenial::new(
        BankHttpDenialKind::DeadlineExceeded,
        BankHttpNextAction::Retry,
    )
}

//! Preserve ordinary output failure recovery through the actual HTTP projection.
use super::*;

#[test]
fn ordinary_output_failures_preserve_recovery_actions() {
    for (cause, expected, status) in [
        (
            BankApplicationOutputSettlementDenialKind::RetentionBudgetExceeded,
            BankHttpDenial::new(Kind::ResourceExhausted, Next::NarrowRequest),
            429,
        ),
        (
            BankApplicationOutputSettlementDenialKind::Cancelled,
            BankHttpDenial::new(Kind::Cancelled, Next::Retry),
            408,
        ),
        (
            BankApplicationOutputSettlementDenialKind::RequiredSuccessorStateUnavailable,
            BankHttpDenial::new(Kind::InternalDenied, Next::None),
            500,
        ),
        (
            BankApplicationOutputSettlementDenialKind::ForeignDemand,
            BankHttpDenial::new(Kind::Stale, Next::Refresh),
            409,
        ),
    ] {
        let answer = query_denial(BankApplicationQueryDenial::OutputSettlement(cause));
        assert_eq!(answer, expected);
        assert_eq!(
            super::super::routes::response_status(answer.kind).as_u16(),
            status
        );
    }
    assert_eq!(
        super::super::routes::response_status(Kind::Unavailable),
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(
        super::super::relational_execution_denial::denial(
            worth_query_host::facade::primary_graph::RelationalExecutionDenialCause::AllocationFailed
        ),
        BankHttpDenial::new(Kind::Unavailable, Next::Retry)
    );
}

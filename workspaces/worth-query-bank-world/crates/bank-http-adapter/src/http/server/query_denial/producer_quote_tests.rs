//! Preserve ordinary output failure recovery through the actual HTTP projection.
use super::*;

#[test]
fn ordinary_output_failures_preserve_recovery_actions() {
    for (cause, expected) in [
        (
            BankApplicationOutputSettlementDenialKind::RetentionBudgetExceeded,
            BankHttpDenial::new(Kind::ResourceExhausted, Next::NarrowRequest),
        ),
        (
            BankApplicationOutputSettlementDenialKind::Cancelled,
            BankHttpDenial::new(Kind::Cancelled, Next::Retry),
        ),
    ] {
        assert_eq!(
            query_denial(BankApplicationQueryDenial::OutputSettlement(cause)),
            expected
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

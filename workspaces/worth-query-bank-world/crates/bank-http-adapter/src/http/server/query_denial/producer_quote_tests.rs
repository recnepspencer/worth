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

// Descriptive denial inputs exercise only the Query-to-Bank and HTTP projections.
#[test]
fn required_output_structure_denials_preserve_source_and_terminal_posture() {
    use worth_query_host::facade::{
        home_program_transition::{EntityId, PartitionId},
        primary_graph::WorthQueryOutputDemandDenialKind as Query,
    };
    use BankApplicationOutputSettlementDenialKind as Bank;

    let source = EntityId::new(PartitionId::new(1), 0, 1);
    for (query, expected) in [
        (
            Query::CyclicRequiredOutputOccurrences { source },
            Bank::CyclicRequiredOutputOccurrences { source },
        ),
        (
            Query::RequiredOutputParentUnavailable { source },
            Bank::RequiredOutputParentUnavailable { source },
        ),
    ] {
        let bank = Bank::from(query);
        assert_eq!(bank, expected);
        let answer = query_denial(BankApplicationQueryDenial::OutputSettlement(bank));
        assert_eq!(
            answer,
            BankHttpDenial::new(Kind::InternalDenied, Next::None)
        );
        assert_eq!(
            super::super::routes::response_status(answer.kind),
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}

//! Preserve intrinsic producer-source failures through the actual HTTP projection.
use super::*;
use worth_query_host::facade::application_contribution::WorthQueryProducerResourceQuoteDenial as Quote;

#[test]
fn producer_quote_causes_have_distinct_correctable_wire_values() {
    for (cause, kind, tag) in [
        (
            Quote::SizeOverflow,
            Kind::ProducerSourceSizeOverflow,
            "producer_source_size_overflow",
        ),
        (
            Quote::SourceNotEncodable,
            Kind::ProducerSourceNotEncodable,
            "producer_source_not_encodable",
        ),
    ] {
        let actual = query_denial(BankApplicationQueryDenial::OutputSettlement(
            BankApplicationOutputSettlementDenialKind::ProducerResourceQuote(cause),
        ));
        assert_eq!(actual, BankHttpDenial::new(kind, Next::CorrectRequest));
        assert_eq!(
            super::super::routes::response_status(kind),
            axum::http::StatusCode::BAD_REQUEST
        );
        let wire = serde_json::to_value(actual).unwrap();
        assert_eq!(
            wire,
            serde_json::json!({"kind": tag, "next_action": "correct_request"})
        );
        assert_eq!(
            serde_json::from_value::<BankHttpDenial>(wire).unwrap(),
            actual
        );
    }
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
        super::super::relational_execution_denial::denial(
            worth_query_host::facade::primary_graph::RelationalExecutionDenialCause::AllocationFailed
        ),
        BankHttpDenial::new(Kind::Unavailable, Next::Retry)
    );
}

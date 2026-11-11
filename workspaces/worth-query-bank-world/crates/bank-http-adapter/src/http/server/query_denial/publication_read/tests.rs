//! Pure diagnostic data exercises transport; it supplies no execution or publication authority.
use super::*;
use worth_query_host::facade::primary_graph::{
    CommitExecutionDenial, ErrorContext, ErrorOperation, RelationalContentRejection as Content,
    RelationalExecutionDenialCause as Cause, RelationalSubsystem, WorthQueryOutputDemandDenialKind,
};

#[test]
fn current_publication_carrier_keeps_original_control_and_content_instructions() {
    for (cause, expected, status) in [
        (
            Cause::Cancelled,
            BankHttpDenial::new(Kind::Cancelled, Next::Retry),
            408,
        ),
        (
            Cause::DeadlineElapsed,
            BankHttpDenial::new(Kind::DeadlineExceeded, Next::Retry),
            408,
        ),
        (
            Cause::Content(Content::HistoryNotResident),
            BankHttpDenial::new(Kind::Unavailable, Next::ContactOperator),
            503,
        ),
        (
            Cause::Content(Content::SizeLimit {
                observed: 9,
                maximum: 3,
            }),
            BankHttpDenial::new(Kind::ResourceExhausted, Next::NarrowRequest),
            429,
        ),
        (
            Cause::Content(Content::IntegrityMismatch),
            BankHttpDenial::new(Kind::InternalDenied, Next::ContactOperator),
            500,
        ),
    ] {
        let original = TransactionCommitError::Execution {
            denial: CommitExecutionDenial {
                kind: CommitExecutionDenialKind::Cause(cause),
                partition_identity: Some(7),
            },
            context: ErrorContext::new(RelationalSubsystem::Transaction, ErrorOperation::Commit),
            commit_log: Default::default(),
        };
        let bank: BankApplicationOutputSettlementDenialKind =
            WorthQueryOutputDemandDenialKind::CurrentPublication(Box::new(original.clone())).into();
        assert_eq!(
            bank,
            BankApplicationOutputSettlementDenialKind::CurrentPublication(Box::new(original))
        );
        let answer = query_denial(BankApplicationQueryDenial::OutputSettlement(bank));
        assert_eq!(answer, expected);
        assert_eq!(
            crate::http::server::routes::response_status(answer.kind).as_u16(),
            status
        );
    }
}

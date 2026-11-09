//! Diagnostic projection evidence; these values do not simulate an owner execution.
use super::*;
use worth_query_host::facade::{
    primary_graph::{
        CompanionPreflightStop, RelationalContentDenial as Content,
        RelationalContentRejection as Rejection, RelationalExecutionDenialCause as Cause,
        SourceQueryObservationStop, StoreDenial, WorthQueryAdmittedOneShotStop,
        WorthQueryOutputDemandDenialKind,
    },
    runtime::{MapKernelStop, WorkCeilingDenial},
};

#[test]
fn predecessor_observation_preserves_nested_control_and_content() {
    for (cause, expected) in [
        (
            Cause::Cancelled,
            BankHttpDenial::new(Kind::Cancelled, Next::Retry),
        ),
        (
            Cause::DeadlineElapsed,
            BankHttpDenial::new(Kind::DeadlineExceeded, Next::Retry),
        ),
        (
            Cause::WorkExhausted,
            BankHttpDenial::new(Kind::ResourceExhausted, Next::NarrowRequest),
        ),
        (
            Cause::Content(Rejection::IntegrityMismatch),
            BankHttpDenial::new(Kind::InternalDenied, Next::ContactOperator),
        ),
    ] {
        let original =
            WorthQueryPredecessorReadDenial::Control(Box::new(SourceQueryObservationStop::Read(
                WorthQueryAdmittedOneShotStop::Admission(CompanionPreflightStop::Execution(cause)),
            )));
        let bank: BankApplicationOutputSettlementDenialKind =
            WorthQueryOutputDemandDenialKind::PredecessorRead(Box::new(original.clone())).into();
        assert_eq!(
            bank,
            BankApplicationOutputSettlementDenialKind::PredecessorRead(Box::new(original))
        );
        assert_eq!(
            query_denial(BankApplicationQueryDenial::OutputSettlement(bank)),
            expected
        );
    }
}

#[test]
fn retained_projection_source_preserves_request_and_content_instructions() {
    for (cause, expected) in [
        (
            StoreDenial::ExecutionRequest(WorkCeilingDenial::Stopped(MapKernelStop::Cancelled)),
            BankHttpDenial::new(Kind::Cancelled, Next::Retry),
        ),
        (
            StoreDenial::Content(Content::Execution(Cause::DeadlineElapsed)),
            BankHttpDenial::new(Kind::DeadlineExceeded, Next::Retry),
        ),
        (
            StoreDenial::Content(Content::SizeLimit {
                observed: 9,
                maximum: 3,
            }),
            BankHttpDenial::new(Kind::ResourceExhausted, Next::NarrowRequest),
        ),
        (
            StoreDenial::Content(Content::IntegrityMismatch),
            BankHttpDenial::new(Kind::InternalDenied, Next::ContactOperator),
        ),
    ] {
        assert_eq!(retained_source(&cause), expected);
    }
}

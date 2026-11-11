//! Transport projection only; these diagnostic values do not claim owner execution.
use super::*;
use bank_server::BankApplicationLiveCauseDenial;
use worth_query_host::facade::primary_graph::{
    RelationalContentDenial as Content, RelationalExecutionDenialCause as Cause,
};

#[test]
fn content_stops_in_live_causes_keep_the_request_instruction() {
    for (cause, kind, next) in [
        (
            Cause::Cancelled,
            BankHttpDenialKind::Cancelled,
            BankHttpNextAction::Retry,
        ),
        (
            Cause::DeadlineElapsed,
            BankHttpDenialKind::DeadlineExceeded,
            BankHttpNextAction::Retry,
        ),
        (
            Cause::WorkExhausted,
            BankHttpDenialKind::ResourceExhausted,
            BankHttpNextAction::NarrowRequest,
        ),
    ] {
        let Some(BankHttpAccountActivityEvent::Denied { request_id, denial }) = live_event(
            "content-request",
            BankAccountActivityLiveOutcome::CauseDenied(BankApplicationLiveCauseDenial::Content(
                Content::Execution(cause),
            )),
        ) else {
            panic!("the original content stop must remain a denial event")
        };
        assert_eq!(request_id, "content-request");
        assert_eq!(denial, BankHttpDenial::new(kind, next));
    }
}

//! Literal receiving vocabulary is independent of the conversion under test.
use super::*;
use worth_relational::facade::{
    content::RelationalContentDenial as Content,
    transactions::RelationalExecutionDenialCause as Native,
};
use {
    crate::domain_computation::vocabulary::provider_session::protocol_denial::WorthQueryProviderSessionDenialKind as Session,
    crate::domain_computation::vocabulary::request_execution::execution_refusal::WorthQueryManagedComputationResourceDenial as Resource,
    crate::domain_computation::vocabulary::request_execution::execution_refusal::WorthQueryMemoryLimitLevel as Level,
};
#[test]
fn content_read_keeps_original_execution_stops_out_of_semantic_content_refusal() {
    use WorthQueryApplicationReadExecutionDenialKind as Kind;
    for (cause, expected) in [
        (
            Native::Content(
                worth_relational::facade::content::RelationalContentRejection::SizeLimit {
                    observed: 103,
                    maximum: 29,
                },
            ),
            WorthQueryApplicationReadExecutionDenialKind::ExecutionRequest(Session::Content(
                worth_relational::facade::content::RelationalContentRejection::SizeLimit {
                    observed: 103,
                    maximum: 29,
                },
            )),
        ),
        (Native::Cancelled, Kind::Cancelled),
        (Native::DeadlineElapsed, Kind::DeadlineExceeded),
        (
            Native::WorkExhausted,
            WorthQueryApplicationReadExecutionDenialKind::ExecutionRequest(
                Session::ExecutionResource {
                    denial: Resource::WorkExhausted,
                    partition_identity: None,
                    policy_ancestor: None,
                },
            ),
        ),
        (
            Native::DeclaredMemoryExhausted {
                requested: 43,
                admitted: 17,
            },
            WorthQueryApplicationReadExecutionDenialKind::ExecutionRequest(
                Session::ExecutionResource {
                    denial: Resource::MemoryLimit {
                        requested: 43,
                        admitted: 17,
                        level: Level::Declared,
                    },
                    partition_identity: None,
                    policy_ancestor: None,
                },
            ),
        ),
        (
            Native::ContentInconsistency,
            WorthQueryApplicationReadExecutionDenialKind::ExecutionRequest(
                Session::ContentInconsistency,
            ),
        ),
    ] {
        assert_eq!(
            content_read_denial(Content::Execution(cause), "field").kind(),
            expected
        );
    }
    for cause in [
        Content::MissingContent,
        Content::FormatMismatch,
        Content::UnsupportedVersion,
        Content::IntegrityMismatch,
    ] {
        assert_eq!(
            content_read_denial(cause, "field").kind(),
            Kind::Content(cause)
        );
    }
}

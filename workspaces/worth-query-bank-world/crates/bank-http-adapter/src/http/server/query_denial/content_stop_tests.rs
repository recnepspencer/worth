//! Diagnostic values prove transport projection, not execution of an owner fault.
#[test]
fn history_recipe_and_ownership_keep_original_control_and_content_instructions() {
    use worth_query_host::facade::{
        application_entry::{
            WorthQueryDocumentHistoryDenial as Document,
            WorthQueryHistoryNavigationDenial as Navigation,
        },
        primary_graph::{
            RelationalHistoricalSelectionReadDenial as Restored,
            RelationalModelOwnershipDenial as Ownership,
        },
    };
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
            Cause::Content(Rejection::SizeLimit {
                observed: 9,
                maximum: 3,
            }),
            BankHttpDenial::new(Kind::ResourceExhausted, Next::NarrowRequest),
        ),
        (
            Cause::Content(Rejection::IntegrityMismatch),
            BankHttpDenial::new(Kind::InternalDenied, Next::ContactOperator),
        ),
    ] {
        for read in [
            Read::RecipeCapture(cause.into()),
            Read::Authority(Navigation::Document(Document::Ownership(
                Ownership::Native(cause),
            ))),
            Read::Authority(Navigation::Document(Document::Recording(cause))),
            Read::Authority(Navigation::OutputLineage(
                CompanionPreflightStop::Execution(cause),
            )),
            Read::Authority(Navigation::RestoredOutputRead(Restored::Execution(cause))),
        ] {
            assert_eq!(
                query_denial(BankApplicationQueryDenial::HistoryOutput(Box::new(read))),
                expected
            );
        }
    }
}
use super::*;
use bank_server::{BankApplicationContinuationDenialKind, BankApplicationLiveOpenDenialKind};
use worth_query_host::facade::{
    application_entry::WorthQueryHistoryOutputReadDenial as Read,
    primary_graph::{
        BoundedEntityFieldLookupDenialKind as Lookup, CompanionPreflightStop,
        IndexedSelectionReobserveDenial as Indexed, RebaseVerificationReason as Verification,
        RelationalContentDenial as Content, RelationalContentRejection as Rejection,
        RelationalExecutionDenialCause as Cause,
    },
};

#[test]
fn content_wrappers_preserve_the_original_request_instruction() {
    for (cause, kind, next) in [
        (Cause::Cancelled, Kind::Cancelled, Next::Retry),
        (Cause::DeadlineElapsed, Kind::DeadlineExceeded, Next::Retry),
        (
            Cause::WorkExhausted,
            Kind::ResourceExhausted,
            Next::NarrowRequest,
        ),
    ] {
        let expected = BankHttpDenial::new(kind, next);
        assert_eq!(
            execution(BankApplicationOneShotDenialKind::Content(
                Content::Execution(cause)
            )),
            expected
        );
        assert_eq!(
            retained_execution::continuation(BankApplicationContinuationDenialKind::Content(
                Content::Execution(cause)
            )),
            expected
        );
        assert_eq!(
            retained_execution::live(BankApplicationLiveOpenDenialKind::Content(
                Content::Execution(cause)
            )),
            expected
        );
        assert_eq!(history_output::denial(Read::Preparation(cause)), expected);
        assert_eq!(
            history_output::denial(Read::Resource(CompanionPreflightStop::Execution(cause))),
            expected
        );
        for verification in [
            Verification::Execution(cause),
            Verification::AdmissionDenied(CompanionPreflightStop::Execution(cause)),
            Verification::IndexedSelectionDenied(Indexed::Lookup(Lookup::Execution(cause))),
            Verification::IndexedSelectionFactDenied(Indexed::Lookup(Lookup::Execution(cause))),
        ] {
            assert_eq!(
                output_settlement::output_settlement(
                    BankApplicationOutputSettlementDenialKind::RetainedVerificationFactUnavailable(
                        Box::new(verification)
                    )
                ),
                expected
            );
        }
    }
}

#[test]
fn content_limits_and_integrity_refusals_require_different_actions() {
    use worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind as Session;
    for (cause, kind, next) in [
        (
            Rejection::SizeLimit {
                observed: 9,
                maximum: 3,
            },
            Kind::ResourceExhausted,
            Next::NarrowRequest,
        ),
        (
            Rejection::IntegrityMismatch,
            Kind::InternalDenied,
            Next::ContactOperator,
        ),
    ] {
        let expected = BankHttpDenial::new(kind, next);
        assert_eq!(
            crate::http::server::mutation_application::pending_execution_denial(Session::Content(
                cause
            )),
            expected
        );
        assert_eq!(
            crate::http::server::relational_execution_denial::denial(Cause::Content(cause)),
            expected
        );
    }
}

#[test]
fn current_read_cutover_deadlines_keep_the_request_instruction() {
    use worth_query_host::facade::primary_graph::{
        CurrentProductReadDependencyDenial as Current, NoEffectCause,
    };
    let expected = BankHttpDenial::new(Kind::DeadlineExceeded, Next::Retry);
    assert_eq!(
        crate::http::server::world_denial::publication(NoEffectCause::CurrentReadDependency(
            Current::CutoverDeadlineElapsed
        )),
        expected
    );
    assert_eq!(
        output_settlement::output_settlement(
            BankApplicationOutputSettlementDenialKind::RecoveryPublication(
                NoEffectCause::CurrentReadDependency(Current::CutoverDeadlineElapsed)
            )
        ),
        expected
    );
}

#[test]
fn history_authority_reads_preserve_native_interruption() {
    use worth_query_host::facade::{
        application_entry::{
            WorthQueryDocumentHistoryDenial as Document,
            WorthQueryHistoryNavigationDenial as Navigation,
        },
        primary_graph::RelationalBranchBasisDenial as Basis,
    };
    for (cause, kind) in [
        (Basis::Cancelled, Kind::Cancelled),
        (Basis::TimedOut, Kind::DeadlineExceeded),
    ] {
        assert_eq!(
            history_output::denial(Read::Authority(Navigation::Document(
                Document::NativeBasis(cause)
            ))),
            BankHttpDenial::new(kind, Next::Retry)
        );
    }
}

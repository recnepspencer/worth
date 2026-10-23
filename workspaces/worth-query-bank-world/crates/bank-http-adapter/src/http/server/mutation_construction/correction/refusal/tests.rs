//! Nested owner controls retain their HTTP action through correction projection.
use super::*;
use worth_query_host::facade::primary_graph::{
    WorthQueryAftermathCausalityReadDenial as History,
    WorthQueryProviderSessionControlStopKind as Control,
    WorthQueryProviderSessionDenialKind as Session,
};
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    runtime::MapKernelStop,
};

#[test]
fn correction_stops_survive_source_ancestry_derivation_and_capture() {
    for (cancel, expected) in [(true, Kind::Cancelled), (false, Kind::DeadlineExceeded)] {
        let branch = if cancel {
            RuntimeWorldBranchAdmissionDenial::CancelledBeforeEffect
        } else {
            RuntimeWorldBranchAdmissionDenial::DeadlineBeforeEffect
        };
        let catalog = if cancel {
            CompositeHistoryCatalogDenial::Cancelled
        } else {
            CompositeHistoryCatalogDenial::DeadlineElapsed
        };
        let kernel = if cancel {
            MapKernelStop::Cancelled
        } else {
            MapKernelStop::DeadlineElapsed
        };
        let interruption = if cancel {
            WorthQueryManagedComputationInterruption::Cancelled
        } else {
            WorthQueryManagedComputationInterruption::DeadlineExceeded
        };
        let control = if cancel {
            Control::Cancelled
        } else {
            Control::TimedOut
        };
        for cause in [
            WorthQueryApplicationCorrectionDenial::Source(
                WorthQueryCorrectionSelectionDenial::Ancestry(RuntimeWorldServiceDenial::Denied(
                    ProductBranchAncestorSelectionDenial::Branch(branch),
                )),
            ),
            WorthQueryApplicationCorrectionDenial::Source(
                WorthQueryCorrectionSelectionDenial::Disclosure(
                    WorthQueryCorrectionDerivationDenial::Ancestry(
                        RuntimeWorldServiceDenial::Denied(
                            ProductBranchAncestorSelectionDenial::History(
                                CompositeAncestorSelectionDenial::Catalog(catalog),
                            ),
                        ),
                    ),
                ),
            ),
            WorthQueryApplicationCorrectionDenial::Derivation(
                WorthQueryCorrectionDerivationDenial::SourceOutput(
                    WorthQueryCorrectionOutputProjectionDenial::Stopped(kernel),
                ),
            ),
            WorthQueryApplicationCorrectionDenial::Derivation(
                WorthQueryCorrectionDerivationDenial::InputCapture(
                    WorthQueryOperationInputCaptureDenial::Execution(
                        WorthQueryAdvancementDenial::Interrupted(interruption),
                    ),
                ),
            ),
            WorthQueryApplicationCorrectionDenial::Derivation(
                WorthQueryCorrectionDerivationDenial::History(History::ExecutionDenied(Err(
                    control,
                ))),
            ),
            WorthQueryApplicationCorrectionDenial::Derivation(
                WorthQueryCorrectionDerivationDenial::Selection(Box::new(
                    WorthQueryCorrectionSelectionDenial::Disclosure(
                        WorthQueryCorrectionDerivationDenial::History(History::ExecutionDenied(
                            Err(control),
                        )),
                    ),
                )),
            ),
        ] {
            assert_eq!(
                correction_denial(cause),
                BankHttpDenial::new(expected, Next::Retry)
            );
        }
    }
}

#[test]
fn correction_history_refusals_preserve_authority_applicability_and_support_remedies() {
    use WorthQueryCorrectionDerivationDenial as D;
    for (cause, kind, next) in [
        (D::ForeignAlternative, Kind::PermissionDenied, Next::None),
        (
            D::Selection(Box::new(
                WorthQueryCorrectionSelectionDenial::SourceUnavailable,
            )),
            Kind::PermissionDenied,
            Next::None,
        ),
        (D::SourceAlreadyReversed, Kind::Stale, Next::Refresh),
        (D::NoActiveReversal, Kind::Stale, Next::Refresh),
        (
            D::AlternativeResolutionIncomplete,
            Kind::Stale,
            Next::Refresh,
        ),
        (D::ActiveReversalConsumed, Kind::Stale, Next::Refresh),
        (
            D::History(History::ForeignContinuation),
            Kind::Stale,
            Next::Refresh,
        ),
        (D::ReapplicationUnavailable, Kind::Unavailable, Next::Retry),
        (
            D::History(History::Unavailable),
            Kind::Unavailable,
            Next::Retry,
        ),
        (
            D::History(History::RetentionCapacityExhausted),
            Kind::Unavailable,
            Next::Retry,
        ),
        (
            D::History(History::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots: 1,
            }),
            Kind::Unavailable,
            Next::Retry,
        ),
        (
            D::History(History::RetentionIdentityExhausted),
            Kind::InternalDenied,
            Next::ContactOperator,
        ),
        (
            D::History(History::SnapshotIdentityExhausted),
            Kind::InternalDenied,
            Next::ContactOperator,
        ),
        (
            D::History(History::InconsistentCanonicalRelationships),
            Kind::InternalDenied,
            Next::ContactOperator,
        ),
    ] {
        assert_eq!(derivation(cause), BankHttpDenial::new(kind, next));
    }
}

#[test]
fn correction_history_execution_resources_keep_the_native_actors_remedy() {
    use worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
    for (resource, kind, next) in [
        (
            Resource::WorkLimit,
            Kind::MalformedRequest,
            Next::CorrectRequest,
        ),
        (
            Resource::WorkerLimit,
            Kind::InternalDenied,
            Next::ContactOperator,
        ),
    ] {
        let cause = WorthQueryCorrectionDerivationDenial::History(History::ExecutionDenied(Ok(
            Session::ExecutionResource {
                denial: resource,
                partition_identity: None,
                policy_ancestor: None,
            },
        )));
        assert_eq!(derivation(cause), BankHttpDenial::new(kind, next));
    }
}

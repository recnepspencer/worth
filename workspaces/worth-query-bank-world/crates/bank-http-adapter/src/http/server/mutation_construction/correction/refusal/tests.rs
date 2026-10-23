//! Nested owner controls retain their HTTP action through correction projection.
use super::*;
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
        ] {
            assert_eq!(
                correction_denial(cause),
                BankHttpDenial::new(expected, Next::Retry)
            );
        }
    }
}

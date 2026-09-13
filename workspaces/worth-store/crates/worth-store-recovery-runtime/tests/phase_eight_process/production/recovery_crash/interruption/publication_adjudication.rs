use std::path::Path;

use worth_store_offline_verifier::RecoveryObserverReport;
use worth_store_recovery_runtime::{RecoveryReportEnvelope, RecoveryReportOutcome};

use super::super::super::super::{comparison, history};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RawPublicationPosture {
    ProvenNoEffect,
    PhysicalEffectObserved,
}

pub(super) fn adjudicate(
    root: &Path,
    report: &RecoveryReportEnvelope,
    before: &history::ParentPhysicalHistory,
    after: &history::ParentPhysicalHistory,
    observer: &RecoveryObserverReport,
    label: &str,
) -> RawPublicationPosture {
    let posture = if after.publication_changed_from(before) {
        RawPublicationPosture::PhysicalEffectObserved
    } else {
        RawPublicationPosture::ProvenNoEffect
    };
    match posture {
        RawPublicationPosture::ProvenNoEffect => {
            assert_eq!(
                report.outcome(),
                RecoveryReportOutcome::Blocked,
                "{label} raw publication state did not change"
            );
        }
        RawPublicationPosture::PhysicalEffectObserved => {
            assert_eq!(
                report.outcome(),
                RecoveryReportOutcome::PublicationIndeterminate,
                "{label} raw publication state changed; paths={:?}",
                after.changed_paths_from(before)
            );
            assert!(
                report.counters().recovery_effects() > 0,
                "{label} changed parent history requires a performed recovery effect"
            );
        }
    }
    comparison::compare_runtime_and_observer(report, observer, after).unwrap_or_else(|error| {
        panic!(
            "{label} publication adjudication disagreed with runtime, observer, or raw parent history at {}: {error:?}",
            root.display()
        )
    });
    posture
}

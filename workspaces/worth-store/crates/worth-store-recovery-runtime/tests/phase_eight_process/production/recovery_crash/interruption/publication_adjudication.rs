use std::path::Path;

use worth_store_offline_verifier::RecoveryObserverReport;
use worth_store_recovery_runtime::{
    RecoveryReportDenialCause, RecoveryReportEnvelope, RecoveryReportOutcome,
};

use super::super::super::super::{comparison, history};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RawPublicationPosture {
    NoDurablePublicationObserved,
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
    let changed_paths = after.changed_paths_from(before);
    let selected_publication_changed = after.publication_changed_from(before);
    let candidate_changed = after.root_candidate_materialized_from(before);
    let posture = if selected_publication_changed || candidate_changed {
        RawPublicationPosture::PhysicalEffectObserved
    } else {
        RawPublicationPosture::NoDurablePublicationObserved
    };
    match posture {
        RawPublicationPosture::NoDurablePublicationObserved => {
            assert_eq!(
                report.outcome(),
                RecoveryReportOutcome::Blocked,
                "{label} had no candidate or selected publication change; changed_paths={changed_paths:?} effects={} cause={:?}",
                report.counters().recovery_effects(),
                report.denial_cause()
            );
        }
        RawPublicationPosture::PhysicalEffectObserved => {
            if candidate_changed && !selected_publication_changed {
                assert_eq!(
                    after.current_root_generation(),
                    before.current_root_generation(),
                    "{label} candidate-only effect must not alter selected root generation"
                );
            }
            assert_eq!(
                report.outcome(),
                RecoveryReportOutcome::PublicationIndeterminate,
                "{label} candidate or selected publication changed; paths={changed_paths:?}"
            );
            assert_eq!(
                report.denial_cause(),
                Some(RecoveryReportDenialCause::PublicationSettlementIndeterminate),
                "{label} physical publication effect requires typed indeterminate settlement"
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

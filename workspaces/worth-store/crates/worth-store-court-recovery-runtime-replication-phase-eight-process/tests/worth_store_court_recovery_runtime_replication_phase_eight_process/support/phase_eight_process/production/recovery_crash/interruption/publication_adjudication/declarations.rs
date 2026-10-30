pub(crate) use std::path::Path;

pub(crate) use worth_store_offline_verifier::RecoveryObserverReport;
pub(crate) use {
    worth_store_recovery_runtime::RecoveryReportDenialCause,
    worth_store_recovery_runtime::RecoveryReportEnvelope,
    worth_store_recovery_runtime::RecoveryReportOutcome,
};

pub(crate) use {crate::history, crate::phase_eight_process::comparison};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RawPublicationPosture {
    NoDurablePublicationObserved,
    PhysicalEffectObserved,
}

pub(crate) fn adjudicate(
    root: &Path,
    report: &RecoveryReportEnvelope,
    before: &crate::phase_eight_process::history::physical_history::ParentPhysicalHistory,
    after: &crate::phase_eight_process::history::physical_history::ParentPhysicalHistory,
    observer: &RecoveryObserverReport,
    label: &str,
) -> RawPublicationPosture {
    let changed_paths = after.changed_paths_from(before);
    let selected_publication_changed = after.publication_changed_from(before);
    let candidate_changed = after.root_candidate_materialized_from(before);
    let posture = if selected_publication_changed || candidate_changed {
        crate::phase_eight_process::production::recovery_crash::interruption::publication_adjudication::RawPublicationPosture::PhysicalEffectObserved
    } else {
        crate::phase_eight_process::production::recovery_crash::interruption::publication_adjudication::RawPublicationPosture::NoDurablePublicationObserved
    };
    match posture {
        crate::phase_eight_process::production::recovery_crash::interruption::publication_adjudication::RawPublicationPosture::NoDurablePublicationObserved => {
            assert_eq!(
                report.outcome(),
                worth_store_recovery_runtime::RecoveryReportOutcome::Blocked,
                "{label} had no candidate or selected publication change; changed_paths={changed_paths:?} effects={} cause={:?}",
                report.counters().recovery_effects(),
                report.denial_cause()
            );
        }
        crate::phase_eight_process::production::recovery_crash::interruption::publication_adjudication::RawPublicationPosture::PhysicalEffectObserved => {
            if candidate_changed && !selected_publication_changed {
                assert_eq!(
                    after.current_root_generation(),
                    before.current_root_generation(),
                    "{label} candidate-only effect must not alter selected root generation"
                );
            }
            assert_eq!(
                report.outcome(),
                worth_store_recovery_runtime::RecoveryReportOutcome::PublicationIndeterminate,
                "{label} candidate or selected publication changed; paths={changed_paths:?}"
            );
            assert_eq!(
                report.denial_cause(),
                Some(worth_store_recovery_runtime::RecoveryReportDenialCause::PublicationSettlementIndeterminate),
                "{label} physical publication effect requires typed indeterminate settlement"
            );
            assert!(
                report.counters().recovery_effects() > 0,
                "{label} changed parent history requires a performed recovery effect"
            );
        }
    }
    crate::phase_eight_process::comparison::compare_runtime_and_observer(report, observer, after).unwrap_or_else(|error| {
        panic!(
            "{label} publication adjudication disagreed with runtime, observer, or raw parent history at {}: {error:?}",
            root.display()
        )
    });
    posture
}

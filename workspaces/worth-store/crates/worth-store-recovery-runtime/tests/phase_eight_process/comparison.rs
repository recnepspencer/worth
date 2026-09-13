use worth_store_offline_verifier::RecoveryObserverReport;
use worth_store_recovery_runtime::{
    RecoveryReportDenialCause, RecoveryReportEnvelope, RecoveryReportOutcome,
};

use super::history::ParentPhysicalHistory;

#[path = "comparison/evidence.rs"]
mod evidence;
const C8_RECOVERY_MEMORY_BUDGET_BYTES: u64 = 512 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RecoveryObserverDisagreement {
    RecoveredWithoutArtifacts,
    ParentHistoryMismatch,
    StoreIdentityMismatch,
    RootGenerationMismatch,
    RuntimeCounterMismatch,
    DenialCauseMismatch,
    ObserverEvidenceMismatch,
}

pub(super) fn compare_runtime_and_observer(
    runtime: &RecoveryReportEnvelope,
    observer: &RecoveryObserverReport,
    expected_history: &ParentPhysicalHistory,
) -> Result<(), RecoveryObserverDisagreement> {
    compare_runtime_and_observer_with_budget(
        runtime,
        observer,
        expected_history,
        C8_RECOVERY_MEMORY_BUDGET_BYTES,
    )
}

pub(super) fn compare_runtime_and_observer_with_budget(
    runtime: &RecoveryReportEnvelope,
    observer: &RecoveryObserverReport,
    expected_history: &ParentPhysicalHistory,
    admitted_memory_budget: u64,
) -> Result<(), RecoveryObserverDisagreement> {
    expected_history
        .compare_report(observer)
        .map_err(|_| RecoveryObserverDisagreement::ParentHistoryMismatch)?;
    if runtime.outcome() == RecoveryReportOutcome::Recovered
        && observer.selector_store_identity() != runtime.store_identity()
    {
        return Err(RecoveryObserverDisagreement::StoreIdentityMismatch);
    }
    if runtime.outcome() == RecoveryReportOutcome::Recovered
        && observer.current_root_generation() != runtime.root_generation()
    {
        return Err(RecoveryObserverDisagreement::RootGenerationMismatch);
    }
    match runtime.outcome() {
        RecoveryReportOutcome::Refused => {
            if !matches!(
                runtime.denial_cause(),
                Some(RecoveryReportDenialCause::Refused(_))
            ) || runtime.store_identity().is_some()
                || runtime.root_generation().is_some()
                || runtime.counters().recovery_effects() != 0
            {
                return Err(RecoveryObserverDisagreement::DenialCauseMismatch);
            }
            return Ok(());
        }
        RecoveryReportOutcome::Blocked => {
            if !matches!(
                runtime.denial_cause(),
                Some(RecoveryReportDenialCause::Blocked(_))
            ) || runtime.store_identity().is_none()
            {
                return Err(RecoveryObserverDisagreement::DenialCauseMismatch);
            }
            if observer.selector_store_identity() != runtime.store_identity() {
                return Err(RecoveryObserverDisagreement::StoreIdentityMismatch);
            }
            if let Some(root_generation) = runtime.root_generation() {
                if observer.current_root_generation() != Some(root_generation) {
                    return Err(RecoveryObserverDisagreement::RootGenerationMismatch);
                }
            }
            if runtime.counters().cleanup_performed() != 0
                || runtime.counters().cleanup_deferred() != 0
            {
                return Err(RecoveryObserverDisagreement::RuntimeCounterMismatch);
            }
            return Ok(());
        }
        RecoveryReportOutcome::PublicationIndeterminate => {
            if runtime.denial_cause()
                != Some(RecoveryReportDenialCause::PublicationSettlementIndeterminate)
                || runtime.store_identity().is_none()
                || runtime.root_generation().is_some()
                || runtime.counters().recovery_effects() == 0
            {
                return Err(RecoveryObserverDisagreement::DenialCauseMismatch);
            }
            if observer.selector_store_identity() != runtime.store_identity() {
                return Err(RecoveryObserverDisagreement::StoreIdentityMismatch);
            }
            return Ok(());
        }
        RecoveryReportOutcome::Recovered => {}
    }
    if runtime.counters().peak_recovery_bytes() == 0
        || runtime.counters().peak_recovery_bytes() >= admitted_memory_budget
    {
        return Err(RecoveryObserverDisagreement::RuntimeCounterMismatch);
    }
    if observer.artifact_count() == 0 {
        return Err(RecoveryObserverDisagreement::RecoveredWithoutArtifacts);
    }
    if observer.bytes_read() == 0 || observer.artifact_set_digest() == [0; 32] {
        return Err(RecoveryObserverDisagreement::RecoveredWithoutArtifacts);
    }
    Ok(())
}

pub(super) fn compare_independent_observers(
    expected: &RecoveryObserverReport,
    observed: &RecoveryObserverReport,
) -> Result<(), RecoveryObserverDisagreement> {
    if expected != observed {
        return Err(RecoveryObserverDisagreement::ObserverEvidenceMismatch);
    }
    Ok(())
}

pub(super) fn compare_independent_physical_evidence(
    expected: &RecoveryObserverReport,
    observed: &RecoveryObserverReport,
) -> Result<(), RecoveryObserverDisagreement> {
    if !evidence::same_physical_evidence(expected, observed) {
        return Err(RecoveryObserverDisagreement::ObserverEvidenceMismatch);
    }
    Ok(())
}

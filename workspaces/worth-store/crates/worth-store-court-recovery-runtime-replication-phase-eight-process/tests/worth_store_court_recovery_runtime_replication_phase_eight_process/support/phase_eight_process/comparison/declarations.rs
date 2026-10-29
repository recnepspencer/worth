pub(crate) use {worth_store_offline_verifier::RecoveryObserverReport};
pub(crate) use {worth_store_recovery_runtime::RecoveryReportDenialCause, worth_store_recovery_runtime::RecoveryReportEnvelope, worth_store_recovery_runtime::RecoveryReportOutcome};

pub(crate) use {crate::phase_eight_process::history::physical_history::ParentHistoryMismatch, crate::phase_eight_process::history::physical_history::ParentPhysicalHistory};


pub(crate) const C8_RECOVERY_MEMORY_BUDGET_BYTES: u64 = 512 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecoveryObserverDisagreement {
    RecoveredWithoutArtifacts,
    ParentHistoryMismatch(ParentHistoryMismatch),
    StoreIdentityMismatch,
    RootGenerationMismatch,
    RuntimeCounterMismatch,
    DenialCauseMismatch,
    ObserverEvidenceMismatch,
}

pub(crate) fn compare_runtime_and_observer(
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

pub(crate) fn compare_runtime_and_observer_with_budget(
    runtime: &RecoveryReportEnvelope,
    observer: &RecoveryObserverReport,
    expected_history: &ParentPhysicalHistory,
    admitted_memory_budget: u64,
) -> Result<(), RecoveryObserverDisagreement> {
    expected_history
        .compare_report(observer)
        .map_err(crate::phase_eight_process::comparison::RecoveryObserverDisagreement::ParentHistoryMismatch)?;
    if runtime.outcome() == worth_store_recovery_runtime::RecoveryReportOutcome::Recovered
        && observer.selector_store_identity() != runtime.store_identity()
    {
        return Err(crate::phase_eight_process::comparison::RecoveryObserverDisagreement::StoreIdentityMismatch);
    }
    if runtime.outcome() == worth_store_recovery_runtime::RecoveryReportOutcome::Recovered
        && observer.current_root_generation() != runtime.root_generation()
    {
        return Err(crate::phase_eight_process::comparison::RecoveryObserverDisagreement::RootGenerationMismatch);
    }
    match runtime.outcome() {
        worth_store_recovery_runtime::RecoveryReportOutcome::Refused => {
            if !matches!(
                runtime.denial_cause(),
                Some(worth_store_recovery_runtime::RecoveryReportDenialCause::Refused(_))
            ) || runtime.store_identity().is_some()
                || runtime.root_generation().is_some()
                || runtime.counters().recovery_effects() != 0
            {
                return Err(crate::phase_eight_process::comparison::RecoveryObserverDisagreement::DenialCauseMismatch);
            }
            return Ok(());
        }
        worth_store_recovery_runtime::RecoveryReportOutcome::Blocked => {
            if !matches!(
                runtime.denial_cause(),
                Some(worth_store_recovery_runtime::RecoveryReportDenialCause::Blocked(_))
            ) || runtime.store_identity().is_none()
            {
                return Err(crate::phase_eight_process::comparison::RecoveryObserverDisagreement::DenialCauseMismatch);
            }
            if observer.selector_store_identity() != runtime.store_identity() {
                return Err(crate::phase_eight_process::comparison::RecoveryObserverDisagreement::StoreIdentityMismatch);
            }
            if let Some(root_generation) = runtime.root_generation()
                && observer.current_root_generation() != Some(root_generation)
            {
                return Err(crate::phase_eight_process::comparison::RecoveryObserverDisagreement::RootGenerationMismatch);
            }
            if runtime.counters().cleanup_performed() != 0
                || runtime.counters().cleanup_deferred() != 0
            {
                return Err(crate::phase_eight_process::comparison::RecoveryObserverDisagreement::RuntimeCounterMismatch);
            }
            return Ok(());
        }
        worth_store_recovery_runtime::RecoveryReportOutcome::PublicationIndeterminate => {
            if runtime.denial_cause()
                != Some(worth_store_recovery_runtime::RecoveryReportDenialCause::PublicationSettlementIndeterminate)
                || runtime.store_identity().is_none()
                || runtime.root_generation().is_some()
                || runtime.counters().recovery_effects() == 0
            {
                return Err(crate::phase_eight_process::comparison::RecoveryObserverDisagreement::DenialCauseMismatch);
            }
            if observer.selector_store_identity() != runtime.store_identity() {
                return Err(crate::phase_eight_process::comparison::RecoveryObserverDisagreement::StoreIdentityMismatch);
            }
            return Ok(());
        }
        worth_store_recovery_runtime::RecoveryReportOutcome::Recovered => {}
    }
    if runtime.counters().peak_recovery_bytes() == 0
        || runtime.counters().peak_recovery_bytes() >= admitted_memory_budget
    {
        return Err(crate::phase_eight_process::comparison::RecoveryObserverDisagreement::RuntimeCounterMismatch);
    }
    if observer.artifact_count() == 0 {
        return Err(crate::phase_eight_process::comparison::RecoveryObserverDisagreement::RecoveredWithoutArtifacts);
    }
    if observer.bytes_read() == 0 || observer.artifact_set_digest() == [0; 32] {
        return Err(crate::phase_eight_process::comparison::RecoveryObserverDisagreement::RecoveredWithoutArtifacts);
    }
    Ok(())
}

pub(crate) fn compare_independent_observers(
    expected: &RecoveryObserverReport,
    observed: &RecoveryObserverReport,
) -> Result<(), RecoveryObserverDisagreement> {
    if expected != observed {
        return Err(crate::phase_eight_process::comparison::RecoveryObserverDisagreement::ObserverEvidenceMismatch);
    }
    Ok(())
}

pub(crate) fn compare_independent_physical_evidence(
    expected: &RecoveryObserverReport,
    observed: &RecoveryObserverReport,
) -> Result<(), RecoveryObserverDisagreement> {
    if !crate::comparison::evidence::same_physical_evidence(expected, observed) {
        return Err(crate::phase_eight_process::comparison::RecoveryObserverDisagreement::ObserverEvidenceMismatch);
    }
    Ok(())
}
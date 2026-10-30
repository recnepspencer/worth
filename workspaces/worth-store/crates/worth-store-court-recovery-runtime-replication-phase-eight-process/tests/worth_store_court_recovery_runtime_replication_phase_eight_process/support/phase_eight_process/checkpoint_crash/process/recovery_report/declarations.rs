pub(crate) use {
    worth_store_recovery_runtime::RecoveryReportDenialCause,
    worth_store_recovery_runtime::RecoveryReportEnvelope,
    worth_store_recovery_runtime::RecoveryReportOutcome,
};

pub(crate) use crate::phase_eight_process::checkpoint_crash::process::recovery_process::fresh_recovery_raw;

pub(crate) fn fresh_recovery(
    parent: &tempfile::TempDir,
    root: &std::path::Path,
    label: &str,
    expected: &crate::phase_eight_process::history::writer_history::expected_history::ExpectedWriterHistory,
) -> RecoveryReportEnvelope {
    let persisted_fates =
        crate::phase_eight_process::history::persisted_fates::classify_persisted_fates(
            expected, root,
        )
        .expect("checkpoint persisted-fate oracle");
    let (output, report) = fresh_recovery_raw(parent, root, label);
    let report = report.expect("recovery report missing at checkpoint case");
    let observed_fates = crate::production::fate_markers::persisted_fate_tags(&output)
        .expect("checkpoint indexed fate evidence");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        observed_fates,
        persisted_fates,
        "checkpoint recovery fates must agree with independent persisted evidence at {label}; outcome={:?}; stdout={}; stderr={}",
        report.outcome(),
        String::from_utf8_lossy(&output.stdout),
        stderr
    );
    let recovered = matches!(
        report.outcome(),
        worth_store_recovery_runtime::RecoveryReportOutcome::Recovered
    );
    assert_eq!(
        output.status.success(),
        recovered,
        "recovery process exit status did not match its typed outcome at {label}: {stderr}"
    );
    match report.outcome() {
        worth_store_recovery_runtime::RecoveryReportOutcome::Recovered => {
            assert_eq!(report.denial_cause(), None);
            assert!(stderr.contains("C8_RECOVERY_RUNTIME"));
            assert!(report.counters().peak_recovery_bytes() > 0);
        }
        worth_store_recovery_runtime::RecoveryReportOutcome::Refused => {
            assert!(matches!(
                report.denial_cause(),
                Some(worth_store_recovery_runtime::RecoveryReportDenialCause::Refused(_))
            ));
            assert!(stderr.contains("C8_RECOVERY_REFUSED"));
            assert_eq!(report.counters().recovery_effects(), 0);
        }
        worth_store_recovery_runtime::RecoveryReportOutcome::Blocked => {
            assert!(matches!(
                report.denial_cause(),
                Some(worth_store_recovery_runtime::RecoveryReportDenialCause::Blocked(_))
            ));
            assert!(stderr.contains("C8_RECOVERY_BLOCKED"));
            assert_eq!(report.counters().recovery_effects(), 0);
        }
        worth_store_recovery_runtime::RecoveryReportOutcome::PublicationIndeterminate => {
            assert_eq!(
                report.denial_cause(),
                Some(worth_store_recovery_runtime::RecoveryReportDenialCause::PublicationSettlementIndeterminate)
            );
            assert!(stderr.contains("C8_RECOVERY_PUBLICATION_INDETERMINATE"));
            assert!(report.counters().recovery_effects() > 0);
        }
    }
    report
}

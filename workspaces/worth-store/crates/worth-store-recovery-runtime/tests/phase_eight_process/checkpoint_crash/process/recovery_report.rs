use worth_store_recovery_runtime::{
    RecoveryReportDenialCause, RecoveryReportEnvelope, RecoveryReportOutcome,
};

use super::recovery_process::fresh_recovery_raw;

pub(crate) fn fresh_recovery(
    parent: &tempfile::TempDir,
    root: &std::path::Path,
    label: &str,
    expected: &super::super::super::history::ExpectedWriterHistory,
) -> RecoveryReportEnvelope {
    let persisted_fates = super::super::super::history::classify_persisted_fates(expected, root)
        .expect("checkpoint persisted-fate oracle");
    let (output, report) = fresh_recovery_raw(parent, root, label);
    let report = report.expect("recovery report missing at checkpoint case");
    let observed_fates = super::super::super::production::persisted_fate_tags(&output)
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
    let recovered = matches!(report.outcome(), RecoveryReportOutcome::Recovered);
    assert_eq!(
        output.status.success(),
        recovered,
        "recovery process exit status did not match its typed outcome at {label}: {stderr}"
    );
    match report.outcome() {
        RecoveryReportOutcome::Recovered => {
            assert_eq!(report.denial_cause(), None);
            assert!(stderr.contains("C8_RECOVERY_RUNTIME"));
            assert!(report.counters().peak_recovery_bytes() > 0);
        }
        RecoveryReportOutcome::Refused => {
            assert!(matches!(
                report.denial_cause(),
                Some(RecoveryReportDenialCause::Refused(_))
            ));
            assert!(stderr.contains("C8_RECOVERY_REFUSED"));
            assert_eq!(report.counters().recovery_effects(), 0);
        }
        RecoveryReportOutcome::Blocked => {
            assert!(matches!(
                report.denial_cause(),
                Some(RecoveryReportDenialCause::Blocked(_))
            ));
            assert!(stderr.contains("C8_RECOVERY_BLOCKED"));
            assert_eq!(report.counters().recovery_effects(), 0);
        }
        RecoveryReportOutcome::PublicationIndeterminate => {
            assert_eq!(
                report.denial_cause(),
                Some(RecoveryReportDenialCause::PublicationSettlementIndeterminate)
            );
            assert!(stderr.contains("C8_RECOVERY_PUBLICATION_INDETERMINATE"));
            assert!(report.counters().recovery_effects() > 0);
        }
    }
    report
}

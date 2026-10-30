use {
    worth_store_recovery_runtime::RecoveryReportDenialCause,
    worth_store_recovery_runtime::RecoveryReportEnvelope,
    worth_store_recovery_runtime::RecoveryReportOutcome,
    worth_store_recovery_runtime::RecoveryReportRefusalCause,
};

use {
    crate::phase_eight_process::production::harness::process::run_recovery_with_profile,
    crate::production::terminal_fixture::certification_persisted_root,
};

#[test]
fn certification_refusal_profile_emits_a_typed_terminal_report() {
    // Certification-only terminal profile; process-death evidence belongs to
    // the default admission lane in checkpoint_crash and production::fates.
    let retained_root = certification_persisted_root("c8-phase8-refused");
    let parent = tempfile::tempdir().expect("refusal process parent");
    let report_path = parent.path().join("refused-report.bin");
    let (_, output) = run_recovery_with_profile(
        retained_root.path(),
        &report_path,
        parent.path(),
        "c8-phase8-refused-v1",
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("C8_RECOVERY_REFUSED"));
    let report = worth_store_recovery_runtime::RecoveryReportEnvelope::decode(
        &std::fs::read(&report_path).expect("refusal process report bytes"),
    )
    .expect("refusal process report");
    assert_eq!(
        report.outcome(),
        worth_store_recovery_runtime::RecoveryReportOutcome::Refused
    );
    assert_eq!(
        report.denial_cause(),
        Some(worth_store_recovery_runtime::RecoveryReportDenialCause::Refused(
            worth_store_recovery_runtime::RecoveryReportRefusalCause::CancelledBeforeReconstruction,
        ))
    );
    assert_eq!(report.counters().recovery_effects(), 0);
    assert_eq!(report.store_identity(), None);
}

#[test]
fn certification_publication_profile_emits_a_typed_terminal_report() {
    // Certification-only terminal profile; this is not a process-death
    // outcome and is excluded from the fresh-process crash matrix.
    let retained_root = certification_persisted_root("c8-phase8-publication-indeterminate");
    let parent = tempfile::tempdir().expect("publication process parent");
    let report_path = parent.path().join("publication-indeterminate-report.bin");
    let (_, output) = run_recovery_with_profile(
        retained_root.path(),
        &report_path,
        parent.path(),
        "c8-phase8-publication-indeterminate-v1",
    );
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("C8_RECOVERY_PUBLICATION_INDETERMINATE"),
        "publication-indeterminate recovery stderr:\n{stderr}"
    );
    let report = worth_store_recovery_runtime::RecoveryReportEnvelope::decode(
        &std::fs::read(&report_path).expect("publication process report bytes"),
    )
    .expect("publication process report");
    assert_eq!(
        report.outcome(),
        worth_store_recovery_runtime::RecoveryReportOutcome::PublicationIndeterminate
    );
    assert_eq!(
        report.denial_cause(),
        Some(worth_store_recovery_runtime::RecoveryReportDenialCause::PublicationSettlementIndeterminate)
    );
    assert!(report.store_identity().is_some());
}

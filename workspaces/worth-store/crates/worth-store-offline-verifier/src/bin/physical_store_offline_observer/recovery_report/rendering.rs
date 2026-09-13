use worth_store_offline_verifier::RecoveryObserverReport;

pub(super) fn emit(report: &RecoveryObserverReport) {
    eprintln!(
        "observed {} recovery artifacts and {} bytes; artifact set {}",
        report.artifact_count(),
        report.bytes_read(),
        super::super::hex(&report.artifact_set_digest())
    );
}

use worth_store_offline_integrity_observer::{
    emit_offline_integrity_report, OfflineIntegrityObservationRequest, OfflineIntegrityReport,
};

pub(super) fn emit(
    request: &OfflineIntegrityObservationRequest,
    report: &OfflineIntegrityReport,
) -> Result<(), String> {
    emit_offline_integrity_report(request, report)
        .map_err(|denial| format!("report emission denied: {denial:?}"))
}

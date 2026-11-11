//! Retention assertions borrow the existing courts' actual native writeback results.
use crate::facade::{
    BridgeDiagnosticsFacade, BridgeDiagnosticsHandle, BridgeWritebackDiagnosticReference,
    BridgeWritebackExecutionRecord, BridgeWritebackReplayBundle, BridgeWritebackReplayRecord,
};
use worth_foundational::facade::{
    DiagnosticRecordAbsence as Absence, DiagnosticRecordReadDenial as Denial,
};

pub(super) fn execution_expiry(
    recorder: &BridgeDiagnosticsFacade,
    record: &BridgeWritebackExecutionRecord,
    receipt: &crate::adapter::TruthWritebackReceipt,
) {
    let diagnostics = recorder.handle();
    let reference = diagnostics
        .retained_writeback_execution_reference_for_identity(record.record_identity())
        .unwrap();
    assert_eq!(
        &diagnostics
            .writeback_execution_record_for_reference(&reference)
            .unwrap(),
        record
    );
    // A duplicate diagnostic identity is a new retained entry, not a new effect.
    recorder.record_writeback_execution(record.clone());
    assert!(matches!(
        diagnostics.writeback_execution_record_for_reference(&reference),
        Err(Denial::Absent(Absence::ExpiredUnderCountLimit))
    ));
    let current = diagnostics
        .retained_writeback_execution_reference_for_identity(record.record_identity())
        .unwrap();
    assert_eq!(
        diagnostics
            .writeback_execution_record_for_reference(&current)
            .unwrap()
            .receipt_digest(),
        Some(receipt.digest())
    );
    assert_eq!(receipt.request_digest(), record.request_digest().unwrap());
    assert_eq!(diagnostics.writeback_execution_records().len(), 1);
}
pub(super) fn comparison_expiry(
    diagnostics: &BridgeDiagnosticsHandle,
    reference: &BridgeWritebackDiagnosticReference<BridgeWritebackReplayRecord>,
    bundle: &BridgeWritebackReplayBundle,
) {
    assert!(matches!(
        diagnostics.writeback_replay_record_for_reference(reference),
        Err(Denial::Absent(Absence::ExpiredUnderCountLimit))
    ));
    let current = diagnostics.last_writeback_replay_record().unwrap();
    assert_eq!(current.expected_replay_digest(), bundle.digest());
    assert_eq!(current.failure_class(), None);
    assert_eq!(diagnostics.writeback_replay_records().len(), 1);
}

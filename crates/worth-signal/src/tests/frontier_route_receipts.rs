use crate::facade::adapters::{
    FrontierRouteEvidenceReason, FrontierRouteEvidenceReceipt, FrontierRouteEvidenceReceiptError,
    FrontierRouteSerialFallbackReason,
};
use crate::facade::specialist::{
    ParallelAdmissionReason, StageExecutionOutcome, StageExecutionRecord,
};

#[test]
fn stage_execution_record_lowers_into_parallel_frontier_route_receipt() {
    let stage = sample_stage_execution_record(Some(
        ParallelAdmissionReason::AdmittedProofSafeGroupedConcurrent,
    ));

    let receipt = FrontierRouteEvidenceReceipt::from_stage_execution_record(&stage)
        .expect("parallel admission reason should lower into a frontier route receipt");

    assert_eq!(
        receipt.reason(),
        FrontierRouteEvidenceReason::AdmittedProofSafeGroupedConcurrent
    );
    assert!(receipt.is_parallel_admitted());
    assert_eq!(receipt.serial_fallback_reason(), None);
}

#[test]
fn stage_execution_record_lowers_into_serial_frontier_route_receipt() {
    let stage = sample_stage_execution_record(Some(ParallelAdmissionReason::BelowMinStageWidth));

    let receipt = FrontierRouteEvidenceReceipt::from_stage_execution_record(&stage)
        .expect("serial fallback reason should lower into a frontier route receipt");

    assert_eq!(
        receipt.serial_fallback_reason(),
        Some(FrontierRouteSerialFallbackReason::BelowMinStageWidth)
    );
    assert!(!receipt.is_parallel_admitted());
}

#[test]
fn stage_execution_record_without_reason_is_rejected() {
    let stage = sample_stage_execution_record(None);

    let error = FrontierRouteEvidenceReceipt::from_stage_execution_record(&stage)
        .expect_err("missing parallel admission reason should be rejected");

    assert_eq!(
        error,
        FrontierRouteEvidenceReceiptError::MissingParallelAdmissionReason
    );
}

#[test]
fn frontier_route_receipt_compile_fail_boundaries_hold() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/frontier_route_evidence_receipt_fields_are_private.rs");
}

fn sample_stage_execution_record(reason: Option<ParallelAdmissionReason>) -> StageExecutionRecord {
    StageExecutionRecord {
        stage_index: 0,
        outcome: StageExecutionOutcome::CompletedSerial,
        authority_policy: None,
        parallel_admission_reason: reason,
        parallel_kind: None,
        apply_mode: None,
        apply_group_count: 0,
        serial_apply_rejection_reason: None,
        serial_fallback_group_count: 0,
        concurrent_apply_task_count: 0,
        serial_apply_task_count: 0,
        snapshot_duration_nanos: 0,
        precompute_duration_nanos: 0,
        apply_duration_nanos: 0,
        semantic_finalize_duration_nanos: 0,
        duration_nanos: 0,
        semantic_task_range: None,
        semantic_segment_count: 0,
        task_records: Vec::new(),
    }
}

use super::super::types::{
    ParallelAdmissionReason, ParallelExecutionKind, StageExecutionOutcome, StageExecutionRecord,
};

pub(crate) fn begin_stage_record(
    stage_index: u32,
    snapshot_nanos: u128,
    precompute_nanos: u128,
    reports: &[worth_foundational::ExecutionReport],
) -> StageExecutionRecord {
    let parallel = reports
        .iter()
        .any(|report| report.resolved_posture() == worth_foundational::ExecutionPosture::Automatic);
    StageExecutionRecord {
        stage_index,
        outcome: if parallel {
            StageExecutionOutcome::CompletedParallel
        } else {
            StageExecutionOutcome::CompletedSerial
        },
        authority_policy: None,
        parallel_admission_reason: Some(if parallel {
            ParallelAdmissionReason::AdmittedProofSafeGroupedConcurrent
        } else {
            ParallelAdmissionReason::SerialExecutor
        }),
        parallel_kind: parallel.then_some(ParallelExecutionKind::FullParallel),
        apply_mode: None,
        apply_group_count: 0,
        serial_apply_rejection_reason: None,
        serial_fallback_group_count: 0,
        concurrent_apply_task_count: 0,
        serial_apply_task_count: 0,
        snapshot_duration_nanos: snapshot_nanos,
        precompute_duration_nanos: precompute_nanos,
        apply_duration_nanos: 0,
        semantic_finalize_duration_nanos: 0,
        duration_nanos: 0,
        semantic_task_range: None,
        semantic_segment_count: 0,
        task_records: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_foundational::{ExecutionPhysicalReport, ExecutionPosture, ExecutionReport};

    #[test]
    fn stage_outcome_uses_resolved_posture_even_when_one_worker_is_observed() {
        for (posture, expected) in [
            (
                ExecutionPosture::Serial,
                StageExecutionOutcome::CompletedSerial,
            ),
            (
                ExecutionPosture::Automatic,
                StageExecutionOutcome::CompletedParallel,
            ),
        ] {
            let report = ExecutionReport::new(
                posture,
                12,
                12,
                ExecutionPhysicalReport::new(1, 0, Some(0), 0, 0),
            );
            assert_eq!(begin_stage_record(0, 0, 0, &[report]).outcome, expected);
        }
    }
}

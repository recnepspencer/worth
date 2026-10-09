use super::super::types::{
    ParallelAdmissionReason, ParallelExecutionKind, StageExecutionOutcome, StageExecutionRecord,
};

pub(in crate::logic::planner) fn execution_resolved_parallel(
    reports: &[worth_foundational::ExecutionReport],
) -> bool {
    reports
        .iter()
        .any(|report| report.resolved_posture() == worth_foundational::ExecutionPosture::Automatic)
}

pub(crate) fn begin_stage_record(
    stage_index: u32,
    snapshot_nanos: u128,
    precompute_nanos: u128,
    reports: &[worth_foundational::ExecutionReport],
) -> StageExecutionRecord {
    let parallel = execution_resolved_parallel(reports);
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
    #[test]
    fn serial_resolution_with_a_multiworker_lease_keeps_apply_and_precompute_serial() {
        use crate::facade::{
            AspectVersion, BoundedSignalInputs, EvaluationRequestMode, NodeContract,
            ParallelAdmissionPolicy, ParallelApplyMode, SignalGraph, SignalRuntimePolicy,
        };
        use crate::tests::leased_execution::support::{authority, request};

        let physical_pair = ExecutionReport::new(
            ExecutionPosture::Serial,
            1,
            1,
            ExecutionPhysicalReport::new(2, 0, Some(0), 0, 0),
        );
        assert!(!execution_resolved_parallel(&[physical_pair]));
        let serial_record = begin_stage_record(0, 0, 0, &[physical_pair]);
        assert_eq!(
            serial_record.outcome,
            StageExecutionOutcome::CompletedSerial
        );
        assert_eq!(serial_record.parallel_kind, None);
        let mut graph = SignalGraph::new();
        crate::logic::planner::precompute::reporting::record_stage_precompute_telemetry(
            &mut graph,
            1,
            0,
            0,
            &[physical_pair],
        );
        assert_eq!(graph.telemetry().execution.parallel_stage_dispatch_count, 0);
        assert_eq!(
            graph.telemetry().execution.parallel_precompute_task_count,
            0
        );
        assert_eq!(graph.telemetry().execution.serial_precompute_task_count, 1);

        graph.set_runtime_policy(SignalRuntimePolicy::forensic().with_parallel_admission(
            ParallelAdmissionPolicy {
                throughput_min_parallel_tasks: 1,
                balanced_min_parallel_tasks: 1,
                latency_bounded_min_parallel_tasks: 1,
                full_parallel_min_tasks: 1,
            },
        ));
        let node = graph
            .node()
            .with_contract(
                NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()),
            )
            .build();
        let lease = authority().request_lease(request(4, 1_000_000)).unwrap();
        assert_eq!(lease.policy().budget().max_workers().get(), 4);
        // One partition resolves Serial even with four workers permitted; thresholds admit the grouped apply door.
        let report = graph
            .evaluate_checked(
                &[node],
                EvaluationRequestMode::Default,
                &(),
                &|_| Ok(AspectVersion::zero()),
                worth_execution::ExecutionRequest::leased(&lease),
            )
            .unwrap();
        assert_eq!(report.stages.len(), 1);
        let stage = &report.stages[0];
        assert_eq!(
            stage.apply_mode,
            Some(ParallelApplyMode::GroupedConcurrentApply)
        );
        assert_eq!(stage.concurrent_apply_task_count, 1);
        assert_eq!(report.execution.len(), 3);
        assert!(report.execution[..2]
            .iter()
            .all(|child| child.resolved_posture() == ExecutionPosture::Serial));
        assert_eq!(stage.outcome, StageExecutionOutcome::CompletedSerial);
        assert_eq!(stage.parallel_kind, None);
        let counts = &graph.telemetry().execution;
        assert_eq!(counts.parallel_stage_dispatch_count, 0);
        assert_eq!(counts.parallel_precompute_task_count, 0);
        assert_eq!(counts.serial_precompute_task_count, 2);
    }
}

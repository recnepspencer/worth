//! Typed stops and admission custody, observed at the workflow entry.
use super::super::super::WorkflowFrontierFailure;
use super::*;
use domain::WorthQueryWorkflowAdvanceDenialKind as Denial;
use worth_execution::{
    CancellationSource, LeaseDenial, MapKernelStop, SerialMemoryBudget, SerialRequest,
};
fn denial(record: &Record) -> &Denial {
    record.outcome.as_ref().unwrap()
}
#[test]
fn zero_budget_refuses_before_compute_and_large_tasks_are_charged() {
    let probe = phased_workflow::fold_executor::ComputeProbe::new();
    for bytes in [0, 64 * 1024] {
        for workers in [0, 1, 2, 4] {
            probe.take_computes();
            let serial = SerialRequest::from_memory(
                SerialMemoryBudget::new(bytes),
                CancellationToken::new(),
                None,
            );
            let lease = (workers != 0).then(|| {
                probe
                    .authority()
                    .request_lease(lease_request(workers, bytes, 1024))
                    .unwrap()
            });
            let request = lease
                .as_ref()
                .map_or(ExecutionRequest::serial(&serial), ExecutionRequest::leased);
            let mut workspace = phased_workflow::phased_workspace("width-oracle");
            let mut input = members(1, None);
            if bytes != 0 {
                input[0].1 = domain::WorthQueryWorkflowValue::Text("1:9:131072".into());
            }
            let (record, work, failed) =
                observe_members(start(&mut workspace), input, &mut workspace, request);
            assert!(matches!(
                denial(&record),
                Denial::ComputationAdmission(LeaseDenial::MemoryExhausted(_))
            ));
            assert!(!failed);
            assert_eq!(work, 0);
            assert!(probe.take_computes().is_empty());
            assert_eq!(record.receipts.len(), 1);
            assert!(record.effects.is_empty());
        }
    }
}
#[test]
fn panic_and_result_capacity_fail_after_canonical_prefix() {
    let _probe = phased_workflow::fold_executor::ComputeProbe::new();
    for mode in [6, 7, 11] {
        let serial = super::super::request::workflow_request();
        let (record, work, failed) = observe(7, Some((1, mode)), ExecutionRequest::serial(&serial));
        assert!(failed);
        assert_eq!(work, COSTS[0] + COSTS[1]);
        assert_eq!(record.receipts.len(), 2);
        assert_eq!(record.effects.len(), 1);
        if mode == 6 {
            assert_eq!(
                denial(&record),
                &Denial::ComputationPanic {
                    stage_identity: Some("middle".into())
                }
            );
        } else {
            assert_eq!(
                denial(&record),
                &Denial::ComputationResultCapacity {
                    stage_identity: Some("middle".into())
                }
            );
        }
    }
}
#[test]
fn work_ceiling_settles_the_same_boundary_at_all_widths() {
    let probe = phased_workflow::fold_executor::ComputeProbe::new();
    for workers in [1, 2, 4] {
        let lease = probe
            .authority()
            .request_lease(lease_request(workers, 64 * 1024 * 1024, COSTS[0]))
            .unwrap();
        let (record, work, failed) = observe(7, None, ExecutionRequest::leased(&lease));
        assert!(!failed);
        assert_eq!(work, COSTS[0]);
        assert_eq!(
            denial(&record),
            &Denial::ComputationWorkExhausted {
                stage_identity: Some("middle".into()),
                cause: MapKernelStop::WorkCeiling
            }
        );
        assert_eq!(record.receipts.len(), 2);
        assert_eq!(record.effects.len(), 1);
    }
}
#[test]
fn nested_stop_is_a_stage_bound_contract_failure_at_every_posture() {
    let probe = phased_workflow::fold_executor::ComputeProbe::new();
    let serial = super::super::request::workflow_request();
    for workers in [0, 1, 2, 4] {
        let lease = (workers != 0).then(|| {
            probe
                .authority()
                .request_lease(lease_request(workers, 64 * 1024 * 1024, 1024))
                .unwrap()
        });
        let request = lease
            .as_ref()
            .map_or(ExecutionRequest::serial(&serial), ExecutionRequest::leased);
        let (record, work, failed) = observe(7, Some((1, 10)), request);
        assert!(failed);
        assert_eq!(work, COSTS[0] + COSTS[1]);
        assert_eq!(
            denial(&record),
            &Denial::ComputationNestedStopped {
                stage_identity: Some("middle".into())
            }
        );
        assert_eq!(record.receipts.len(), 2);
        assert_eq!(record.effects.len(), 1);
    }
}
#[test]
fn every_scope_stop_keeps_its_specific_typed_cause() {
    use super::super::super::stop_conversion;
    use worth_execution::WorkCeilingDenial;
    let cases = [
        (
            WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped),
            Denial::ComputationNestedStopped {
                stage_identity: None,
            },
        ),
        (
            WorkCeilingDenial::Admission(LeaseDenial::NoActiveExecutionScope),
            Denial::ComputationAdmission(LeaseDenial::NoActiveExecutionScope),
        ),
        (
            WorkCeilingDenial::Admission(LeaseDenial::ChargedBytesOverflow),
            Denial::ComputationAdmission(LeaseDenial::ChargedBytesOverflow),
        ),
        (
            WorkCeilingDenial::Stopped(MapKernelStop::DeadlineElapsed),
            Denial::ComputationDeadline {
                stage_identity: None,
            },
        ),
        (
            WorkCeilingDenial::Stopped(MapKernelStop::Cancelled),
            Denial::ComputationCancelled {
                stage_identity: None,
            },
        ),
        (
            WorkCeilingDenial::Stopped(MapKernelStop::WorkCounterOverflow),
            Denial::ComputationWorkExhausted {
                stage_identity: None,
                cause: MapKernelStop::WorkCounterOverflow,
            },
        ),
    ];
    for (stop, expected) in cases {
        let WorkflowFrontierFailure::Denied(actual) = stop_conversion::scope(stop) else {
            panic!("typed refusal")
        };
        assert_eq!(actual, expected);
    }
}

#[test]
fn already_elapsed_deadline_and_cancelled_entry_do_not_compute() {
    let probe = phased_workflow::fold_executor::ComputeProbe::new();
    for cancelled in [false, true] {
        probe.take_computes();
        let source = CancellationSource::new();
        let deadline = if cancelled {
            source.cancel();
            None
        } else {
            Some(std::time::Instant::now() - std::time::Duration::from_secs(1))
        };
        let serial = SerialRequest::from_memory(
            SerialMemoryBudget::new(64 * 1024 * 1024),
            source.token(),
            deadline,
        );
        let (record, work, failed) = observe(7, None, ExecutionRequest::serial(&serial));
        let expected = if cancelled {
            Denial::ComputationCancelled {
                stage_identity: None,
            }
        } else {
            Denial::ComputationDeadline {
                stage_identity: None,
            }
        };
        assert_eq!(denial(&record), &expected);
        assert!(!failed);
        assert_eq!(work, 0);
        assert!(probe.take_computes().is_empty());
        assert!(record.effects.is_empty());
        assert_eq!(record.receipts.len(), 1);
    }
}

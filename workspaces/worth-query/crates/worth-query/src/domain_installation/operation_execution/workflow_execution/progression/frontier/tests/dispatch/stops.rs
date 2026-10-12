//! Typed stops and admission custody, observed at the workflow entry.
use super::super::super::WorkflowFrontierFailure;
use super::*;
use domain::WorthQueryWorkflowAdvanceDenialKind as Denial;
use worth_execution::{LeaseDenial, MapKernelStop};
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource as CancellationSource;
fn denial(record: &Record) -> &Denial {
    record.outcome.as_ref().unwrap()
}
#[test]
fn zero_memory_public_frontier_refuses_without_retaining_start_receipts() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    use worth_query_execution::facade::application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationResourceDenial,
    };
    let probe = phased_workflow::fold_executor::ComputeProbe::new();
    let mut prepared = Policy::width(0).workspace(request_execution, &probe);
    let held = start(&mut prepared);
    let mut workspace = Policy {
        memory: 0,
        ..Policy::width(0)
    }
    .workspace(request_execution, &probe);
    probe.take_computes();
    // Admission precedes any inspection of the held run or the frontier. The
    // refused call must retain none of that input's earlier start evidence.
    let (record, work, failed) = observe_members(held, members(1, None), &mut workspace, None);
    let Some(Denial::ExecutionRequest(WorthQueryAdvancementDenial::Resource(
        WorthQueryManagedComputationResourceDenial::MemoryLimit {
            level,
            requested,
            admitted,
        },
    ))) = record.outcome
    else {
        panic!(
            "the serial request retains its leased memory refusal: {:?}",
            record.outcome
        );
    };
    assert_eq!(
        level,
        worth_query_execution::facade::application_contribution::WorthQueryMemoryLimitLevel::Policy
    );
    assert_eq!(admitted, 0);
    assert!(requested > 0);
    assert!(!failed);
    assert_eq!(work, 0);
    assert!(probe.take_computes().is_empty());
    assert_eq!(record.receipts.len(), 0);
    assert!(record.effects.is_empty());
}

#[test]
fn admitted_opening_refuses_the_owned_maps_declared_reservation() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let probe = phased_workflow::fold_executor::ComputeProbe::new();
    // The fixture's start stage has an empty payload. Its exact map reservation
    // is derived below; the frontier owns a larger Words payload, independently
    // of its zero-work declaration. No measured or fitted budget is used.
    let bytes = super::reservation::start_reservation(request_execution, &probe);
    let mut workspace = Policy {
        memory: bytes,
        ..Policy::width(0)
    }
    .workspace(request_execution, &probe);
    let run = start(&mut workspace);
    probe.take_computes();
    let mut input = members(1, None);
    input[0].1 = domain::WorthQueryWorkflowValue::Text("1:9:131072".into());
    let authority = probe.authority();
    let lease = authority
        .request_lease(worth_execution::LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::MIN, 64 * 1024 * 1024, 1024),
            ),
            cancellation: worth_execution::CancellationToken::new(),
            deadline: None,
        })
        .unwrap();
    let root = super::reservation::framework(&lease, 1);
    let mut sample = Policy::width(0).workspace(request_execution, &probe);
    let sample_run = start(&mut sample);
    let (requested, admitted) = sample
        .advancement_owner()
        .with_advancement(|_phase| {
            let nested = super::reservation::framework(&lease, 1);
            let map = super::reservation::framework(&lease, 3);
            let mut declaration = members(1, None);
            declaration[0].1 = domain::WorthQueryWorkflowValue::Text("1:9:131072".into());
            (
                map + super::reservation::declared_reservation(
                    sample_run.prepare_frontier_computation(declaration),
                ),
                bytes - root - nested,
            )
        })
        .unwrap();
    assert!(requested > bytes);
    let (record, work, failed) = observe_members(run, input, &mut workspace, None);
    assert_eq!(
        denial(&record),
        &Denial::ComputationAdmission(LeaseDenial::MemoryExhausted(
            worth_execution::MemoryLimitDenial {
                requested,
                admitted,
                level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 },
            }
        ))
    );
    assert!(!failed);
    assert_eq!(work, 0);
    assert!(probe.take_computes().is_empty());
    assert_eq!(record.receipts.len(), 1);
    assert!(record.effects.is_empty());
}
#[test]
fn large_task_refusal_stays_typed_at_every_width() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let probe = phased_workflow::fold_executor::ComputeProbe::new();
    // Preserve the incoming positive-memory width oracle independently of the
    // new declaration-derived minimum-budget case above.
    for workers in [0, 1, 2, 4] {
        let mut workspace = Policy {
            memory: 64 * 1024,
            ..Policy::width(workers)
        }
        .workspace(request_execution, &probe);
        let run = start(&mut workspace);
        probe.take_computes();
        let mut input = members(1, None);
        input[0].1 = domain::WorthQueryWorkflowValue::Text("1:9:131072".into());
        let (record, work, failed) = observe_members(run, input, &mut workspace, None);
        assert!(matches!(
            denial(&record),
            Denial::ComputationAdmission(LeaseDenial::MemoryExhausted(
                worth_execution::MemoryLimitDenial {
                    level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 },
                    ..
                }
            ))
        ));
        assert!(!failed);
        assert_eq!(work, 0);
        assert!(probe.take_computes().is_empty());
        assert_eq!(record.receipts.len(), 1);
        assert!(record.effects.is_empty());
    }
}

#[test]
fn panic_and_result_capacity_fail_after_canonical_prefix() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let probe = phased_workflow::fold_executor::ComputeProbe::new();
    for mode in [6, 7, 11] {
        let (record, work, failed) = observe(
            request_execution,
            7,
            Some((1, mode)),
            Policy::width(0),
            &probe,
        );
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
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let probe = phased_workflow::fold_executor::ComputeProbe::new();
    for workers in [1, 2, 4] {
        // The caller funds the real first-stage effect as well as computation.
        // Only the map declaration is short: later tasks still attempt their
        // original work, so the second task meets the exact computation ceiling.
        let mut workspace = Policy::width(workers).workspace(request_execution, &probe);
        let mut input = members(7, None);
        for (rank, (_, value)) in input.iter_mut().enumerate() {
            let declared_work = if rank == 0 { COSTS[rank] } else { 0 };
            *value = domain::WorthQueryWorkflowValue::Text(format!(
                "{}:0:{}:{declared_work}",
                7 + rank as u64,
                COSTS[rank],
            ));
        }
        let (record, work, failed) =
            observe_members(start(&mut workspace), input, &mut workspace, None);
        assert!(!failed, "width {workers}: {record:?}");
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
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let probe = phased_workflow::fold_executor::ComputeProbe::new();
    for workers in [0, 1, 2, 4] {
        let (record, work, failed) = observe(
            request_execution,
            7,
            Some((1, 10)),
            Policy::width(workers),
            &probe,
        );
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
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

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
        let scope =
            WorthQueryRequestScope::new(
                Some(deadline.unwrap_or_else(|| {
                    std::time::Instant::now() + std::time::Duration::from_secs(60)
                })),
                source.token(),
            );
        let mut workspace = Policy::width(0).workspace(request_execution, &probe);
        let run = start(&mut workspace);
        let (record, work, failed) =
            observe_members(run, members(7, None), &mut workspace, Some(&scope));
        use worth_query_execution::facade::application_contribution::{
            WorthQueryAdvancementDenial as Advancement,
            WorthQueryManagedComputationInterruption as Interrupted,
        };
        assert_eq!(
            denial(&record),
            &Denial::ExecutionRequest(Advancement::Interrupted(if cancelled {
                Interrupted::Cancelled
            } else {
                Interrupted::DeadlineExceeded
            }))
        );
        assert!(!failed);
        assert_eq!(work, 0);
        assert!(probe.take_computes().is_empty());
        assert!(record.effects.is_empty());
        assert_eq!(record.receipts.len(), 0);
    }
}

#[path = "stops/caller_budget.rs"]
mod caller_budget;

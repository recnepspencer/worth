//! Read-owned declines may be handled; original execution refusal remains stopping.
use super::fixtures::*;
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::facade::runtime::{
    PinnedAggregateReadDenialKind as Denial, PinnedAggregateReadScope as Scope,
};
use crate::tests::support::*;
use std::cell::{Cell, RefCell};
use std::num::NonZeroUsize;
use worth_execution::{CancellationSource, ExecutionRequest, LeaseRequest, SerialRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

fn lease(
    work: u64,
    cancellation: &CancellationSource,
) -> worth_execution::ExecutionResourceLease<'static> {
    crate::tests::support::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::MIN, Some(128 * 1024 * 1024), Some(work)),
            ),
            cancellation: cancellation.token(),
            deadline: None,
        })
        .unwrap()
}

#[test]
fn local_read_limit_is_data_and_funded_retry_uses_the_same_accounted_or_unaccounted_extent() {
    let setup = crate::test_execution_policy::serial_request();
    let setup_execution = &ExecutionRequest::serial(&setup);
    let runtime = runtime();
    let ring = (0..32).map(|n| (n + 1) % 32).collect::<Vec<_>>();
    let born = commit(setup_execution, &runtime, "main", birth(&ring));
    let member = members(&born, 32)[0];
    let pin = runtime
        .read_truth()
        .pin_read_capability(&born.snapshot, &[], setup_execution)
        .unwrap();
    for accounted in [false, true] {
        let cancellation = CancellationSource::new();
        let lease = lease(200_000, &cancellation);
        let policy = ExecutionRequestPolicy::for_workers(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            NonZeroUsize::MIN,
        );
        let host = SerialRequest::from_policy(&policy, cancellation.token(), None);
        let execution = if accounted {
            ExecutionRequest::leased(&lease)
        } else {
            ExecutionRequest::serial(&host)
        };
        let reached = Cell::new(false);
        let outer = execution.run_native_with_checkpoint(|context| {
            if context.has_accounting() {
                context.checkpoint(3).unwrap();
            } else {
                context.poll().unwrap();
            }
            let denial = pin
                .read_aggregate(
                    &execution,
                    Scope::new(KindId(2), member).with_work_limit(Some(100)),
                )
                .unwrap_err();
            assert_eq!(denial.kind, Denial::WorkLimitExceeded);
            assert!(denial.reads.entity_records > 0 && denial.reads.adjacency_lists > 0);
            assert!(denial.work > 100);
            if context.has_accounting() {
                context.checkpoint(5).unwrap();
            } else {
                context.poll().unwrap();
            }
            let funded = pin
                .read_aggregate(
                    &execution,
                    Scope::new(KindId(2), member).with_work_limit(Some(100_000)),
                )
                .unwrap();
            assert_eq!(funded.len(), 32);
            assert!(funded.work_units() > denial.work);
            drop(funded);
            if context.has_accounting() {
                context.checkpoint(7).unwrap();
            } else {
                context.poll().unwrap();
            }
            reached.set(true);
        });
        assert!(
            outer.is_ok(),
            "a local decline must not poison this original extent: {outer:?}"
        );
        assert!(reached.get());
        assert_eq!(lease.charged_memory_bytes(), 0);
        assert!(
            host.memory().is_none(),
            "worker-only serial retains no invented payer"
        );
    }
    release_test_commit_snapshot(&runtime, &born);
}

#[test]
fn ignored_request_work_cancellation_and_memory_failures_still_stop_the_original_parent() {
    let setup = crate::test_execution_policy::serial_request();
    let setup_execution = &ExecutionRequest::serial(&setup);
    let runtime = runtime();
    let ring = (0..32).map(|n| (n + 1) % 32).collect::<Vec<_>>();
    let born = commit(setup_execution, &runtime, "main", birth(&ring));
    let member = members(&born, 32)[0];
    let pin = runtime
        .read_truth()
        .pin_read_capability(&born.snapshot, &[], setup_execution)
        .unwrap();
    for case in ["work", "cancel", "memory"] {
        let cancellation = CancellationSource::new();
        let lease = lease(if case == "work" { 100 } else { 200_000 }, &cancellation);
        let execution = ExecutionRequest::leased(&lease);
        let ticks = Cell::new(0usize);
        let held = RefCell::new(None);
        let captured = Cell::new(None);
        let control = || {
            let tick = ticks.get() + 1;
            ticks.set(tick);
            if tick == 20 {
                match case {
                    "cancel" => cancellation.cancel(),
                    "memory" => {
                        let remaining = 128 * 1024 * 1024 - lease.charged_memory_bytes();
                        *held.borrow_mut() = Some(lease.reserve_memory(remaining).unwrap());
                    }
                    _ => {}
                }
            }
            Ok(())
        };
        let outer = execution.run_native_with_checkpoint(|_| {
            let denial = pin
                .read_aggregate(
                    &execution,
                    Scope::new(KindId(2), member).with_control(&control),
                )
                .unwrap_err();
            captured.set(Some(denial));
            // Deliberately ignore the typed read refusal at this parent boundary.
        });
        let denial = captured
            .get()
            .expect("the actual read reached a typed refusal");
        assert!(
            matches!(outer, Err(worth_execution::WorkCeilingDenial::Stopped(_))),
            "request refusal must survive ignored read error: {case}: {outer:?}"
        );
        match case {
            "work" => assert_eq!(denial.kind, Denial::Read(Cause::WorkExhausted)),
            "cancel" => assert_eq!(denial.kind, Denial::Read(Cause::Cancelled)),
            "memory" => assert!(
                matches!(
                    denial.kind,
                    Denial::Read(Cause::PolicyMemoryExhausted { .. })
                ),
                "{denial:?}"
            ),
            _ => unreachable!(),
        }
        assert!(
            denial.reads.entity_records > 0 && denial.work > 0,
            "outside-payload observations survive handoff: {denial:?}"
        );
        held.borrow_mut().take();
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
    release_test_commit_snapshot(&runtime, &born);
}

#[derive(Debug)]
struct Empty;
impl worth_execution::ChargedBytes for Empty {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

#[test]
fn external_nested_failure_cannot_be_hidden_by_a_simultaneous_local_read_limit() {
    let setup = crate::test_execution_policy::serial_request();
    let setup_execution = &ExecutionRequest::serial(&setup);
    let runtime = runtime();
    let born = commit(setup_execution, &runtime, "main", birth(&[1, 0]));
    let member = members(&born, 2)[0];
    let pin = runtime
        .read_truth()
        .pin_read_capability(&born.snapshot, &[], setup_execution)
        .unwrap();
    let cancellation = CancellationSource::new();
    let lease = lease(200_000, &cancellation);
    let execution = ExecutionRequest::leased(&lease);
    let retention = RefCell::new(lease.reserve_memory(0).unwrap());
    let triggered = Cell::new(false);
    let captured = Cell::new(None);
    let control = || {
        if !triggered.replace(true) {
            let outcome = execution.run_measured_result(
                worth_execution::ExecutionWorkCeiling::new(None),
                0,
                &mut retention.borrow_mut(),
                |context, _| {
                    context.checkpoint(1)?;
                    Err::<Empty, _>(worth_execution::MapKernelFailure::Domain(Empty))
                },
            );
            assert!(matches!(
                outcome.result,
                Err(worth_execution::ScopeStop::Failure(
                    worth_execution::MapKernelFailure::Domain(Empty)
                ))
            ));
        }
        Ok(())
    };
    let outer = execution.run_native_with_checkpoint(|_| {
        captured.set(Some(
            pin.read_aggregate(
                &execution,
                Scope::new(KindId(2), member)
                    .with_work_limit(Some(0))
                    .with_control(&control),
            )
            .unwrap_err(),
        ));
    });
    assert!(triggered.get());
    assert_eq!(
        captured.get().unwrap().kind,
        Denial::Read(Cause::NestedStopped)
    );
    assert!(matches!(
        outer,
        Err(worth_execution::WorkCeilingDenial::Stopped(
            worth_execution::MapKernelStop::NestedStopped
        ))
    ));
    drop(retention);
    assert_eq!(lease.charged_memory_bytes(), 0);
    release_test_commit_snapshot(&runtime, &born);
}

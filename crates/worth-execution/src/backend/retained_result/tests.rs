use super::*;
use crate::{
    tests::{authority, request, TEST_LOCK},
    CancellationSource, ExecutionMap, ExecutionRequest, ExecutionWorkCeiling, KeylessPartition,
    MapOutcome, MapStop, SerialMemoryBudget, SerialRequest,
};
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicUsize, Ordering},
};

fn map(count: u64, result_bytes: u64) -> ExecutionMap<u64, u64> {
    ExecutionMap::from_keyless_partitions(
        (1..=count)
            .map(|id| {
                (
                    PartitionIdentity::new(id),
                    KeylessPartition {
                        value: id,
                        kernel_scratch_bytes: 0,
                        max_result_bytes: result_bytes,
                    },
                )
            })
            .collect::<BTreeMap<_, _>>(),
    )
    .unwrap()
}

fn kernel(_: &u64, context: &mut KernelContext<'_, '_>) -> Result<u64, KernelFailure<()>> {
    for work in [2, 0, 3, 2] {
        context.checkpoint(work)?;
    }
    Ok(73)
}

#[test]
fn settled_native_result_carries_its_actual_value_and_every_tight_prefix() {
    let _lock = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(1, 2_000, 100)).unwrap();
    let source = map(1, 0);
    let MapOutcome::Complete { mut values, .. } = source
        .run_retained(
            ExecutionRequest::leased(&lease),
            ExecutionWorkCeiling::new(100),
            kernel,
        )
        .unwrap()
    else {
        panic!("real native complete")
    };
    let prior = values.pop().unwrap();
    drop(values);
    assert_eq!(
        (*prior.value(), prior.charged_work(), prior.charged_span()),
        (73, 7, 7)
    );
    assert_eq!(prior.identity(), PartitionIdentity::new(1));
    for ceiling in 0..=9 {
        let fresh = ExecutionWorkCeiling::new(ceiling)
            .run(&lease, || source.run(Some(&lease), kernel))
            .unwrap()
            .0;
        let carried = ExecutionWorkCeiling::new(ceiling)
            .run(&lease, || {
                source.run(Some(&lease), |_, context| {
                    prior
                        .try_carry(context)
                        .expect("checkpoint-only native capture")
                        .copied()
                        .map_err(KernelFailure::<()>::Stop)
                })
            })
            .unwrap()
            .0;
        assert_eq!(
            (fresh.report().charged_work(), fresh.report().charged_span()),
            (
                carried.report().charged_work(),
                carried.report().charged_span()
            ),
            "ceiling={ceiling}"
        );
        match (fresh, carried) {
            (MapOutcome::Complete { values: a, .. }, MapOutcome::Complete { values: b, .. }) => {
                assert_eq!(a, b)
            }
            (
                MapOutcome::Stopped {
                    completed_prefix: a,
                    boundary: x,
                    reason: r,
                    ..
                },
                MapOutcome::Stopped {
                    completed_prefix: b,
                    boundary: y,
                    reason: s,
                    ..
                },
            ) => {
                assert_eq!((a, x, r), (b, y, s), "ceiling={ceiling}");
            }
            _ => panic!("different real native prefix at {ceiling}"),
        }
    }
}

#[test]
fn context_fallback_capacity_and_swallowed_cancellation_issue_no_false_carry() {
    let _lock = TEST_LOCK.lock().unwrap();
    let memory = SerialMemoryBudget::new(64_000);
    let serial = SerialRequest::from_memory(memory.clone(), crate::CancellationToken::new(), None);
    let source = map(1, 0);
    let MapOutcome::Complete { mut values, .. } = source
        .run_retained(
            ExecutionRequest::serial(&serial),
            ExecutionWorkCeiling::new(100),
            |_, context| Ok::<_, KernelFailure<()>>(context.remaining_work()),
        )
        .unwrap()
    else {
        panic!("observed context still computes")
    };
    let observed = values.pop().unwrap();
    let actual = ExecutionWorkCeiling::new(10)
        .run_serial(&serial, || {
            source.run(None, |_, context| {
                assert!(observed.try_carry(context).is_none());
                kernel(&1, context)
            })
        })
        .unwrap()
        .0;
    assert!(
        matches!(actual, MapOutcome::Complete { .. }),
        "fallback really executes"
    );
    drop(observed);
    drop(values);
    let capacity = source
        .run_retained(
            ExecutionRequest::serial(&serial),
            ExecutionWorkCeiling::new(100),
            |_, _| Ok::<_, KernelFailure<()>>(String::from("not admitted")),
        )
        .unwrap();
    assert!(
        matches!(capacity, MapOutcome::Stopped { completed_prefix, reason: MapStop::Failure {
        cause: KernelFailure::ResultCapacityExceeded, .. }, .. } if completed_prefix.is_empty())
    );
    let cancellation = CancellationSource::new();
    let cancelled = serial.clone().with_cancellation(cancellation.token());
    let stopped = source.run_retained(
        ExecutionRequest::serial(&cancelled),
        ExecutionWorkCeiling::new(100),
        |_, context| {
            cancellation.cancel();
            let _ = context.checkpoint(1);
            Ok::<_, KernelFailure<()>>(73_u64)
        },
    );
    assert!(matches!(
        stopped,
        Err(crate::RetainedMapDenial::Request(
            crate::WorkCeilingDenial::Stopped(KernelStop::Cancelled)
        ))
    ));
    let fresh = source
        .run_retained(
            ExecutionRequest::serial(&serial),
            ExecutionWorkCeiling::new(100),
            kernel,
        )
        .unwrap();
    assert!(
        matches!(fresh, MapOutcome::Complete { .. }),
        "fresh lawful retry"
    );
    drop(fresh);
    assert!(
        memory.reserve(64_000).is_ok(),
        "all result/capture/scratch grants released"
    );
}

struct Payload {
    bytes: String,
    drops: Arc<AtomicUsize>,
}
impl ChargedBytes for Payload {
    fn additional_charged_bytes(&self) -> u64 {
        self.bytes.capacity() as u64 + (size_of::<AtomicUsize>() + 2 * size_of::<usize>()) as u64
    }
}
impl Drop for Payload {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::Relaxed);
    }
}

#[test]
fn each_result_owns_only_its_payload_and_final_clone_holds_the_grant() {
    let _lock = TEST_LOCK.lock().unwrap();
    let memory = SerialMemoryBudget::new(64_000);
    let serial = SerialRequest::from_memory(memory.clone(), crate::CancellationToken::new(), None);
    let drops = Arc::new(AtomicUsize::new(0));
    let MapOutcome::Complete { mut values, .. } = map(2, 4_100)
        .run_retained(
            ExecutionRequest::serial(&serial),
            ExecutionWorkCeiling::new(100),
            |_, context| {
                context.checkpoint(1)?;
                Ok::<_, KernelFailure<()>>(Payload {
                    bytes: "x".repeat(4_000),
                    drops: Arc::clone(&drops),
                })
            },
        )
        .unwrap()
    else {
        panic!("native result funding succeeds")
    };
    let retained = values.pop().unwrap();
    let clone = retained.clone();
    drop(retained);
    drop(values);
    assert_eq!(
        drops.load(Ordering::Relaxed),
        1,
        "unrelated result is not a hidden batch parent"
    );
    assert_eq!(clone.value().bytes.len(), 4_000);
    let tight_memory = SerialMemoryBudget::new(2_000);
    let tight =
        SerialRequest::from_memory(tight_memory.clone(), crate::CancellationToken::new(), None);
    let entered = AtomicUsize::new(0);
    let refused = map(2, 4_100).run_retained(
        ExecutionRequest::serial(&tight),
        ExecutionWorkCeiling::new(100),
        |_, _| {
            entered.fetch_add(1, Ordering::Relaxed);
            Ok::<_, KernelFailure<()>>(0_u64)
        },
    );
    assert!(matches!(
        refused,
        Err(crate::RetainedMapDenial::Admission(_))
    ));
    assert_eq!(
        entered.load(Ordering::Relaxed),
        0,
        "retain admission precedes kernel effects"
    );
    assert_eq!(
        clone.value().bytes.len(),
        4_000,
        "prior remains usable after refused replacement"
    );
    assert!(tight_memory.reserve(2_000).is_ok());
    assert!(memory.reserve(64_000).is_err());
    drop(clone);
    assert_eq!(drops.load(Ordering::Relaxed), 2);
    assert!(memory.reserve(64_000).is_ok());
}

#[test]
fn speculative_suffix_request_stop_keeps_native_ownership_after_earlier_domain_failure() {
    let _lock = TEST_LOCK.lock().unwrap();
    let cancellation = CancellationSource::new();
    let memory = SerialMemoryBudget::new(64_000);
    let serial = SerialRequest::from_memory(memory.clone(), cancellation.token(), None);
    let source = crate::tests::batch(&[1, 2], 0);
    let outcome =
        crate::backend::scope::run_scope_within(None, 0, 0, true, 100, Some(&serial), |context| {
            let stopped =
                run_retained_batch(None, &source, BackendKind::Serial, None, &|value, child| {
                    child.checkpoint(1)?;
                    if *value == 1 {
                        return Err(KernelFailure::Domain(()));
                    }
                    cancellation.cancel();
                    let _ = child.checkpoint(1);
                    Ok(73_u64)
                })
                .unwrap();
            assert!(stopped.values.is_empty());
            assert_eq!(stopped.prefix_boundary, Some(PartitionIdentity::new(1)));
            assert!(matches!(
                stopped.stop,
                Some(MapStop::Failure {
                    cause: KernelFailure::Domain(()),
                    ..
                })
            ));
            assert_eq!(
                stopped.stop_owner,
                Some(crate::backend::NestedStopOwner::Request)
            );
            assert_eq!(
                context.take_nested_computation_stop(),
                None,
                "request stop is not a clearable computational stop"
            );
            assert_eq!(context.checkpoint(0), Err(KernelStop::Cancelled));
            Ok::<_, KernelFailure<()>>(())
        });
    assert!(matches!(
        outcome.result,
        Err(crate::backend::scope::ScopeStop::Failure(
            KernelFailure::Stop(KernelStop::Cancelled)
        ))
    ));
    assert!(memory.reserve(64_000).is_ok());
}

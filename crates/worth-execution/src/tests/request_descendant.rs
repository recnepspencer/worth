use super::*;
use crate::{ExecutionRequest, ExecutionWorkCeiling};
use std::collections::BTreeMap;

fn worker_request(posture: ExecutionPosture) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::for_workers(
            posture,
            DeterminismContract::CanonicalBitwise,
            NonZeroUsize::MIN,
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    }
}

#[test]
fn retained_native_original_request_reborrows_the_active_child_without_widening() {
    let parent =
        crate::ExecutionResourceLease::serial(worker_request(ExecutionPosture::Serial)).unwrap();
    let child = parent
        .child(worker_request(ExecutionPosture::Automatic))
        .unwrap();
    assert_ne!(
        parent.policy(),
        child.policy(),
        "distinct original child policy"
    );
    let original = ExecutionRequest::leased(&parent);
    let map = crate::ExecutionMap::<_, ()>::from_native_partitions(BTreeMap::from([
        (PartitionIdentity::new(1), 3_u64),
        (PartitionIdentity::new(2), 7),
    ]))
    .unwrap();
    let ((retained, kept), _) = ExecutionRequest::leased(&child)
        .run_native(|_| {
            let kernel = |value: &u64, context: &mut crate::MapKernelContext<'_, '_>| {
                assert!(!context.has_accounting());
                original
                    .in_scope(|effective| assert_eq!(effective.unwrap().policy(), child.policy()))
                    .unwrap();
                context.poll()?;
                Ok::<_, crate::MapKernelFailure<()>>(value * 2)
            };
            let crate::RetainedRunOutcome::Complete { run, report, .. } = map
                .run_retained_run(original, ExecutionWorkCeiling::new(None), kernel)
                .unwrap()
            else {
                panic!("lawful original-request retained map refused")
            };
            assert_eq!(report.accounted_cost(), None);
            assert_eq!(run.result(PartitionIdentity::new(1)).unwrap().value(), &6);
            assert_eq!(run.result(PartitionIdentity::new(2)).unwrap().value(), &14);
            let kept = run.result(PartitionIdentity::new(1)).unwrap().clone();
            let crate::RetainedRunOutcome::Complete {
                run: next,
                report,
                metrics,
            } = run
                .advance_native(
                    original,
                    ExecutionWorkCeiling::new(None),
                    BTreeMap::from([(PartitionIdentity::new(2), 8_u64)]),
                    worth_proof::CanonicalUniqueVec::try_from_sorted_unique(Vec::new()).unwrap(),
                    |value, context| {
                        original
                            .in_scope(|effective| {
                                assert_eq!(effective.unwrap().policy(), child.policy())
                            })
                            .unwrap();
                        context.poll()?;
                        Ok::<_, crate::MapKernelFailure<()>>(value * 2)
                    },
                )
                .unwrap()
            else {
                panic!("lawful original-request retained advance refused")
            };
            assert_eq!(metrics.carried_slots, 1);
            assert!(kept.same_backing(next.result(PartitionIdentity::new(1)).unwrap()));
            assert_eq!(next.result(PartitionIdentity::new(2)).unwrap().value(), &16);
            assert_eq!(report.accounted_cost(), None);
            (next, kept)
        })
        .unwrap();
    assert_eq!(
        retained.result(PartitionIdentity::new(2)).unwrap().value(),
        &16
    );
    assert!(
        kept.same_backing(retained.result(PartitionIdentity::new(1)).unwrap()),
        "original retained custody survives the child execution scope"
    );
}

#[test]
fn retained_native_original_request_keeps_child_cancellation_and_sibling_refusal() {
    for cancelled in [false, true] {
        let parent =
            crate::ExecutionResourceLease::serial(worker_request(ExecutionPosture::Serial))
                .unwrap();
        let cancellation = CancellationSource::new();
        let child = parent.controlled_child(cancellation.token(), None);
        let sibling = parent
            .child(worker_request(ExecutionPosture::Serial))
            .unwrap();
        let original = ExecutionRequest::leased(if cancelled { &parent } else { &sibling });
        let map = crate::ExecutionMap::<_, ()>::from_native_partitions(BTreeMap::from([(
            PartitionIdentity::new(1),
            3_u64,
        )]))
        .unwrap();
        let entered = std::sync::atomic::AtomicUsize::new(0);
        let outer = ExecutionRequest::leased(&child).run_native(|_| {
            if cancelled {
                cancellation.cancel();
            }
            let refused = map
                .run_retained_run(original, ExecutionWorkCeiling::new(None), |_, _| {
                    entered.fetch_add(1, Ordering::SeqCst);
                    Ok::<_, crate::MapKernelFailure<()>>(99_u64)
                })
                .err()
                .expect("the original admission/control must refuse");
            let expected = if cancelled {
                crate::WorkCeilingDenial::Stopped(KernelStop::Cancelled)
            } else {
                crate::WorkCeilingDenial::Admission(LeaseDenial::UnrelatedNestedLease)
            };
            match refused {
                crate::RetainedRunDenial::Native(crate::RetainedMapDenial::Request(cause)) => {
                    assert_eq!(cause, expected)
                }
                other => panic!("wrong retained request refusal: {other:?}"),
            }
        });
        let expected = if cancelled {
            KernelStop::Cancelled
        } else {
            KernelStop::NestedStopped
        };
        assert_eq!(
            outer.unwrap_err(),
            crate::WorkCeilingDenial::Stopped(expected)
        );
        assert_eq!(entered.load(Ordering::SeqCst), 0);
        assert!(
            !parent.status().is_cancelled(),
            "child controls cannot cancel the parent"
        );
    }
}
#[test]
fn request_scope_reborrows_its_active_descendant_without_widening_policy() {
    let _guard = TEST_LOCK.lock().unwrap();
    let parent = authority().request_lease(request(1, 1500, 100)).unwrap();
    let child = parent.child(request(1, 1200, 20)).unwrap();
    let execution = ExecutionRequest::leased(&parent);
    let (answer, _) = ExecutionWorkCeiling::new(Some(20))
        .run(&child, || {
            execution.in_scope(|lease| {
                let lease = lease.unwrap();
                assert_eq!(lease.policy(), child.policy());
                assert!(!lease.status().is_cancelled());
                7
            })
        })
        .unwrap();
    assert_eq!(answer, Ok(7));
}
#[test]
fn request_scope_never_substitutes_a_sibling_worker() {
    let _guard = TEST_LOCK.lock().unwrap();
    let parent = authority().request_lease(request(1, 1500, 100)).unwrap();
    let first = parent.child(request(1, 1200, 20)).unwrap();
    let sibling = parent.child(request(1, 1100, 10)).unwrap();
    let original = ExecutionRequest::leased(&first);
    let (answer, _) = ExecutionWorkCeiling::new(Some(10))
        .run(&sibling, || {
            original.in_scope(|lease| {
                assert_eq!(lease.unwrap().policy(), first.policy());
                9
            })
        })
        .unwrap();
    assert_eq!(answer, Ok(9));
}

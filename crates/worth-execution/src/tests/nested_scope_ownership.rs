//! A checked topology edit cannot turn a refused reservation into recoverable data.
use super::*;
use crate::{
    Bisection, ComponentPartitioner, PartitionItemId, PartitionUpdateDenial, WeightedItem,
};

#[test]
fn checked_partitioner_reservations_remain_request_owned() {
    let _serial = TEST_LOCK.lock().unwrap();
    for width in [0_usize, 1, 2, 4] {
        for component in [false, true] {
            let parent = authority()
                .request_lease(request(width.max(1), 2_000, 100))
                .unwrap();
            let child = parent.child(request(1, 1_000, 100)).unwrap();
            let mut topology = ComponentPartitioner::new();
            for item in 1..=64 {
                topology.upsert_item(PartitionItemId(item));
            }
            let topology = Mutex::new(topology);
            let bisection = Mutex::new(Bisection::new(1, 0).unwrap());
            let outer = run_checked_batch(
                Some(&parent),
                &batch(&[1], 0),
                if width == 0 {
                    BackendKind::Serial
                } else {
                    BackendKind::Native
                },
                &|_, context| {
                    let edit = if component {
                        topology.lock().unwrap().upsert_item_checked(
                            &child,
                            context,
                            PartitionItemId(1),
                        )
                    } else {
                        bisection.lock().unwrap().upsert_item_checked(
                            &child,
                            context,
                            WeightedItem {
                                item: PartitionItemId(1),
                                weight: 1,
                            },
                        )
                    };
                    assert!(
                        matches!(
                            edit,
                            Err(PartitionUpdateDenial::Admission(
                                LeaseDenial::MemoryExhausted(_)
                            ))
                        ),
                        "actual retained-state reservation: {edit:?}"
                    );
                    assert_eq!(
                        context.take_nested_computation_stop(),
                        None,
                        "a checked edit's refused reservation belongs to its request"
                    );
                    Ok::<_, KernelFailure<()>>(1_u64)
                },
            );
            assert!(matches!(
                outer.stop,
                Some(BatchStop::Failure {
                    cause: KernelFailure::Stop(KernelStop::NestedStopped),
                    ..
                })
            ));
        }
    }
}

#[test]
fn a_canonical_comparison_reservation_remains_request_owned() {
    let _serial = TEST_LOCK.lock().unwrap();
    // A constant stream has no heap input; its declared encoding exceeds the
    // existing process allowance before the first encoding byte is produced.
    struct Stream;
    impl crate::CanonicalBits for Stream {
        fn canonical_len(&self) -> Option<usize> {
            Some(4_096)
        }
        fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
            for _ in 0..64 {
                if !visitor(&[0; 64]) {
                    return false;
                }
            }
            true
        }
    }
    for width in [0_usize, 1, 2, 4] {
        let parent = authority()
            .request_lease(request(width.max(1), 2_000, 100))
            .unwrap();
        let child = parent.child(request(1, 1_000, 100)).unwrap();
        let outer = run_checked_batch(
            Some(&parent),
            &batch(&[1], 0),
            if width == 0 {
                BackendKind::Serial
            } else {
                BackendKind::Native
            },
            &|_, context| {
                assert_eq!(
                    crate::oracle::compare_canonical_values(&child, &Stream, &Stream),
                    Err(crate::oracle::OracleMismatch::Stop)
                );
                assert_eq!(
                    context.take_nested_computation_stop(),
                    None,
                    "the canonical buffer's refused reservation belongs to its request"
                );
                Ok::<_, KernelFailure<()>>(1_u64)
            },
        );
        assert!(matches!(
            outer.stop,
            Some(BatchStop::Failure {
                cause: KernelFailure::Stop(KernelStop::NestedStopped),
                ..
            })
        ));
    }
}

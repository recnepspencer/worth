#[allow(unused_imports)]
pub(crate) use crate::fast::partitioners::*;

#[test]
fn candidate_partition_work_is_charged_by_enclosing_map_lease() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let authority = ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
        max_workers: NonZeroUsize::new(1).unwrap(),
        charged_memory_bytes: Some(8_192),
    })
    .unwrap();
    let map = ExecutionMap::try_from_declared_partitions(
        vec![PartitionIdentity::new(20)],
        vec![MapPartition {
            identity: PartitionIdentity::new(20),
            value: 1_u64,
            read_keys: Vec::<u64>::new(),
            write_keys: Vec::<u64>::new(),
            kernel_scratch_bytes: 256,
            max_result_bytes: 0,
        }],
    )
    .unwrap();
    for (ceiling, accepted) in [(1, false), (2, true)] {
        let lease = authority
            .request_lease(LeaseRequest {
                policy: ExecutionRequestPolicy::new(
                    ExecutionPosture::Serial,
                    DeterminismContract::CanonicalBitwise,
                    ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), Some(8_192), Some(ceiling)),
                ),
                deadline: None,
                cancellation: CancellationToken::new(),
            })
            .unwrap();
        let outcome = map.run(Some(&lease), |value, context| {
            let mut candidate = KeyedPartitioner::new();
            let work = candidate
                .upsert(
                    KeyedItem {
                        item: item(*value),
                        key: 7_u64,
                        partition: PartitionIdentity::new(7),
                    },
                    admit,
                )
                .unwrap();
            work.charge(context)?;
            Ok::<_, MapKernelFailure<()>>(candidate.route(item(*value)).unwrap().value())
        });
        match (accepted, outcome) {
            (true, MapOutcome::Complete { values, report }) => {
                assert_eq!(values, vec![7]);
                assert_eq!(report.charged_work(), 2);
            }
            (
                false,
                MapOutcome::Stopped {
                    reason: MapStop::WorkExhausted { .. },
                    ..
                },
            ) => {}
            (_, other) => panic!(
                "unexpected charged partition outcome: {}",
                other.report().charged_work()
            ),
        }
    }
}

#[allow(unused_imports)]
pub(crate) use {
    crate::partitioners::ordinary_context::admit, crate::partitioners::ordinary_context::item,
    std::num::NonZeroUsize, worth_execution::CancellationToken,
    worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig,
    worth_execution::ExecutionMap, worth_execution::KeyedItem, worth_execution::KeyedPartitioner,
    worth_execution::LeaseRequest, worth_execution::MapKernelFailure, worth_execution::MapOutcome,
    worth_execution::MapPartition, worth_execution::MapStop,
    worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget,
    worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy,
    worth_foundational::PartitionIdentity,
};

#[test]
fn candidate_partition_work_is_charged_by_enclosing_map_lease() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let authority = worth_execution::ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
        max_workers: std::num::NonZeroUsize::new(1).unwrap(),
        charged_memory_bytes: Some(8_192),
    })
    .unwrap();
    let map = worth_execution::ExecutionMap::try_from_declared_partitions(
        vec![worth_foundational::PartitionIdentity::new(20)],
        vec![MapPartition {
            identity: worth_foundational::PartitionIdentity::new(20),
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
                policy: worth_foundational::ExecutionRequestPolicy::new(
                    worth_foundational::ExecutionPosture::Serial,
                    worth_foundational::DeterminismContract::CanonicalBitwise,
                    worth_foundational::ExecutionBudget::new(
                        std::num::NonZeroUsize::new(1).unwrap(),
                        Some(8_192),
                        Some(ceiling),
                    ),
                ),
                deadline: None,
                cancellation: worth_execution::CancellationToken::new(),
            })
            .unwrap();
        let outcome = map.run(Some(&lease), |value, context| {
            let mut candidate = worth_execution::KeyedPartitioner::new();
            let work = candidate
                .upsert(
                    KeyedItem {
                        item: item(*value),
                        key: 7_u64,
                        partition: worth_foundational::PartitionIdentity::new(7),
                    },
                    admit,
                )
                .unwrap();
            work.charge(context)?;
            Ok::<_, MapKernelFailure<()>>(candidate.route(item(*value)).unwrap().value())
        });
        match (accepted, outcome) {
            (true, worth_execution::MapOutcome::Complete { values, report }) => {
                assert_eq!(values, vec![7]);
                assert_eq!(report.charged_work(), 2);
            }
            (
                false,
                worth_execution::MapOutcome::Stopped {
                    reason: worth_execution::MapStop::WorkExhausted { .. },
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

// A reduction interruption crosses the actual decomposition scope wrapper.
use {worth_execution::CancellationToken, worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget, worth_execution::ExecutionMap, worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy, worth_execution::LeaseRequest, worth_execution::MapKernelFailure, worth_execution::MapKernelStop, worth_execution::MapOutcome, worth_execution::MapStop, std::num::NonZeroUsize, worth_foundational::PartitionIdentity, crate::public_nested_regressions::isolated_context::TEST_LOCK, crate::public_nested_regressions::isolated_context::authority, crate::public_nested_regressions::isolated_context::map};
use {worth_execution::CancellationSource, worth_execution::DecomposeFailure, worth_execution::DecomposeKernelEditions, worth_execution::ExecutionDecompose, worth_execution::InteriorResult, worth_execution::KeylessPartition, worth_execution::ReductionRunFailure, worth_execution::ReductionRunStop};

#[test]
fn a_real_decomposition_scope_keeps_the_reduction_interruption_request_owned() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let source = worth_execution::CancellationSource::new();
    let policy = worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Automatic,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, Some(100)),
    );
    let parent = authority()
        .request_lease(LeaseRequest {
            policy,
            deadline: None,
            cancellation: worth_execution::CancellationToken::new(),
        })
        .unwrap();
    let child = parent
        .child(LeaseRequest {
            policy,
            deadline: None,
            cancellation: source.token(),
        })
        .unwrap();
    let input =
        ExecutionMap::<(), u64>::from_keyless_partitions(std::collections::BTreeMap::from([(
            worth_foundational::PartitionIdentity::new(1),
            KeylessPartition {
                value: (),
                kernel_scratch_bytes: 0,
                max_result_bytes: 0,
            },
        )]))
        .unwrap();
    let outer = map(1).run(Some(&parent), |value, context| {
        let mut decomposition = ExecutionDecompose::<(), (), (), (), (), _>::try_new(
            vec![worth_foundational::PartitionIdentity::new(1)],
            (),
            |_, _| source.cancel(),
            0,
            0,
            0,
            0,
            0,
        )
        .unwrap();
        let stopped = decomposition.run(
            Some(&child),
            &input,
            DecomposeKernelEditions {
                interior: 1,
                interface: 1,
                back: 1,
            },
            |_, _| {
                Ok::<_, MapKernelFailure<()>>(InteriorResult {
                    interior: (),
                    contribution: (),
                })
            },
            |_, _| panic!("the reduction must stop before the interface"),
            |_, _| panic!("the reduction must stop before back substitution"),
        );
        assert!(matches!(
            stopped,
            Err(worth_execution::DecomposeRunFailure {
                cause: worth_execution::DecomposeFailure::ReductionRun(ReductionRunFailure {
                    reason: worth_execution::ReductionRunStop::Hook(worth_execution::MapKernelStop::Cancelled),
                    ..
                }),
                ..
            })
        ));
        assert_eq!(context.take_nested_computation_stop(), None);
        Ok::<_, MapKernelFailure<()>>(*value)
    });
    assert!(matches!(
        outer,
        worth_execution::MapOutcome::Stopped {
            reason: worth_execution::MapStop::Failure {
                cause: worth_execution::MapKernelFailure::Stop(worth_execution::MapKernelStop::NestedStopped),
                ..
            },
            ..
        }
    ));
}

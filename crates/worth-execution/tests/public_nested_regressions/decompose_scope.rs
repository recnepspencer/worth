//! A reduction interruption crosses the actual decomposition scope wrapper.
use super::*;
use worth_execution::{
    CancellationSource, DecomposeFailure, DecomposeKernelEditions, ExecutionDecompose,
    InteriorResult, KeylessPartition, ReductionRunFailure, ReductionRunStop,
};

#[test]
fn a_real_decomposition_scope_keeps_the_reduction_interruption_request_owned() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let source = CancellationSource::new();
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Automatic,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::MIN, None, Some(100)),
    );
    let parent = authority()
        .request_lease(LeaseRequest {
            policy,
            deadline: None,
            cancellation: CancellationToken::new(),
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
            PartitionIdentity::new(1),
            KeylessPartition {
                value: (),
                kernel_scratch_bytes: 0,
                max_result_bytes: 0,
            },
        )]))
        .unwrap();
    let outer = map(1).run(Some(&parent), |value, context| {
        let mut decomposition = ExecutionDecompose::<(), (), (), (), (), _>::try_new(
            vec![PartitionIdentity::new(1)],
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
                cause: DecomposeFailure::ReductionRun(ReductionRunFailure {
                    reason: ReductionRunStop::Hook(MapKernelStop::Cancelled),
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
        MapOutcome::Stopped {
            reason: MapStop::Failure {
                cause: MapKernelFailure::Stop(MapKernelStop::NestedStopped),
                ..
            },
            ..
        }
    ));
}

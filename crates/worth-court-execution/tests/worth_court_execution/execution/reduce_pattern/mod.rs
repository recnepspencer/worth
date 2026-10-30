#[allow(unused_imports)]
use crate::reduce_pattern::ordinary_context::*;
#[allow(unused_imports)]
pub(crate) use crate::test_serialization;

#[allow(unused_imports)]
pub(crate) use crate::reducer_overlap;
pub(crate) use {
    std::num::NonZeroUsize, std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering,
    std::sync::Mutex, std::sync::OnceLock,
};

pub(crate) use {
    worth_execution::CancellationToken, worth_execution::CanonicalBits,
    worth_execution::ChargedBytes, worth_execution::ExecutionAuthority,
    worth_execution::ExecutionAuthorityConfig, worth_execution::ExecutionMap,
    worth_execution::LeaseRequest, worth_execution::MapKernelFailure, worth_execution::MapOutcome,
    worth_execution::MapPartition, worth_execution::ReduceInputDenial,
    worth_execution::ReductionMetrics, worth_execution::ReductionRunFailure,
    worth_execution::ReductionRunStop,
};
pub(crate) use {
    worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget,
    worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy,
    worth_foundational::PartitionIdentity,
};

#[test]
fn no_lease_nested_reduction_charges_its_enclosing_map() {
    let _guard = crate::test_serialization::guard(&TEST_LOCK);
    let (_, inner_report, _) = map().run_reduce(None, mapped, 0, sum, 8, 0).unwrap();
    let outer = map_with_count(1).run(None, |_, context| {
        context.checkpoint(1)?;
        let (_, report, _) = map()
            .run_reduce(None, mapped, 0, sum, 8, 0)
            .map_err(|_| worth_execution::MapKernelFailure::Domain(()))?;
        Ok::<_, MapKernelFailure<()>>(report.charged_work())
    });
    match outer {
        worth_execution::MapOutcome::Complete { values, report } => {
            assert_eq!(values, vec![inner_report.charged_work()]);
            assert_eq!(report.charged_work(), inner_report.charged_work() + 1);
        }
        worth_execution::MapOutcome::Stopped { .. } => {
            panic!("nested serial reduction must complete")
        }
    }
}

#[test]
fn no_lease_reduction_contains_clone_panic() {
    let _guard = crate::test_serialization::guard(&TEST_LOCK);
    let caught = std::panic::catch_unwind(|| {
        map().run_reduce(
            None,
            |value, _| Ok::<_, MapKernelFailure<()>>(PanickingClone(*value)),
            PanickingClone(0),
            |_, _| PanickingClone(0),
            8,
            0,
        )
    });
    assert!(matches!(
        caught,
        Ok(Err(worth_execution::ReduceInputDenial::ReductionStopped {
            failure: ReductionRunFailure {
                reason: worth_execution::ReductionRunStop::Panic,
                ..
            },
            ..
        }))
    ));
}

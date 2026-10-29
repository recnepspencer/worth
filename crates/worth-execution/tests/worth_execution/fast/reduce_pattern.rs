#[allow(unused_imports)]
pub(crate) use crate::test_serialization;

#[allow(unused_imports)]
pub(crate) use crate::reducer_overlap;
pub(crate) use std::{
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex, OnceLock,
    },
};

pub(crate) use worth_execution::{
    CancellationToken, CanonicalBits, ChargedBytes, ExecutionAuthority, ExecutionAuthorityConfig,
    ExecutionMap, LeaseRequest, MapKernelFailure, MapOutcome, MapPartition, ReduceInputDenial,
    ReductionMetrics, ReductionRunFailure, ReductionRunStop,
};
pub(crate) use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    PartitionIdentity,
};

pub(crate) static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();
pub(crate) static TEST_LOCK: Mutex<()> = Mutex::new(());

pub(crate) fn authority() -> &'static ExecutionAuthority {
    AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(2).unwrap(),
            charged_memory_bytes: Some(1 << 20),
        })
        .unwrap()
    })
}

pub(crate) fn lease(work: u64) -> worth_execution::ExecutionResourceLease<'static> {
    lease_with_workers(work, 2)
}

pub(crate) fn lease_with_workers(
    work: u64,
    workers: usize,
) -> worth_execution::ExecutionResourceLease<'static> {
    authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(
                    NonZeroUsize::new(workers).unwrap(),
                    Some(1 << 20),
                    Some(work),
                ),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap()
}

pub(crate) fn map() -> ExecutionMap<u64, u64> {
    map_with_count(3)
}

pub(crate) fn map_with_count(count: u64) -> ExecutionMap<u64, u64> {
    let ids: Vec<_> = (1..=count).map(PartitionIdentity::new).collect();
    let partitions = ids
        .iter()
        .copied()
        .enumerate()
        .map(|(index, identity)| MapPartition {
            identity,
            value: index as u64 + 1,
            read_keys: Vec::new(),
            write_keys: Vec::new(),
            kernel_scratch_bytes: 0,
            max_result_bytes: 0,
        })
        .collect();
    ExecutionMap::try_from_declared_partitions(ids, partitions).unwrap()
}

pub(crate) fn mapped(
    value: &u64,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<u64, MapKernelFailure<()>> {
    context.checkpoint(1)?;
    Ok(*value)
}

pub(crate) fn sum(left: &u64, right: &u64) -> u64 {
    left + right
}

#[test]
fn no_lease_nested_reduction_charges_its_enclosing_map() {
    let _guard = test_serialization::guard(&TEST_LOCK);
    let (_, inner_report, _) = map().run_reduce(None, mapped, 0, sum, 8, 0).unwrap();
    let outer = map_with_count(1).run(None, |_, context| {
        context.checkpoint(1)?;
        let (_, report, _) = map()
            .run_reduce(None, mapped, 0, sum, 8, 0)
            .map_err(|_| MapKernelFailure::Domain(()))?;
        Ok::<_, MapKernelFailure<()>>(report.charged_work())
    });
    match outer {
        MapOutcome::Complete { values, report } => {
            assert_eq!(values, vec![inner_report.charged_work()]);
            assert_eq!(report.charged_work(), inner_report.charged_work() + 1);
        }
        MapOutcome::Stopped { .. } => panic!("nested serial reduction must complete"),
    }
}

pub(crate) struct PanickingClone(u64);

impl Clone for PanickingClone {
    fn clone(&self) -> Self {
        panic!("reduction clone panic")
    }
}

impl ChargedBytes for PanickingClone {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

impl CanonicalBits for PanickingClone {
    fn canonical_len(&self) -> Option<usize> {
        Some(8)
    }

    fn visit_canonical_bits(&self, visit: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        visit(&self.0.to_le_bytes())
    }
}

#[test]
fn no_lease_reduction_contains_clone_panic() {
    let _guard = test_serialization::guard(&TEST_LOCK);
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
        Ok(Err(ReduceInputDenial::ReductionStopped {
            failure: ReductionRunFailure {
                reason: ReductionRunStop::Panic,
                ..
            },
            ..
        }))
    ));
}

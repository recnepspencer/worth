use super::model::{combine, ResidentValue};
use std::{
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicUsize, Ordering},
        OnceLock,
    },
};
use worth_execution::{
    CancellationToken, ChargedBytes, ExecutionAuthority, ExecutionAuthorityConfig, ExecutionMap,
    ExecutionResourceLease, ExecutionWorkCeiling, LeaseDenial, LeaseRequest, MapKernelFailure,
    MapOutcome, MapPartition, MapStop,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    PartitionIdentity,
};

const BUDGET: u64 = 1 << 20;

fn lease() -> ExecutionResourceLease<'static> {
    static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();
    AUTHORITY
        .get_or_init(|| {
            ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
                max_workers: NonZeroUsize::new(2).unwrap(),
                charged_memory_bytes: Some(1 << 28),
            })
            .unwrap()
        })
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(2).unwrap(), Some(BUDGET), Some(u64::MAX)),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap()
}

fn map(value: ResidentValue, cap: u64) -> ExecutionMap<ResidentValue, u64> {
    ExecutionMap::try_from_declared_partitions(
        vec![PartitionIdentity::new(1)],
        vec![MapPartition {
            identity: PartitionIdentity::new(1),
            value,
            read_keys: Vec::new(),
            write_keys: Vec::new(),
            kernel_scratch_bytes: 0,
            max_result_bytes: cap,
        }],
    )
    .unwrap()
}

#[test]
fn derived_input_storage_refuses_before_kernel_and_oversized_output_refuses_publication() {
    let lease = lease();
    let entered = AtomicUsize::new(0);
    let input = map(ResidentValue::with_notes(2 << 20), 64);
    match input.run(Some(&lease), |_, _| {
        entered.fetch_add(1, Ordering::SeqCst);
        Ok::<_, MapKernelFailure<()>>(0_u64)
    }) {
        MapOutcome::Stopped {
            completed_prefix,
            reason: MapStop::Admission(LeaseDenial::MemoryExhausted(denial)),
            ..
        } => {
            assert!(completed_prefix.is_empty());
            assert!(
                matches!(denial, worth_execution::MemoryLimitDenial::Capacity { requested, admitted, .. } if requested > admitted)
            );
        }
        _ => panic!("nested input allocation must refuse admission"),
    }
    assert_eq!(entered.load(Ordering::SeqCst), 0);
    let input = map(ResidentValue::empty(), 64);
    match input.run(Some(&lease), |_, context| {
        context.checkpoint(1)?;
        Ok::<_, MapKernelFailure<()>>(ResidentValue::with_notes(4096))
    }) {
        MapOutcome::Stopped {
            completed_prefix,
            reason:
                MapStop::Failure {
                    cause: MapKernelFailure::ResultCapacityExceeded,
                    ..
                },
            ..
        } => assert!(completed_prefix.is_empty()),
        _ => panic!("resident output exceeds its declared cap"),
    }
}

#[test]
fn funded_nested_work_stops_and_fresh_retry_hands_tree_charge_to_callers_hold() {
    let lease = lease();
    let mut value = ResidentValue::with_notes(128);
    value.labels = ["north", "south", "east"]
        .into_iter()
        .map(String::from)
        .collect();
    let input = map(value, 2048);
    let completed_units = AtomicUsize::new(0);
    let kernel = |value: &ResidentValue,
                  context: &mut worth_execution::MapKernelContext<'_, '_>| {
        let mut total = 0;
        for label in &value.labels {
            context.checkpoint(1)?;
            total += label.len() as u64;
            completed_units.fetch_add(1, Ordering::SeqCst);
        }
        let mut result = value.clone();
        result.total = total;
        Ok::<_, MapKernelFailure<()>>(result)
    };
    let (stopped, _) = ExecutionWorkCeiling::new(Some(2))
        .run(&lease, || input.run(Some(&lease), kernel))
        .unwrap();
    assert!(
        matches!(stopped, MapOutcome::Stopped { ref completed_prefix, reason: MapStop::WorkExhausted { .. }, .. } if completed_prefix.is_empty())
    );
    let finished = completed_units.load(Ordering::SeqCst);
    assert!(
        finished > 0 && finished < 3,
        "funding stops inside the loop"
    );

    let mut held = lease.reserve_memory(0).unwrap();
    let (tree, _, _) = input
        .run_reduce_holding(
            Some(&lease),
            None,
            Some(&mut held),
            kernel,
            ResidentValue::empty(),
            combine,
            2048,
            0,
        )
        .expect("fresh funded run completes");
    assert_eq!(tree.result().total, 14);
    assert_eq!(tree.result().labels, ["north", "south", "east"]);
    let expected = tree.additional_charged_bytes() + std::mem::size_of_val(&tree) as u64;
    assert_eq!(held.bytes(), expected);
    assert!(expected > 128);
    let denial = lease.reserve_memory(BUDGET).unwrap_err();
    assert!(
        matches!(denial, worth_execution::MemoryLimitDenial::Capacity { admitted, .. } if admitted == BUDGET - held.bytes())
    );
    drop(tree);
    drop(held);
    assert_eq!(lease.reserve_memory(BUDGET).unwrap().bytes(), BUDGET);
}

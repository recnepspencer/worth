#[allow(unused_imports)] use crate::charged_bytes_derive::isolated_context::admission::*;
pub(crate) use {crate::charged_bytes_derive::ordinary_context::model::combine, crate::charged_bytes_derive::ordinary_context::model::ResidentValue};
pub(crate) use {std::num::NonZeroUsize, std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering, std::sync::OnceLock};
pub(crate) use {worth_execution::CancellationToken, worth_execution::ChargedBytes, worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig, worth_execution::ExecutionMap, worth_execution::ExecutionResourceLease, worth_execution::ExecutionWorkCeiling, worth_execution::LeaseDenial, worth_execution::LeaseRequest, worth_execution::MapKernelFailure, worth_execution::MapOutcome, worth_execution::MapPartition, worth_execution::MapStop};
pub(crate) use {worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget, worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy, worth_foundational::PartitionIdentity};

#[test]
fn derived_input_storage_refuses_before_kernel_and_oversized_output_refuses_publication() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let lease = lease();
    let entered = std::sync::atomic::AtomicUsize::new(0);
    let input = map(crate::charged_bytes_derive::ordinary_context::model::ResidentValue::with_notes(2 << 20), 64);
    match input.run(Some(&lease), |_, _| {
        entered.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok::<_, MapKernelFailure<()>>(0_u64)
    }) {
        worth_execution::MapOutcome::Stopped {
            completed_prefix,
            reason: worth_execution::MapStop::Admission(worth_execution::LeaseDenial::MemoryExhausted(denial)),
            ..
        } => {
            assert!(completed_prefix.is_empty());
            assert!(
                matches!(denial, worth_execution::MemoryLimitDenial::Capacity { requested, admitted, .. } if requested > admitted)
            );
        }
        _ => panic!("nested input allocation must refuse admission"),
    }
    assert_eq!(entered.load(std::sync::atomic::Ordering::SeqCst), 0);
    let input = map(crate::charged_bytes_derive::ordinary_context::model::ResidentValue::empty(), 64);
    match input.run(Some(&lease), |_, context| {
        context.checkpoint(1)?;
        Ok::<_, MapKernelFailure<()>>(crate::charged_bytes_derive::ordinary_context::model::ResidentValue::with_notes(4096))
    }) {
        worth_execution::MapOutcome::Stopped {
            completed_prefix,
            reason:
                worth_execution::MapStop::Failure {
                    cause: worth_execution::MapKernelFailure::ResultCapacityExceeded,
                    ..
                },
            ..
        } => assert!(completed_prefix.is_empty()),
        _ => panic!("resident output exceeds its declared cap"),
    }
}

#[test]
fn funded_nested_work_stops_and_fresh_retry_hands_tree_charge_to_callers_hold() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let lease = lease();
    let mut value = crate::charged_bytes_derive::ordinary_context::model::ResidentValue::with_notes(128);
    value.labels = ["north", "south", "east"]
        .into_iter()
        .map(String::from)
        .collect();
    let input = map(value, 2048);
    let completed_units = std::sync::atomic::AtomicUsize::new(0);
    let kernel = |value: &ResidentValue,
                  context: &mut worth_execution::MapKernelContext<'_, '_>| {
        let mut total = 0;
        for label in &value.labels {
            context.checkpoint(1)?;
            total += label.len() as u64;
            completed_units.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
        let mut result = value.clone();
        result.total = total;
        Ok::<_, MapKernelFailure<()>>(result)
    };
    let (stopped, _) = worth_execution::ExecutionWorkCeiling::new(Some(2))
        .run(&lease, || input.run(Some(&lease), kernel))
        .unwrap();
    assert!(
        matches!(stopped, worth_execution::MapOutcome::Stopped { ref completed_prefix, reason: worth_execution::MapStop::WorkExhausted { .. }, .. } if completed_prefix.is_empty())
    );
    let finished = completed_units.load(std::sync::atomic::Ordering::SeqCst);
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
            crate::charged_bytes_derive::ordinary_context::model::ResidentValue::empty(),
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

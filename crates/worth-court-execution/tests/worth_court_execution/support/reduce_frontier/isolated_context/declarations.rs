#[allow(unused_imports)]
pub(crate) use {crate::test_serialization};

#[allow(unused_imports)]
pub(crate) use {crate::reducer_overlap};
pub(crate) use {std::num::NonZeroUsize, std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering, std::sync::Mutex, std::sync::OnceLock};

pub(crate) use {worth_execution::CancellationToken, worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig, worth_execution::ExecutionMap, worth_execution::LeaseRequest, worth_execution::MapKernelFailure, worth_execution::MapPartition, worth_execution::ReduceInputDenial, worth_execution::ReductionPlan, worth_execution::ReductionRunStop, worth_execution::ReductionTree};
pub(crate) use {worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget, worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy, worth_foundational::PartitionIdentity};

pub(crate) static AUTHORITY: OnceLock<ExecutionAuthority> = std::sync::OnceLock::new();
pub(crate) static TEST_LOCK: Mutex<()> = std::sync::Mutex::new(());

pub(crate) fn authority() -> &'static ExecutionAuthority {
    AUTHORITY.get_or_init(|| {
        worth_execution::ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: std::num::NonZeroUsize::new(4).unwrap(),
            charged_memory_bytes: Some(8 << 20),
        })
        .unwrap()
    })
}

pub(crate) fn lease(workers: usize) -> worth_execution::ExecutionResourceLease<'static> {
    lease_with_ceiling(workers, 10_000)
}

pub(crate) fn lease_with_ceiling(
    workers: usize,
    work_ceiling: u64,
) -> worth_execution::ExecutionResourceLease<'static> {
    authority()
        .request_lease(LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Automatic,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    std::num::NonZeroUsize::new(workers).unwrap(),
                    None,
                    Some(work_ceiling),
                ),
            ),
            deadline: None,
            cancellation: worth_execution::CancellationToken::new(),
        })
        .unwrap()
}

pub(crate) fn map(count: u64) -> ExecutionMap<u64, u64> {
    let ids: Vec<_> = (1..=count).map(worth_foundational::PartitionIdentity::new).collect();
    let partitions = ids
        .iter()
        .copied()
        .map(|identity| MapPartition {
            identity,
            value: identity.value(),
            read_keys: Vec::new(),
            write_keys: Vec::new(),
            kernel_scratch_bytes: 0,
            max_result_bytes: 1,
        })
        .collect();
    worth_execution::ExecutionMap::try_from_declared_partitions(ids, partitions).unwrap()
}

pub(crate) fn mapped_bytes(
    value: &u64,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<Vec<u8>, MapKernelFailure<()>> {
    context.checkpoint(1)?;
    Ok(vec![*value as u8])
}

pub(crate) fn competing_failures(left: &Vec<u8>, right: &Vec<u8>) -> Vec<u8> {
    if left.is_empty() && right.as_slice() == [4] {
        panic!("later canonical task");
    }
    if left.as_slice() == [1] && right.as_slice() == [2] {
        return vec![9; 64];
    }
    let mut joined = left.clone();
    joined.extend(right);
    joined
}

pub(crate) fn mapped(
    value: &u64,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<f64, MapKernelFailure<()>> {
    context.checkpoint(1)?;
    Ok((*value as f64).sin())
}

pub(crate) fn sum(left: &f64, right: &f64) -> f64 {
    left + right
}

pub(crate) fn mapped_u64(
    value: &u64,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<u64, MapKernelFailure<()>> {
    context.checkpoint(1)?;
    Ok(*value)
}

pub(crate) fn sum_u64(left: &u64, right: &u64) -> u64 {
    left + right
}

pub(crate) fn prove_frontier(count: u64, workers: usize, expected_overlap: usize) {
    let input = map(count);
    let (oracle, oracle_report, oracle_metrics) = input
        .run_reduce(Some(&lease(1)), mapped, 0.0, sum, 8, 0)
        .unwrap();
    let active = std::sync::atomic::AtomicUsize::new(0);
    let peak = std::sync::atomic::AtomicUsize::new(0);
    let overlap = crate::reducer_overlap::ReducerOverlap::new(expected_overlap);
    let combine = |left: &f64, right: &f64| {
        let now = active.fetch_add(1, std::sync::atomic::Ordering::AcqRel) + 1;
        peak.fetch_max(now, std::sync::atomic::Ordering::AcqRel);
        overlap.rendezvous();
        active.fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
        left + right
    };
    let (native, report, metrics) = input
        .run_reduce(Some(&lease(workers)), mapped, 0.0, combine, 8, 0)
        .unwrap();
    assert_eq!(native.result().to_bits(), oracle.result().to_bits());
    assert_eq!(metrics, oracle_metrics);
    assert_eq!(report.charged_work(), oracle_report.charged_work());
    assert_eq!(report.charged_span(), oracle_report.charged_span());
    assert!(
        peak.load(std::sync::atomic::Ordering::Acquire) >= expected_overlap,
        "expected {expected_overlap} overlapping reducer calls"
    );
}

pub(crate) fn assert_ceiling_width_invariant(input: &ExecutionMap<u64, u64>, count: u64, ceiling: u64) {
    let run = |workers| {
        input.run_reduce(
            Some(&lease_with_ceiling(workers, ceiling)),
            mapped_u64,
            0_u64,
            sum_u64,
            8,
            0,
        )
    };
    let serial = run(1);
    let native = run(4);
    match (serial, native) {
        (
            Err(worth_execution::ReduceInputDenial::ReductionStopped {
                failure: left,
                report: left_report,
            }),
            Err(worth_execution::ReduceInputDenial::ReductionStopped {
                failure: right,
                report: right_report,
            }),
        ) => {
            assert_eq!(
                left.reason,
                worth_execution::ReductionRunStop::Hook(worth_execution::MapKernelStop::WorkCeiling),
                "count {count}, ceiling {ceiling}"
            );
            assert_eq!(
                left_report.charged_work(),
                ceiling,
                "count {count}, ceiling {ceiling}"
            );
            let accounting = left
                .metrics
                .accounted_metrics()
                .expect("a serial reduction stop carries accounting");
            assert_eq!(
                accounting.charged_work + count,
                ceiling,
                "count {count}, ceiling {ceiling}"
            );
            assert_eq!(
                accounting.charged_span, accounting.charged_work,
                "count {count}, ceiling {ceiling}"
            );
            assert_eq!(
                right.reason, left.reason,
                "count {count}, ceiling {ceiling}"
            );
            assert_eq!(
                right.metrics, left.metrics,
                "count {count}, ceiling {ceiling}"
            );
            assert_eq!(
                right_report.charged_work(),
                left_report.charged_work(),
                "count {count}, ceiling {ceiling}"
            );
            assert_eq!(
                right_report.charged_span(),
                left_report.charged_span(),
                "count {count}, ceiling {ceiling}"
            );
        }
        (Ok((left, left_report, left_metrics)), Ok((right, right_report, right_metrics))) => {
            assert_eq!(
                right.result(),
                left.result(),
                "count {count}, ceiling {ceiling}"
            );
            assert_eq!(
                right_metrics, left_metrics,
                "count {count}, ceiling {ceiling}"
            );
            assert_eq!(
                right_report.charged_work(),
                left_report.charged_work(),
                "count {count}, ceiling {ceiling}"
            );
            assert_eq!(
                right_report.charged_span(),
                left_report.charged_span(),
                "count {count}, ceiling {ceiling}"
            );
        }
        _ => panic!("width divergence at count {count}, ceiling {ceiling}"),
    }
}
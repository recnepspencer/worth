use super::*;
use crate::{
    ExecutionMap, ExecutionRequest, ExecutionWorkCeiling, KeylessPartition, MapOutcome,
    RetainedMapInput, SerialMemoryBudget, SerialRequest,
};
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Value(u64);
impl ChargedBytes for Value {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

fn serial() -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(100_000),
        crate::CancellationToken::new(),
        None,
    )
}
fn map() -> ExecutionMap<u64, u64> {
    ExecutionMap::from_keyless_partitions(BTreeMap::from([(
        PartitionIdentity::new(1),
        KeylessPartition {
            value: 7,
            kernel_scratch_bytes: 0,
            max_result_bytes: 0,
        },
    )]))
    .unwrap()
}

#[test]
fn carried_input_keeps_the_same_nonclone_result_cell_and_changed_input_evaluates() {
    let serial = serial();
    let source = map();
    let MapOutcome::Complete { mut values, .. } = source
        .run_retained(
            ExecutionRequest::serial(&serial),
            ExecutionWorkCeiling::new(100),
            |input, context| {
                context.checkpoint(3)?;
                Ok::<_, KernelFailure<()>>(Value(*input * 10))
            },
        )
        .unwrap()
    else {
        panic!("native complete")
    };
    let prior = values.pop().unwrap();
    drop(values);
    let planned = ExecutionMap::<_, u64>::from_keyless_partitions(BTreeMap::from([
        (
            PartitionIdentity::new(1),
            KeylessPartition {
                value: RetainedMapInput::Carry(prior.clone()),
                kernel_scratch_bytes: 0,
                max_result_bytes: prior.additional_charged_bytes(),
            },
        ),
        (
            PartitionIdentity::new(2),
            KeylessPartition {
                value: RetainedMapInput::Evaluate(9_u64),
                kernel_scratch_bytes: 0,
                max_result_bytes: 0,
            },
        ),
    ]))
    .unwrap();
    let entered = AtomicUsize::new(0);
    let MapOutcome::Complete { values, report } = planned
        .run_retained_inputs(
            ExecutionRequest::serial(&serial),
            ExecutionWorkCeiling::new(100),
            None,
            |input, context| {
                entered.fetch_add(1, Ordering::Relaxed);
                context.checkpoint(5)?;
                Ok::<_, KernelFailure<()>>(Value(*input * 10))
            },
        )
        .unwrap()
    else {
        panic!("mixed native complete")
    };
    assert_eq!(entered.load(Ordering::Relaxed), 1);
    assert_eq!((values[0].value().0, values[1].value().0), (70, 90));
    assert!(
        std::ptr::eq(values[0].value(), prior.value()),
        "no R clone or replacement allocation for carried result"
    );
    assert_eq!((report.charged_work(), report.charged_span()), (8, 5));
}

#[test]
fn unqualified_or_misbound_native_result_refuses_before_any_new_kernel() {
    let serial = serial();
    let source = map();
    let MapOutcome::Complete { mut values, .. } = source
        .run_retained(
            ExecutionRequest::serial(&serial),
            ExecutionWorkCeiling::new(100),
            |_, context| Ok::<_, KernelFailure<()>>(context.remaining_work()),
        )
        .unwrap()
    else {
        panic!("context-dependent success")
    };
    let prior = values.pop().unwrap();
    let MapOutcome::Complete { mut values, .. } = source
        .run_retained(
            ExecutionRequest::serial(&serial),
            ExecutionWorkCeiling::new(100),
            |input, context| {
                context.checkpoint(1)?;
                Ok::<_, KernelFailure<()>>(*input)
            },
        )
        .unwrap()
    else {
        panic!("qualified native result")
    };
    let qualified = values.pop().unwrap();
    assert!(qualified.native_reuse_available());
    for (identity, prior) in [(1, prior), (2, qualified)] {
        let planned = ExecutionMap::<_, u64>::from_keyless_partitions(BTreeMap::from([
            (
                PartitionIdentity::new(0),
                KeylessPartition {
                    value: RetainedMapInput::Evaluate(1_u64),
                    kernel_scratch_bytes: 0,
                    max_result_bytes: 0,
                },
            ),
            (
                PartitionIdentity::new(identity),
                KeylessPartition {
                    value: RetainedMapInput::Carry(prior.clone()),
                    kernel_scratch_bytes: 0,
                    max_result_bytes: prior.additional_charged_bytes(),
                },
            ),
        ]))
        .unwrap();
        let entered = AtomicUsize::new(0);
        let denied = planned.run_retained_inputs(
            ExecutionRequest::serial(&serial),
            ExecutionWorkCeiling::new(100),
            None,
            |input, _| {
                entered.fetch_add(1, Ordering::Relaxed);
                Ok::<_, KernelFailure<()>>(*input)
            },
        );
        assert!(
            matches!(denied,Err(crate::RetainedMapDenial::RequiresExecution(id)) if id==PartitionIdentity::new(identity))
        );
        assert_eq!(entered.load(Ordering::Relaxed), 0);
    }
    let retry = source
        .run_retained(
            ExecutionRequest::serial(&serial),
            ExecutionWorkCeiling::new(100),
            |input, context| {
                context.checkpoint(1)?;
                Ok::<_, KernelFailure<()>>(*input)
            },
        )
        .unwrap();
    assert!(
        matches!(retry, MapOutcome::Complete { .. }),
        "explicit actual execution fallback"
    );
}

#[test]
fn retained_reduction_matches_the_full_native_tree_and_refusal_keeps_no_successful_run() {
    let serial = serial();
    let source = map();
    let kernel = |input: &u64, context: &mut KernelContext<'_, '_>| {
        context.checkpoint(3)?;
        Ok::<_, KernelFailure<()>>(*input * 10)
    };
    let (expected, expected_report, expected_metrics) = ExecutionWorkCeiling::new(100)
        .run_serial(&serial, || {
            source
                .run_reduce(None, kernel, 0_u64, |a, b| a + b, 1_000, 0)
                .unwrap()
        })
        .unwrap()
        .0;
    let actual = source.run_reduce_retained_holding(
        ExecutionRequest::serial(&serial),
        ExecutionWorkCeiling::new(100),
        None,
        None,
        kernel,
        0_u64,
        |a, b| a + b,
        1_000,
        0,
    );
    let actual = actual.result.unwrap();
    assert_eq!(*actual.tree.result(), *expected.result());
    assert_eq!(
        (
            actual.report.charged_work(),
            actual.report.charged_span(),
            actual.metrics
        ),
        (
            expected_report.charged_work(),
            expected_report.charged_span(),
            expected_metrics
        )
    );
    let run = actual.run.unwrap();
    assert_eq!(*run.result(PartitionIdentity::new(1)).unwrap().value(), 70);
    assert_eq!(run.charged_work(), 3);
    let advanced = run
        .advance(
            ExecutionRequest::serial(&serial),
            ExecutionWorkCeiling::new(100),
            BTreeMap::new(),
            worth_proof::CanonicalUniqueVec::try_from_sorted_unique(Vec::new()).unwrap(),
            |_: &u64, _| -> Result<u64, KernelFailure<()>> { panic!("unchanged native run") },
        )
        .unwrap();
    let crate::RetainedRunOutcome::Complete { run, report, .. } = advanced else {
        panic!("native complete")
    };
    assert_eq!(*run.result(PartitionIdentity::new(1)).unwrap().value(), 70);
    assert_eq!(report.charged_work(), 3);
    let stopped = source.run_reduce_retained_holding(
        ExecutionRequest::serial(&serial),
        ExecutionWorkCeiling::new(3),
        None,
        None,
        kernel,
        0_u64,
        |a, b| a + b,
        1_000,
        0,
    );
    assert!(matches!(
        stopped.result,
        Err(crate::RetainedReduceDenial::Reduction(_))
    ));
    assert_eq!(
        stopped.mapped_report.unwrap().charged_work(),
        3,
        "real successful map precedes failed canonical tree"
    );
}

#[test]
fn surviving_result_uses_the_same_deep_bound_checked_by_native_evaluation() {
    struct Measured<'a> {
        bytes: String,
        measurements: &'a AtomicUsize,
    }
    impl ChargedBytes for Measured<'_> {
        fn additional_charged_bytes(&self) -> u64 {
            self.measurements.fetch_add(1, Ordering::Relaxed);
            self.bytes.capacity() as u64
        }
    }
    let serial = serial();
    let measurements = AtomicUsize::new(0);
    let source = ExecutionMap::<_, u64>::from_keyless_partitions(BTreeMap::from([(
        PartitionIdentity::new(1),
        KeylessPartition {
            value: (),
            kernel_scratch_bytes: 0,
            max_result_bytes: 4_000,
        },
    )]))
    .unwrap();
    let MapOutcome::Complete { values, .. } = source
        .run_retained(
            ExecutionRequest::serial(&serial),
            ExecutionWorkCeiling::new(100),
            |_, context| {
                context.checkpoint(1)?;
                Ok::<_, KernelFailure<()>>(Measured {
                    bytes: "x".repeat(4_000),
                    measurements: &measurements,
                })
            },
        )
        .unwrap()
    else {
        panic!("funded native deep result")
    };
    assert_eq!(
        measurements.load(Ordering::Relaxed),
        1,
        "surviving custody consumes original capacity examination"
    );
    assert_eq!(values[0].value().bytes.len(), 4_000);
    assert!(values[0].additional_charged_bytes() > 4_000);
    assert_eq!(measurements.load(Ordering::Relaxed), 1);
}

#[test]
fn retained_reduction_with_access_keys_completes_without_a_keyless_advance_capability() {
    let serial = serial();
    let source = ExecutionMap::try_from_declared_partitions(
        vec![PartitionIdentity::new(1)],
        vec![crate::MapPartition {
            identity: PartitionIdentity::new(1),
            value: 7_u64,
            read_keys: vec![99_u64],
            write_keys: Vec::new(),
            kernel_scratch_bytes: 0,
            max_result_bytes: 0,
        }],
    )
    .unwrap();
    let completed = source
        .run_reduce_retained_holding(
            ExecutionRequest::serial(&serial),
            ExecutionWorkCeiling::new(100),
            None,
            None,
            |value, context| {
                context.checkpoint(3)?;
                Ok::<_, KernelFailure<()>>(*value * 10)
            },
            0_u64,
            |a, b| a + b,
            1_000,
            0,
        )
        .result
        .unwrap();
    assert_eq!(*completed.tree.result(), 70);
    assert!(
        completed.run.is_none(),
        "declared keys need full access checking"
    );
}

//! Fallback causes at their producer, including arithmetic that rebuilding survives.

use worth_execution::{
    CanonicalBits, ChargedBytes, ReductionDenial, ReductionRunFailure, ReductionRunStop,
};

use super::*;
use crate::domain_computation::primary_graph::application_contribution::request_execution::test_policy;
use crate::domain_computation::primary_graph::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;

#[test]
fn edit_capacity_overflow_rebuilds_and_matches_reuse_off() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let source = template();
    let keys: Vec<_> = (0..32).map(Id::new).collect();
    let retained = retained(&source, &keys);
    // Full construction needs 6P+6 value caps; cumulative edits need at least
    // 8(sum(depth)+P)+6P. For 32 binary nodes, sum(depth)>=135, so the edit
    // coefficient exceeds 1024 while the build coefficient is only 198.
    const DECLARED: u64 = u64::MAX / 1024;
    let request = live_scope();
    let execution = QueryRequestExecution::open(
        RuntimeWorldExecutionPlacement::Serial(test_policy(std::num::NonZeroUsize::MIN, None)),
        &request,
    );
    let reducer = WorthQueryDeterministicReducer::canonical(|| 0, counted);
    let run = |work| {
        let mut memory = execution
            .reserve(32 * size_of::<(Id, u64)>() as u64)
            .unwrap();
        reset_counts();
        let next = next_tree::<_, _, _, u32>(
            &retained,
            plan(&keys),
            work,
            keys.iter().map(|id| (*id, 2)).collect(),
            u64::MAX,
            DECLARED,
            &reducer,
            &execution,
            &mut memory,
        );
        reconcile(next.report);
        next
    };
    let edited = run(plan(&keys).checked_build_work());
    assert!(matches!(
        edited.report,
        TreeRun::Rebuilt(Rebuild::EditCapacityOverflow, _)
    ));
    assert!(
        edited.report.metrics_for_test().combine_calls > 64,
        "successful edits preceded the overflow"
    );
    let rebuilt = run(None);
    assert!(matches!(
        rebuilt.report,
        TreeRun::Rebuilt(Rebuild::WorkCeiling, _)
    ));
    let result = *edited.outcome.unwrap().result();
    assert_eq!(result, *rebuilt.outcome.unwrap().result());
    // Execute the full builder with every leaf freshly provided, no prior.
    reset_counts();
    let (fresh, _) = Tree::try_from_declared_checked(
        plan(&keys),
        keys.iter().map(|id| (*id, 2)).collect(),
        0,
        counted as fn(&u64, &u64) -> u64,
        DECLARED,
        || execution.checkpoint(),
    )
    .unwrap();
    assert_eq!(result, *fresh.result());
    assert_eq!(CALLS.get(), 64);
    assert_eq!(NODES.get(), 32);
}

#[derive(Clone)]
struct Value {
    sum: u64,
    invalid: bool,
    storage: Vec<u8>,
}

impl ChargedBytes for Value {
    fn additional_charged_bytes(&self) -> u64 {
        self.storage.capacity() as u64
    }
}
impl CanonicalBits for Value {
    fn canonical_len(&self) -> Option<usize> {
        (!self.invalid).then_some(8)
    }
    fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        !self.invalid && visitor(&self.sum.to_le_bytes())
    }
}
fn value(sum: u64) -> Value {
    Value {
        sum,
        invalid: false,
        storage: Vec::new(),
    }
}
fn combine(left: &Value, right: &Value) -> Value {
    value(left.sum + right.sum)
}
type ValueTree = ReductionTree<Value, fn(&Value, &Value) -> Value>;

#[test]
fn edit_encoding_and_result_capacity_denials_rebuild_and_match_reuse_off() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let source = template();
    let keys: Vec<_> = (0..8).map(Id::new).collect();
    let prior = retained(&source, &keys);
    let native_policy = test_policy(std::num::NonZeroUsize::MIN, Some(1 << 30));
    let native = worth_execution::SerialRequest::from_policy(
        &native_policy,
        worth_execution::CancellationToken::new(),
        None,
    );
    let map = worth_execution::ExecutionMap::<_, u64>::from_keyless_partitions(
        keys.iter()
            .map(|id| {
                (
                    *id,
                    worth_execution::KeylessPartition {
                        value: (),
                        kernel_scratch_bytes: 0,
                        max_result_bytes: 0,
                    },
                )
            })
            .collect(),
    )
    .unwrap();
    let worth_execution::RetainedRunOutcome::Complete {
        run: native_run, ..
    } = map
        .run_retained_run(
            worth_execution::ExecutionRequest::serial(&native),
            worth_execution::ExecutionWorkCeiling::new(None),
            |_, context| {
                context.checkpoint(1)?;
                Ok::<_, worth_execution::MapKernelFailure<()>>(value(1))
            },
        )
        .unwrap()
    else {
        panic!("real native Value run")
    };
    let retained = RetainedPartitions {
        item_bytes: prior.item_bytes,
        item_call_bytes: prior.item_call_bytes,
        call_sequence: Arc::clone(&prior.call_sequence),
        items: prior.items,
        digests: prior.digests,
        membership: prior.membership,
        item_keys: prior.item_keys,
        routing: prior.routing,
        partitions: prior.partitions,
        native_run,
        tree: ValueTree::try_from_declared(
            plan(&keys),
            keys.iter().map(|id| (*id, value(1))).collect(),
            value(0),
            combine as fn(&Value, &Value) -> Value,
        )
        .unwrap()
        .0,
    };
    let request = live_scope();
    let execution = QueryRequestExecution::open(serial_placement(), &request);
    let reducer = WorthQueryDeterministicReducer::canonical(|| value(0), combine);
    let cases = [
        (
            Value {
                invalid: true,
                ..value(2)
            },
            Rebuild::EditDenied,
            WorthQueryPartitionedComputationDenial::ReducedEncodingInvalid,
        ),
        (
            Value {
                storage: vec![0; 4096],
                ..value(2)
            },
            Rebuild::ResultCapacityExceeded,
            WorthQueryPartitionedComputationDenial::Resource(Resource::ResultCapacityExceeded),
        ),
    ];
    for (leaf, cause, denial) in cases {
        let run = |work| {
            let mut memory = execution.reserve(size_of::<(Id, Value)>() as u64).unwrap();
            next_tree::<_, _, _, u32>(
                &retained,
                plan(&keys),
                work,
                BTreeMap::from([(keys[0], leaf.clone())]),
                u64::MAX,
                4096,
                &reducer,
                &execution,
                &mut memory,
            )
        };
        let edited = run(plan(&keys).checked_build_work());
        assert!(matches!(edited.report, TreeRun::Rebuilt(observed, _) if observed == cause));
        assert!(matches!(edited.outcome, Err(actual) if actual == denial));
        let rebuilt = run(None);
        assert!(matches!(rebuilt.outcome, Err(actual) if actual == denial));
        let fresh = ValueTree::try_from_declared_checked(
            plan(&keys),
            keys.iter()
                .map(|id| {
                    (
                        *id,
                        if *id == keys[0] {
                            leaf.clone()
                        } else {
                            value(1)
                        },
                    )
                })
                .collect(),
            value(0),
            combine as fn(&Value, &Value) -> Value,
            4096,
            || execution.checkpoint(),
        );
        let fresh = fresh.map_err(WorthQueryPartitionedComputationDenial::<u32>::from_reduction);
        assert!(matches!(fresh, Err(actual) if actual == denial));
    }
}

#[test]
fn supplied_native_counter_failure_is_normalized_and_its_metrics_are_carried() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    // Supply a typed native failure to exercise production normalization and metric carriage.
    use super::super::super::tree_attempt::TreeAttempt;
    let native =
        TreeAttempt::<(), _>::native(Err(ReductionRunFailure::<worth_execution::MapKernelStop> {
            reason: ReductionRunStop::WorkCounterOverflow,
            metrics: worth_execution::ReductionMetrics {
                structural_visits: u64::MAX,
                charged_work: u64::MAX,
                charged_span: u64::MAX,
                ..Default::default()
            },
        }));

    let reported = native.finish_edits::<u32>().resolve(|| {
        TreeAttempt::<(), ReductionRunFailure<worth_execution::MapKernelStop>>::native(Ok((
            (),
            worth_execution::ReductionMetrics::default(),
        )))
        .map_result(|outcome| {
            outcome.map_err(
                super::super::super::super::WorthQueryPartitionedComputationDenial::from_reduction,
            )
        })
    });
    assert!(matches!(
        reported.report,
        TreeRun::Rebuilt(Rebuild::WorkCounterOverflow, _)
    ));
    assert_eq!(
        reported.report.metrics_for_test().structural_visits,
        u128::from(u64::MAX)
    );
    for (reason, cause) in [
        (
            ReductionDenial::ResultCapacityExceeded,
            Rebuild::ResultCapacityExceeded,
        ),
        (
            ReductionDenial::WorkCounterOverflow,
            Rebuild::WorkCounterOverflow,
        ),
        (ReductionDenial::ReducerPanic, Rebuild::ReducerPanicked),
    ] {
        let supplied = TreeAttempt::<(), _>::native(Err(ReductionRunFailure {
            reason: ReductionRunStop::<worth_execution::MapKernelStop>::Denial(reason),
            metrics: worth_execution::ReductionMetrics::default(),
        }));
        let reported = supplied.finish_edits::<u32>().resolve(|| {
            TreeAttempt::<(), ReductionRunFailure<worth_execution::MapKernelStop>>::native(Ok(((), worth_execution::ReductionMetrics::default())))
            .map_result(|outcome| outcome.map_err(super::super::super::super::WorthQueryPartitionedComputationDenial::from_reduction))
        });
        assert!(matches!(reported.report, TreeRun::Rebuilt(actual, _) if actual == cause));
    }
}

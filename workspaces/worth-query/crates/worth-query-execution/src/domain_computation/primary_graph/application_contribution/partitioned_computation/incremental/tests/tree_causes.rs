//! Fallback causes at their producer, including arithmetic that rebuilding survives.

use worth_execution::{CanonicalBits, ReductionDenial, ReductionRunFailure, ReductionRunStop};

use super::*;
use crate::domain_computation::primary_graph::application_contribution::request_execution::test_policy;

#[derive(Clone)]
struct Value {
    sum: u64,
    invalid: bool,
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
    }
}
fn combine(left: &Value, right: &Value) -> Value {
    value(left.sum + right.sum)
}

#[test]
fn edit_encoding_denial_rebuilds_and_matches_reuse_off() {
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
    let map = worth_execution::ExecutionMap::from_native_partitions(
        keys.iter().map(|id| (*id, ())).collect(),
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
    let request = live_scope();
    let execution = QueryRequestExecution::open(serial_placement(), &request);
    let retained = RetainedPartitions {
        call_sequence: Arc::clone(&prior.call_sequence),
        items: prior.items,
        digests: prior.digests,
        membership: prior.membership,
        item_keys: prior.item_keys,
        routing: prior.routing,
        partitions: prior.partitions,
        native_run,
        tree: ReductionTree::<Value, fn(&Value, &Value) -> Value>::try_from_declared_native(
            plan(&keys),
            keys.iter().map(|id| (*id, value(1))).collect(),
            value(0),
            combine as fn(&Value, &Value) -> Value,
            Some(execution.dispatch().unwrap().native_lease_for_test()),
            || execution.checkpoint(),
        )
        .unwrap()
        .0,
    };
    let reducer = WorthQueryDeterministicReducer::canonical(|| value(0), combine);
    let cases = [(
        Value {
            invalid: true,
            ..value(2)
        },
        Rebuild::EditDenied,
        WorthQueryPartitionedComputationDenial::ReducedEncodingInvalid,
    )];
    for (leaf, cause, denial) in cases {
        let run = |work| {
            next_tree::<_, _, _, u32>(
                &retained,
                plan(&keys),
                work,
                BTreeMap::from([(keys[0], leaf.clone())]),
                None,
                &reducer,
                &execution,
            )
        };
        let edited = run(plan(&keys).checked_build_work());
        assert!(matches!(edited.report, TreeRun::Rebuilt(observed, _) if observed == cause));
        assert!(matches!(edited.outcome, Err(actual) if actual == denial));
        let rebuilt = run(None);
        assert!(matches!(rebuilt.outcome, Err(actual) if actual == denial));
        let fresh = ReductionTree::<Value, fn(&Value, &Value) -> Value>::try_from_declared_native(
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
            Some(execution.dispatch().unwrap().native_lease_for_test()),
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

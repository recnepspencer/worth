//! Storage/canonical-prefix proofs; no source or native completion is fabricated here.
use super::*;
use crate::domain_computation::primary_graph::{
    application_contribution::test_policy, tests::fixture::live_scope,
};
fn read(id: u64) -> ComputationRead {
    ComputationRead::ItemKey(worth_execution::PartitionItemId(id))
}
#[test]
fn insert_retire_changes_only_paths_preserving_order_history_range_and_fresh_closure() {
    let request = live_scope();
    let execution = QueryRequestExecution::open(
        worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Serial(test_policy(
            std::num::NonZeroUsize::MIN,
            Some(1 << 24),
        )),
        &request,
    );
    let mut calls = super::tests::calls();
    let entity = worth_relational::facade::identity::EntityId::new(
        worth_relational::facade::identity::PartitionId::main(),
        1,
        1,
    );
    calls[3].1 .0.reached = Arc::from([entity]);
    let prior = ComputationCallSequence::build(
        || {
            calls
                .iter()
                .map(|(read, (charge, declared))| (*read, charge, *declared))
        },
        &execution,
    )
    .unwrap();
    let charge = &calls[2].1 .0;
    let edit = prior
        .shape()
        .prepare_transition(
            &BTreeSet::from([read(8)]),
            &BTreeSet::from([read(1)]),
            &execution,
        )
        .unwrap();
    let next = prior
        .edited(&edit, std::iter::once((read(8), charge, 17)), &execution)
        .unwrap();
    assert!(prior.contains_in(entity, &(3..4)));
    assert!(next.contains_in(entity, &(2..3)));
    assert!(!next.contains_in(entity, &(3..4)));
    let expected = vec![read(0), read(2), read(3), read(4), read(5), read(8)];
    assert_eq!(next.shape.calls().collect::<Vec<_>>(), expected);
    for (ordinal, read) in expected.iter().enumerate() {
        assert_eq!(next.shape.ordinal(*read), Some(ordinal));
        assert_eq!(next.shape.call_at(ordinal), Some(*read));
    }
    assert_eq!(
        prior.shape.calls().collect::<Vec<_>>(),
        calls.iter().map(|(read, _)| *read).collect::<Vec<_>>()
    );
    assert!(edit.maps(2..6, 1..5));
    assert!(!edit.maps(0..6, 0..6));
    assert!(!edit.maps(2..6, 2..6));
    assert!(edit.matches_prior(prior.shape()));
    assert!(!edit.matches_prior(next.shape()));
    assert!(edit.closes_fresh(&BTreeSet::from([read(8)])));
    assert!(!edit.closes_fresh(&BTreeSet::new()));
    assert!(!edit.closes_fresh(&BTreeSet::from([read(1), read(8)])));
    let mut truth = calls
        .iter()
        .filter(|(key, _)| *key != read(1))
        .cloned()
        .collect::<Vec<_>>();
    truth.push((read(8), (charge.clone(), 17)));
    for start in 0..truth.len() {
        for end in start + 1..=truth.len() {
            let summary = next.summary(start..end).unwrap();
            assert_eq!(
                summary.declared,
                truth[start..end]
                    .iter()
                    .map(|(_, (_, declared))| *declared)
                    .sum::<u64>()
            );
            assert_eq!(
                summary.consumed,
                truth[start..end]
                    .iter()
                    .map(|(_, (charge, _))| charge.consumed)
                    .sum::<usize>()
            );
        }
    }
    // The edited sequence must descend to the same actual unfitting call,
    // rather than charging a carried summary beyond its native reader prefix.
    use crate::domain_computation::primary_graph::invariant_projection::work::WorthQueryInvariantProjectionWorkBudget as Budget;
    for remaining in 0..40 {
        let mut full = Budget::bounded(remaining);
        let expected = truth.iter().position(|(_, (charge, _))| {
            if full.can_afford(charge.demand) {
                assert!(full.consume(charge.consumed));
                false
            } else {
                true
            }
        });
        let mut ranged = Budget::bounded(remaining);
        let actual = next
            .visit(0..truth.len(), &BTreeSet::new(), &mut |visit| -> Result<
                bool,
                usize,
            > {
                match visit {
                    CallVisit::Range(_, sum) => Ok(ranged.carry(sum.demand, sum.consumed)),
                    CallVisit::Leaf(at, _, charge, _) => {
                        if !ranged.can_afford(charge.demand) {
                            return Err(at);
                        }
                        assert!(ranged.consume(charge.consumed));
                        Ok(true)
                    }
                }
            })
            .err();
        assert_eq!(actual, expected);
        assert_eq!(
            (ranged.remaining(), ranged.exceeded()),
            (full.remaining(), full.exceeded())
        );
    }
    let back = next
        .shape()
        .prepare_transition(
            &BTreeSet::from([read(1)]),
            &BTreeSet::from([read(8)]),
            &execution,
        )
        .unwrap();
    let returned = next
        .edited(
            &back,
            std::iter::once((read(1), &calls[1].1 .0, calls[1].1 .1)),
            &execution,
        )
        .unwrap();
    assert!(returned.shape.calls().eq(prior.shape.calls()));
    assert_eq!(
        returned.summary(0..calls.len()).unwrap().declared,
        prior.summary(0..calls.len()).unwrap().declared
    );
}
#[test]
fn refused_shape_edit_leaves_original_and_shape_views_usable() {
    let request = live_scope();
    let execution = QueryRequestExecution::open(
        worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Serial(test_policy(
            std::num::NonZeroUsize::MIN,
            Some(1 << 24),
        )),
        &request,
    );
    let calls = super::tests::calls();
    let prior = ComputationCallSequence::build(
        || {
            calls
                .iter()
                .map(|(read, (charge, declared))| (*read, charge, *declared))
        },
        &execution,
    )
    .unwrap();
    let shape = Arc::clone(prior.shape());
    let narrow = QueryRequestExecution::open(
        worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Serial(test_policy(
            std::num::NonZeroUsize::MIN,
            Some(1),
        )),
        &request,
    );
    assert!(prior
        .shape()
        .prepare_transition(&BTreeSet::from([read(8)]), &BTreeSet::new(), &narrow)
        .is_err());
    assert!(prior
        .shape()
        .prepare_transition(&BTreeSet::new(), &BTreeSet::from([read(99)]), &execution)
        .is_err());
    assert!(prior
        .shape()
        .prepare_transition(
            &BTreeSet::from([read(1)]),
            &BTreeSet::from([read(1)]),
            &execution
        )
        .is_err());
    let edit = prior
        .shape()
        .prepare_transition(&BTreeSet::from([read(8)]), &BTreeSet::new(), &execution)
        .unwrap();
    assert!(
        prior.edited(&edit, std::iter::empty(), &execution).is_err(),
        "inserted call needs an actual measured charge"
    );
    assert!(prior
        .edited(
            &edit,
            std::iter::once((read(8), &calls[0].1 .0, 1)),
            &narrow
        )
        .is_err());
    drop(edit);
    assert_eq!(shape.len(), calls.len());
    drop(prior);
    assert!(execution.reserve(1 << 24).is_err());
    drop(shape);
    assert!(execution.reserve(1 << 24).is_ok());
}

//! Storage/canonical-prefix proofs; no source or native completion is fabricated here.
use std::sync::Arc;
use std::collections::BTreeSet;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::operation_reader::computation_calls::sequence::CallVisit;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::operation_reader::computation_calls::sequence::ComputationCallSequence;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::computation_routing::ComputationRead;
use worth_query_execution::court_doors_for_test::domain_computation::request_execution::QueryRequestExecution;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::operation_reader::computation_calls::sequence::tree::summary;
use crate::domain_computation::primary_graph::invariant_projection::operation_reader::computation_calls::sequence::tests;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::operation_reader::computation_calls::sequence::tree::visit;
use worth_query_execution::court_doors_for_test::domain_computation::request_execution::test_placement::test_policy;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub fn read(id: u64) -> ComputationRead {
    ComputationRead::ItemKey(worth_execution::PartitionItemId(id))
}
#[test]
fn insert_retire_changes_only_paths_preserving_order_history_range_and_fresh_closure() {
    let request = live_scope();
    let execution = QueryRequestExecution::open_for_test(
        worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Serial(test_policy(
            std::num::NonZeroUsize::MIN,
            Some(1 << 24),
        )),
        &request,
    );
    let mut calls = crate::domain_computation::primary_graph::invariant_projection::operation_reader::computation_calls::sequence::tests::calls();
    let entity = worth_relational::facade::identity::EntityId::new(
        worth_relational::facade::identity::PartitionId::main(),
        1,
        1,
    );
    (*calls[3].1 .0.reached_mut_for_test()) = Arc::from([entity]);
    let prior = ComputationCallSequence::build_for_test(
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
        .shape_for_test()
        .prepare_transition_for_test(
            &BTreeSet::from([read(8)]),
            &BTreeSet::from([read(1)]),
            &execution,
        )
        .unwrap();
    let next = prior
        .edited_for_test(
            &edit,
            std::iter::once((read(8), charge, Some(17))),
            &execution,
        )
        .unwrap();
    assert!(prior.contains_in(entity, &(3..4)));
    assert!(next.contains_in(entity, &(2..3)));
    assert!(!next.contains_in(entity, &(3..4)));
    let expected = vec![read(0), read(2), read(3), read(4), read(5), read(8)];
    assert_eq!(
        (*next.shape_for_test()).calls().collect::<Vec<_>>(),
        expected
    );
    for (ordinal, read) in expected.iter().enumerate() {
        assert_eq!((*next.shape_for_test()).ordinal(*read), Some(ordinal));
        assert_eq!((*next.shape_for_test()).call_at(ordinal), Some(*read));
    }
    assert_eq!(
        (*prior.shape_for_test()).calls().collect::<Vec<_>>(),
        calls.iter().map(|(read, _)| *read).collect::<Vec<_>>()
    );
    assert!(edit.maps(2..6, 1..5));
    assert!(!edit.maps(0..6, 0..6));
    assert!(!edit.maps(2..6, 2..6));
    assert!(edit.matches_prior(prior.shape_for_test()));
    assert!(!edit.matches_prior(next.shape_for_test()));
    assert!(edit.closes_fresh(&BTreeSet::from([read(8)])));
    assert!(!edit.closes_fresh(&BTreeSet::new()));
    assert!(!edit.closes_fresh(&BTreeSet::from([read(1), read(8)])));
    let mut truth = calls
        .iter()
        .filter(|(key, _)| *key != read(1))
        .cloned()
        .collect::<Vec<_>>();
    truth.push((read(8), (charge.clone(), Some(17))));
    for start in 0..truth.len() {
        for end in start + 1..=truth.len() {
            let summary = next.summary(start..end, true).unwrap();
            assert_eq!(
                (*summary.declared_for_test()),
                Some(
                    truth[start..end]
                        .iter()
                        .map(|(_, (_, declared))| declared.unwrap())
                        .sum::<u64>()
                )
            );
            assert_eq!(
                (*(*summary.accounting_for_test())
                    .unwrap()
                    .consumed_for_test()),
                truth[start..end]
                    .iter()
                    .map(|(_, (charge, _))| (*(*charge.accounting_for_test())
                        .unwrap()
                        .consumed_for_test()))
                    .sum::<usize>()
            );
        }
    }
    // The edited sequence must descend to the same actual unfitting call,
    // rather than charging a carried summary beyond its native reader prefix.
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::work::WorthQueryInvariantProjectionWorkBudget as Budget;
    for remaining in 0..40 {
        let mut full = Budget::bounded_for_test(remaining);
        let expected = truth.iter().position(|(_, (charge, _))| {
            let charge = (*charge.accounting_for_test()).unwrap();
            if full.can_afford_for_test((*charge.demand_for_test())) {
                assert!(full.consume_for_test((*charge.consumed_for_test())));
                false
            } else {
                true
            }
        });
        let mut ranged = Budget::bounded_for_test(remaining);
        let actual = next
            .visit_for_test(0..truth.len(), &BTreeSet::new(), &mut |visit| -> Result<
                bool,
                usize,
            > {
                match visit {
                    CallVisit::Range(_, sum) => {
                        let cost = (*sum.accounting_for_test()).unwrap();
                        Ok(ranged
                            .carry_for_test((*cost.demand_for_test()), (*cost.consumed_for_test())))
                    }
                    CallVisit::Leaf(at, _, charge, _) => {
                        let charge = (*charge.accounting_for_test()).unwrap();
                        if !ranged.can_afford_for_test((*charge.demand_for_test())) {
                            return Err(at);
                        }
                        assert!(ranged.consume_for_test((*charge.consumed_for_test())));
                        Ok(true)
                    }
                }
            })
            .err();
        assert_eq!(actual, expected);
        assert_eq!(
            (ranged.remaining_for_test(), ranged.exceeded_for_test()),
            (full.remaining_for_test(), full.exceeded_for_test())
        );
    }
    let back = next
        .shape_for_test()
        .prepare_transition_for_test(
            &BTreeSet::from([read(1)]),
            &BTreeSet::from([read(8)]),
            &execution,
        )
        .unwrap();
    let returned = next
        .edited_for_test(
            &back,
            std::iter::once((read(1), &calls[1].1 .0, calls[1].1 .1)),
            &execution,
        )
        .unwrap();
    assert!((*returned.shape_for_test())
        .calls()
        .eq((*prior.shape_for_test()).calls()));
    assert_eq!(
        (*returned
            .summary(0..calls.len(), true)
            .unwrap()
            .declared_for_test()),
        (*prior
            .summary(0..calls.len(), true)
            .unwrap()
            .declared_for_test())
    );
}
#[test]
fn refused_shape_edit_leaves_original_and_shape_views_usable() {
    let request = live_scope();
    let execution = QueryRequestExecution::open_for_test(
        worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Serial(test_policy(
            std::num::NonZeroUsize::MIN,
            Some(1 << 24),
        )),
        &request,
    );
    let calls = crate::domain_computation::primary_graph::invariant_projection::operation_reader::computation_calls::sequence::tests::calls();
    let prior = ComputationCallSequence::build_for_test(
        || {
            calls
                .iter()
                .map(|(read, (charge, declared))| (*read, charge, *declared))
        },
        &execution,
    )
    .unwrap();
    let shape = Arc::clone(prior.shape_for_test());
    let narrow = QueryRequestExecution::open_for_test(
        worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Serial(test_policy(
            std::num::NonZeroUsize::MIN,
            Some(1),
        )),
        &request,
    );
    assert!(prior
        .shape_for_test()
        .prepare_transition_for_test(&BTreeSet::from([read(8)]), &BTreeSet::new(), &narrow)
        .is_err());
    assert!(prior
        .shape_for_test()
        .prepare_transition_for_test(&BTreeSet::new(), &BTreeSet::from([read(99)]), &execution)
        .is_err());
    assert!(prior
        .shape_for_test()
        .prepare_transition_for_test(
            &BTreeSet::from([read(1)]),
            &BTreeSet::from([read(1)]),
            &execution
        )
        .is_err());
    let edit = prior
        .shape_for_test()
        .prepare_transition_for_test(&BTreeSet::from([read(8)]), &BTreeSet::new(), &execution)
        .unwrap();
    assert!(
        prior
            .edited_for_test(&edit, std::iter::empty(), &execution)
            .is_err(),
        "inserted call needs an actual measured charge"
    );
    assert!(prior
        .edited_for_test(
            &edit,
            std::iter::once((read(8), &calls[0].1 .0, Some(1))),
            &narrow
        )
        .is_err());
    drop(edit);
    assert_eq!(shape.len_for_test(), calls.len());
    drop(prior);
    assert!(execution.reserve(1 << 24).is_err());
    drop(shape);
    assert!(execution.reserve(1 << 24).is_ok());
}

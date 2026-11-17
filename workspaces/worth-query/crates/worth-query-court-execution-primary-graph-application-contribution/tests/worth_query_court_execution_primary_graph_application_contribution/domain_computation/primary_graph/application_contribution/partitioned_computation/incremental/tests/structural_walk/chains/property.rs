//! Projected carry equals fresh Full after generated lawful committed edits.
use super::*;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 16,
        failure_persistence: Some(Box::new(proptest::test_runner::FileFailurePersistence::Direct(
            concat!(env!("CARGO_MANIFEST_DIR"), "/proptest-regressions/structural-chains.txt"),
        ))),
        rng_seed: proptest::test_runner::RngSeed::Fixed(20901),
        .. ProptestConfig::default()
    })]

    #[test]
    fn projected_walk_carry_matches_full_after_generated_committed_edits(
        change in 0u8..3,
        edited_nodes in proptest::collection::vec(any::<bool>(), 1..4),
    ) {
        let _observation = worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
        let mut graph = graph();
        if change == 0 {
            graph.edges.push((7, 3));
        }
        let world = world::install(graph);
        let installed = Installed::new(Owner::default(), ComputationRetention::ProducerOperation);
        let before = attempt::run(&world, &installed, first(), Some(Cause::FirstRun));
        let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
        let program = match change {
            0 => fixture::program::replace_next(&world, &request, 7),
            1 => fixture::program::link(&world, &request, 3, 7),
            _ => fixture::program::replace_next(&world, &request, 2),
        };
        prop_assert!(matches!(fixture::commit(&world, &request, program, 120), WorthQueryApplicationCommitOutcome::Committed(_)));
        let mut current = attempt::run(&world, &installed, prior(before), None);
        prop_assert_eq!(&current.gathered, &[0], "the second chain is unchanged");
        prop_assert_eq!(current.outcome, if change == 2 { 6 } else { 8 });
        let mut active = [false; 2];
        for (step, second_node) in edited_nodes.into_iter().enumerate() {
            let full = attempt::run(&world, &installed, first(), Some(Cause::FirstRun));
            prop_assert_eq!(current.outcome, full.outcome, "same committed source Full oracle");
            prop_assert!((*(*current.sealed.state_for_test()).facts_for_test()).has_structural_walk_for_test());
            // The helper also asserts real projected completion consumes current-issued Arcs.
            let carried = attempt::run(&world, &installed, prior(current), None);
            prop_assert!(carried.gathered.is_empty(), "consecutive unchanged seal");
            prop_assert_eq!(carried.outcome, full.outcome);
            // Active is outside this owner's walk input. There are no blocked members.
            let slot = usize::from(second_node);
            active[slot] = !active[slot];
            let program = fixture::program::set_node_active(&world, &request, 6 + slot as u64, active[slot]);
            prop_assert!(matches!(fixture::commit(&world, &request, program, 121 + step as u8), WorthQueryApplicationCommitOutcome::Committed(_)));
            current = attempt::run(&world, &installed, prior(carried), None);
            prop_assert!(current.gathered.is_empty(), "unread field edits preserve both walks");
        }
        let full = attempt::run(&world, &installed, first(), Some(Cause::FirstRun));
        prop_assert_eq!(current.outcome, full.outcome, "last committed edit is also checked");
        prop_assert!((*(*current.sealed.state_for_test()).facts_for_test()).has_structural_walk_for_test());
    }
}

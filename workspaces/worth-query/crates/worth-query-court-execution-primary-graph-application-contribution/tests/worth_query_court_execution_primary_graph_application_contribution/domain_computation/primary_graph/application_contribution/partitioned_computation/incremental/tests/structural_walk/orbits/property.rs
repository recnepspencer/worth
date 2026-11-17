//! Generated lawful merge/split sequences preserve projected aggregate meaning.
use super::*;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 16,
        failure_persistence: Some(Box::new(proptest::test_runner::FileFailurePersistence::Direct(
            concat!(env!("CARGO_MANIFEST_DIR"), "/proptest-regressions/structural-orbits.txt"),
        ))),
        rng_seed: proptest::test_runner::RngSeed::Fixed(20902),
        .. ProptestConfig::default()
    })]

    #[test]
    fn projected_orbit_carry_matches_full_after_generated_committed_edits(
        left in 0u64..3,
        right in 6u64..8,
        swaps in proptest::collection::vec(any::<bool>(), 1..4),
    ) {
        let _observation = worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
        let world = fixture::decision::identity::install();
        let installed = Installed::new(Owner::default(), ComputationRetention::ProducerOperation);
        let mut current = attempt::run(&world, &installed, first(), Run::Full(Cause::FirstRun));
        let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
        let mut merged = false;
        // A required first edit prevents an all-unchanged generated sequence.
        for (step, swap) in std::iter::once(true).chain(swaps).enumerate() {
            if swap {
                let program = fixture::program::swap_successors(&world, &request, left, right);
                prop_assert!(matches!(fixture::commit(&world, &request, program, 140 + step as u8), WorthQueryApplicationCommitOutcome::Committed(_)));
                merged = !merged;
            }
            current = attempt::run(&world, &installed, prior(current), Run::Incremental);
            if swap {
                prop_assert_eq!(&current.gathered, &[0], "the independent second orbit is carried");
            } else {
                prop_assert!(current.gathered.is_empty(), "unchanged source carries both orbits");
            }
            let full = attempt::run(&world, &installed, first(), Run::Full(Cause::FirstRun));
            prop_assert_eq!(current.outcome, full.outcome, "same committed source Full oracle");
            prop_assert_eq!(current.outcome, if merged { 8 } else { 6 }, "independent membership total");
            let facts = (*current.sealed.state_for_test()).facts_for_test();
            prop_assert!(facts.has_structural_walk_for_test());
            prop_assert!(facts.has_aggregate_for_test());
            // The helper asserts the current-issued walk and aggregate Arc custody.
            current = attempt::run(&world, &installed, prior(current), Run::Incremental);
            prop_assert!(current.gathered.is_empty(), "consecutive unchanged seal");
            prop_assert_eq!(current.outcome, full.outcome);
            let facts = (*current.sealed.state_for_test()).facts_for_test();
            prop_assert!(facts.has_structural_walk_for_test());
            prop_assert!(facts.has_aggregate_for_test());
        }
    }
}

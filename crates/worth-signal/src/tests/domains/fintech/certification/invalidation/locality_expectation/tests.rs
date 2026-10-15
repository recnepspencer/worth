use super::*;
use crate::tests::domains::fintech::certification::invalidation::FreshFinancialLocalityRecompute;
use crate::tests::domains::fintech::world::{
    compile_financial_locality_world, ordinary_locality_cases, FinancialLocalityScenario,
    FinancialWorldDefinition, LocalityCaseContract, SparseFanoutAxis,
};

#[test]
fn complete_sparse_trace_retains_every_binding_axis_and_hop() {
    let definition =
        FinancialWorldDefinition::sparse_book_fanout(41, 64, SparseFanoutAxis::IndexDisjoint);
    let locality = definition.locality().unwrap();
    let manifest = FinancialLocalityExpectationManifest::derive(locality, 73);

    assert_eq!(
        manifest.scenario(),
        FinancialLocalityScenario::SparseBookFanout
    );
    assert_eq!(manifest.queried_bucket_keys().len(), 16);
    assert_eq!(manifest.candidate_dependencies().len(), 15);
    assert_eq!(manifest.canonical_causes().len(), 15);
    assert_eq!(manifest.canonical_work().len(), 16);
    assert_eq!(manifest.necessary_evaluations().len(), 16);
    assert!(manifest.unchanged_output_stops().is_empty());
    assert_eq!(manifest.peak_ready_width(), 1);
    assert_eq!(
        manifest
            .counter_manifest()
            .value(ExpectedLocalityCounterRow::SourceOutputDeltasConsumed),
        16
    );
    assert!(manifest.canonical_causes().iter().all(|cause| {
        cause.graph.graph_instance == 73
            && cause.graph.seed == 41
            && cause.graph.scale == locality.scale()
            && cause.dependency_revision == 1
            && cause.cached_version == 1
            && cause.committed_version == 2
            && cause.output_commit_ordinal > locality.outputs().len() as u64
    }));
    assert_eq!(
        manifest
            .canonical_causes()
            .iter()
            .map(|cause| cause.output_commit_ordinal)
            .collect::<BTreeSet<_>>(),
        (65_u64..80).collect()
    );
}

#[test]
fn partition_trace_separates_queried_candidates_from_fully_bound_causes() {
    let definition = FinancialWorldDefinition::partitioned_curve_universe(41, 16, 4, 8);
    let locality = definition.locality().unwrap();
    let manifest = FinancialLocalityExpectationManifest::derive(locality, 73);
    let detail = LocalityScope::detail(0, 0);

    assert_eq!(manifest.queried_bucket_keys().len(), 12);
    assert_eq!(manifest.candidate_dependencies().len(), 12);
    assert_eq!(manifest.canonical_causes().len(), 9);
    assert_eq!(manifest.canonical_work().len(), 10);
    assert_eq!(manifest.necessary_evaluations().len(), 10);
    assert_eq!(manifest.peak_ready_width(), 8);
    assert_eq!(
        manifest
            .canonical_causes()
            .iter()
            .filter(|cause| cause.producer == locality.mutation().producer)
            .map(|cause| cause.changed_scopes.clone())
            .collect::<Vec<_>>(),
        vec![vec![detail]]
    );
    assert!(manifest
        .canonical_causes()
        .iter()
        .filter(|cause| cause.producer != locality.mutation().producer)
        .all(|cause| cause.changed_scopes.is_empty()));
}

#[test]
fn candidate_contract_is_an_ordered_multiset_not_a_distinct_set() {
    let definition =
        FinancialWorldDefinition::sparse_book_fanout(41, 64, SparseFanoutAxis::QueriedRejecting);
    let manifest = FinancialLocalityExpectationManifest::derive(definition.locality().unwrap(), 73);

    assert_eq!(manifest.queried_bucket_keys().len(), 18);
    assert_eq!(manifest.candidate_dependencies().len(), 63);
    assert!(manifest
        .candidate_dependencies()
        .windows(2)
        .all(|pair| pair[0].query_ordinal <= pair[1].query_ordinal));
}

#[test]
fn every_financial_scenario_compiles_subscriptions_into_a_sealed_baseline() {
    let cases = ordinary_locality_cases();
    for scenario in FinancialLocalityScenario::ALL {
        let case = representative_case(&cases, scenario);
        let world = FinancialWorldDefinition::locality_case(41, case);
        let fresh = FreshFinancialLocalityRecompute::run(world.locality().unwrap());
        let compiled = compile_financial_locality_world(world)
            .expect("financial compiler must seal the economic locality baseline");
        assert_eq!(
            compiled
                .committed_locality_financial_values()
                .expect("compiled locality baseline must expose committed values"),
            *fresh.baseline_values()
        );
        assert_eq!(compiled.locality_definition().scenario(), scenario);
    }
}

fn representative_case(
    cases: &[LocalityCaseContract],
    scenario: FinancialLocalityScenario,
) -> LocalityCaseContract {
    *cases
        .iter()
        .find(|case| case.scenario() == scenario)
        .expect("each lane freezes every locality scenario")
}

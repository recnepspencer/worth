use crate::data::node::NodeState;
use crate::data::telemetry::InvalidationPerformedCounter;
use crate::tests::domains::fintech::{
    compile_financial_world_with_policy, CompiledFinancialWorld, FinancialWorldDefinition,
    FreshFinancialRecompute,
};

use super::throughput_definition::profiles;

#[test]
fn throughput_idle_preserves_financial_truth_without_optional_observation_work() {
    let idle = profiles()
        .into_iter()
        .find(|profile| profile.name == "throughput_idle")
        .expect("idle profile");
    let definition = FinancialWorldDefinition::deterministic(41);
    let changed = definition.with_first_market_factor_delta(20_000);
    let fresh = FreshFinancialRecompute::run(&changed);

    let mut throughput = compile_financial_world_with_policy(definition, idle.policy)
        .expect("financial world compiles under idle policy")
        .into_compiled();
    let observation_before = OptionalObservationSnapshot::capture(&throughput);
    assert!(
        observation_before.is_idle_zero(),
        "idle compile must not retain optional observation: {observation_before:?}"
    );
    throughput
        .apply_first_market_factor_change(changed)
        .expect("throughput mutation should settle");

    assert_committed_financial_truth(&throughput, &fresh);
    let observation_after = OptionalObservationSnapshot::capture(&throughput);
    assert!(
        observation_after.is_idle_zero(),
        "throughput idle must not materialize optional observation surfaces: {observation_after:?}"
    );
}

#[test]
fn throughput_and_balanced_profiles_commit_the_same_financial_projection() {
    let idle = profiles()
        .into_iter()
        .find(|profile| profile.name == "throughput_idle")
        .expect("idle profile");
    let balanced = profiles()
        .into_iter()
        .find(|profile| profile.name == "balanced_continuous")
        .expect("balanced profile");
    let definition = FinancialWorldDefinition::deterministic(41);
    let changed = definition.with_first_market_factor_delta(20_000);

    let mut throughput = compile_financial_world_with_policy(definition.clone(), idle.policy)
        .expect("idle world compiles")
        .into_compiled();
    throughput
        .apply_first_market_factor_change(changed.clone())
        .expect("throughput mutation should settle");

    let mut balanced_world = compile_financial_world_with_policy(definition, balanced.policy)
        .expect("balanced world compiles")
        .into_compiled();
    balanced_world
        .apply_first_market_factor_change(changed)
        .expect("balanced mutation should settle");

    assert_committed_financial_truth(
        &throughput,
        &FreshFinancialRecompute::run(throughput.definition()),
    );
    assert_committed_financial_truth(
        &balanced_world,
        &FreshFinancialRecompute::run(balanced_world.definition()),
    );
    let idle_observation = OptionalObservationSnapshot::capture(&throughput);
    let balanced_observation = OptionalObservationSnapshot::capture(&balanced_world);
    assert!(idle_observation.is_idle_zero());
    assert!(!balanced_observation.is_idle_zero());
    assert_eq!(
        throughput.economic_snapshot(),
        balanced_world.economic_snapshot()
    );
    assert_eq!(throughput.projection(), balanced_world.projection());
    assert_eq!(
        throughput.ledger().observed_work(),
        balanced_world.ledger().observed_work(),
        "profile changes may alter observation, not semantic work identity"
    );
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct OptionalObservationSnapshot {
    lineage_records: usize,
    replay_events: usize,
    explanation_facts: usize,
    provenance_facts: usize,
    frontier_summary: bool,
    flow_summary: bool,
    performed_counter_total: u64,
    performed_work_records: usize,
}

impl OptionalObservationSnapshot {
    fn capture(world: &CompiledFinancialWorld) -> Self {
        let observer = world.graph().observe();
        let semantic_nodes = world
            .semantic_output_keys()
            .into_iter()
            .map(|key| world.handles().node_for(key))
            .collect::<Vec<_>>();
        let graph = world.graph();
        Self {
            lineage_records: observer.lineage_records().len(),
            replay_events: observer.replay_events().len(),
            explanation_facts: semantic_nodes
                .iter()
                .filter(|node| observer.explanation_fact(**node).is_some())
                .count(),
            provenance_facts: semantic_nodes
                .iter()
                .filter(|node| observer.provenance_fact(**node).is_some())
                .count(),
            frontier_summary: observer.latest_frontier_execution_summary().is_some(),
            flow_summary: observer.latest_flow_diagnostics().is_some(),
            performed_counter_total: InvalidationPerformedCounter::ALL
                .into_iter()
                .map(|counter| graph.invalidation_performed_counters().value(counter))
                .sum(),
            performed_work_records: graph.invalidation_performed_work().len(),
        }
    }

    fn is_idle_zero(&self) -> bool {
        *self
            == Self {
                lineage_records: 0,
                replay_events: 0,
                explanation_facts: 0,
                provenance_facts: 0,
                frontier_summary: false,
                flow_summary: false,
                performed_counter_total: 0,
                performed_work_records: 0,
            }
    }
}

fn assert_committed_financial_truth(
    world: &CompiledFinancialWorld,
    fresh: &FreshFinancialRecompute,
) {
    assert_eq!(
        world
            .committed_financial_values()
            .expect("committed values"),
        fresh.economic_snapshot().semantic_value_map(),
        "profile proof must observe committed artifact values, not wrapper projections"
    );
    let required = world.semantic_output_keys();
    world
        .verify_committed_financial_truth(&required)
        .expect("committed graph truth must match the independent financial oracle");
    for key in required {
        assert_eq!(
            world.node_state(key).expect("node state"),
            NodeState::Clean,
            "committed financial output must be settled"
        );
        assert!(
            world
                .graph()
                .pending_causes(world.handles().node_for(key))
                .expect("pending causes")
                .is_empty(),
            "committed financial output must not retain unresolved causes"
        );
    }
}

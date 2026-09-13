use crate::data::telemetry::{InvalidationPerformedCounter, SignalInvalidationRealizedCounters};
use crate::tests::domains::fintech::certification::invalidation::{
    ExpectedLocalityCounterRow, FinancialLocalityExpectationManifest,
};
use crate::tests::domains::fintech::world::{
    compile_financial_locality_world, ordinary_locality_cases, FinancialLocalityScenario,
    FinancialWorldDefinition,
};

#[test]
fn convergent_world_exposes_all_twenty_four_observed_rows() {
    let definition = FinancialWorldDefinition::convergent_factor_batch(41, 0);
    let mut compiled = compile_financial_locality_world(definition).unwrap();
    let manifest = FinancialLocalityExpectationManifest::derive(
        compiled.locality_definition(),
        compiled.locality_graph_instance(),
    );
    let observation = compiled.run_locality_action_trace(0).unwrap();
    let expected = expected_counters(&manifest);

    for counter in InvalidationPerformedCounter::ALL {
        assert_eq!(
            observation.performed_counters.value(counter),
            expected.value(counter),
            "performed counter drifted for {}",
            counter.name(),
        );
    }
}

#[test]
fn performed_receipt_requires_work_in_the_same_runtime_observation() {
    let mut first = crate::facade::SignalRuntime::builder(crate::facade::SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let second = crate::facade::SignalRuntime::builder(crate::facade::SignalGraph::new())
        .with_kernel_defaults()
        .build();

    let empty = first.begin_invalidation_execution_observation().unwrap();
    assert!(first
        .finish_invalidation_execution_observation(&empty)
        .is_err());
    let wrong_runtime = first.begin_invalidation_execution_observation().unwrap();
    assert!(second
        .finish_invalidation_execution_observation(&wrong_runtime)
        .is_err());
    first
        .cancel_observation_session(&wrong_runtime)
        .expect("foreign denial must leave the owner session available to cancel");
    let current = first.begin_invalidation_execution_observation().unwrap();
    assert!(matches!(
        first.begin_invalidation_execution_observation(),
        Err(crate::facade::SignalObservationAdmissionDenial::SessionAlreadyActive)
    ));
    assert!(first
        .finish_invalidation_execution_observation(&current)
        .is_err());
}

#[test]
fn topology_only_and_rejected_topology_observations_cannot_mint_execution_receipts() {
    let mut runtime = crate::facade::SignalRuntime::builder(crate::facade::SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let source = runtime.graph_mut().node().build();
    let consumer = runtime.graph_mut().node().build();

    let topology_only = runtime.begin_invalidation_execution_observation().unwrap();
    runtime
        .graph_mut()
        .set_dependencies(
            consumer,
            [crate::facade::DependencyEdge::new(
                source,
                crate::tests::support::ASPECT_A,
            )],
        )
        .unwrap();
    assert_eq!(
        runtime
            .graph()
            .invalidation_performed_counters()
            .value(InvalidationPerformedCounter::TopologyRevisionRevalidations),
        1
    );
    assert!(runtime
        .finish_invalidation_execution_observation(&topology_only)
        .unwrap_err()
        .to_string()
        .contains("no executed invalidation batch"));

    let rejected_only = runtime.begin_invalidation_execution_observation().unwrap();
    assert!(runtime
        .graph_mut()
        .set_dependencies(
            source,
            [crate::facade::DependencyEdge::new(
                consumer,
                crate::tests::support::ASPECT_A,
            )],
        )
        .is_err());
    assert_eq!(
        runtime
            .graph()
            .invalidation_performed_counters()
            .value(InvalidationPerformedCounter::RejectedTopologyMutations),
        1
    );
    assert!(runtime
        .finish_invalidation_execution_observation(&rejected_only)
        .unwrap_err()
        .to_string()
        .contains("no executed invalidation batch"));
}

#[test]
fn performed_rows_attach_to_foundational_and_drift_is_denied() {
    let definition = FinancialWorldDefinition::convergent_factor_batch(41, 0);
    let mut compiled = compile_financial_locality_world(definition).unwrap();
    let manifest = FinancialLocalityExpectationManifest::derive(
        compiled.locality_definition(),
        compiled.locality_graph_instance(),
    );
    let receipt_observation = compiled
        .locality_mut()
        .runtime
        .begin_invalidation_execution_observation()
        .unwrap();
    compiled.run_locality_action_trace(0).unwrap();
    let expected = expected_counters(&manifest);
    let receipt = compiled
        .locality()
        .runtime
        .finish_invalidation_execution_observation(&receipt_observation)
        .unwrap();
    let foundational =
        crate::data::proof::attach_foundational_invalidation_performance_receipt(receipt, expected)
            .unwrap();
    assert_eq!(foundational.counter_rows().len(), 23);

    let mut drifted = expected.values();
    drifted[InvalidationPerformedCounter::NodesEvaluated as usize] += 1;
    let mut drifted_world =
        compile_financial_locality_world(FinancialWorldDefinition::convergent_factor_batch(41, 0))
            .unwrap();
    let drifted_observation = drifted_world
        .locality_mut()
        .runtime
        .begin_invalidation_execution_observation()
        .unwrap();
    drifted_world.run_locality_action_trace(0).unwrap();
    let drifted_receipt = drifted_world
        .locality()
        .runtime
        .finish_invalidation_execution_observation(&drifted_observation)
        .unwrap();
    assert!(matches!(
        crate::data::proof::attach_foundational_invalidation_performance_receipt(
            drifted_receipt,
            SignalInvalidationRealizedCounters::from_values(drifted),
        ),
        Err(crate::data::proof::InvalidationFoundationalReceiptDenial::CounterRows(
            worth_foundational::FoundationalCounterBackedPerformanceReceiptConstructionDenial::CounterValueMismatch
        ))
    ));

    let mut recovery_world =
        compile_financial_locality_world(FinancialWorldDefinition::convergent_factor_batch(41, 0))
            .unwrap();
    let recovery_observation = recovery_world
        .locality_mut()
        .runtime
        .begin_invalidation_execution_observation()
        .unwrap();
    recovery_world.run_locality_action_trace(0).unwrap();
    let recovery_receipt = recovery_world
        .locality()
        .runtime
        .finish_invalidation_execution_observation(&recovery_observation)
        .unwrap();
    let mut laundered_recovery = expected.values();
    laundered_recovery[InvalidationPerformedCounter::RecoveryReconstructionWork as usize] = 1;
    assert!(matches!(
        crate::data::proof::attach_foundational_invalidation_performance_receipt(
            recovery_receipt,
            SignalInvalidationRealizedCounters::from_values(laundered_recovery),
        ),
        Err(crate::data::proof::InvalidationFoundationalReceiptDenial::ExcludedRecoveryWork)
    ));
}

#[test]
fn work_only_receipt_cannot_claim_counter_backed_foundational_evidence() {
    let mut compiled =
        compile_financial_locality_world(FinancialWorldDefinition::convergent_factor_batch(41, 0))
            .unwrap();
    let manifest = FinancialLocalityExpectationManifest::derive(
        compiled.locality_definition(),
        compiled.locality_graph_instance(),
    );
    let observation = compiled
        .locality_mut()
        .runtime
        .begin_observation_session(crate::facade::SignalObservationRequest::work())
        .unwrap();
    compiled.run_locality_action_trace(0).unwrap();
    let work_was_retained = !compiled
        .locality()
        .runtime
        .graph()
        .invalidation_performed_work()
        .is_empty();
    let receipt = compiled
        .locality()
        .runtime
        .finish_observation_session(&observation)
        .unwrap();
    assert!(crate::data::telemetry::InvalidationPerformedCounter::ALL
        .into_iter()
        .all(|counter| receipt.realized_counters().value(counter) == 0));
    assert!(work_was_retained);
    assert!(matches!(
        crate::data::proof::attach_foundational_invalidation_performance_receipt(
            receipt,
            expected_counters(&manifest),
        ),
        Err(crate::data::proof::InvalidationFoundationalReceiptDenial::ObservationSurfaceUnavailable)
    ));
}

#[test]
fn diagnostics_tiers_change_sidecar_policy_but_not_operational_rows() {
    let mut observations = Vec::new();
    for tier in [
        crate::facade::DiagnosticsTier::Operational,
        crate::facade::DiagnosticsTier::Development,
        crate::facade::DiagnosticsTier::Forensic,
    ] {
        let definition = FinancialWorldDefinition::convergent_factor_batch(41, 0);
        let mut compiled =
            super::compile_financial_locality_world_at_tier(definition, tier).unwrap();
        let baseline_fact_counts = compiled.locality_retained_fact_counts();
        let receipt_observation = compiled
            .locality_mut()
            .runtime
            .begin_invalidation_execution_observation()
            .unwrap();
        let observation = compiled.run_locality_action_trace(0).unwrap();
        let receipt = compiled
            .locality()
            .runtime
            .finish_invalidation_execution_observation(&receipt_observation)
            .unwrap();
        observations.push((
            compiled.locality().runtime.graph().runtime_policy().tier,
            *receipt.realized_counters(),
            compiled
                .locality()
                .runtime
                .graph()
                .observe()
                .recent_execution_history_diagnostics()
                .last()
                .cloned()
                .expect("performed locality execution emits a history sidecar"),
            compiled.committed_locality_financial_values().unwrap(),
            compiled
                .locality()
                .runtime
                .graph()
                .runtime_policy()
                .frontier_tracing_policy,
            baseline_fact_counts,
        ));
        assert_eq!(observation.performed_counters, *receipt.realized_counters());
    }
    assert_eq!(
        observations[0].0,
        crate::facade::DiagnosticsTier::Operational
    );
    assert_eq!(
        observations[1].0,
        crate::facade::DiagnosticsTier::Development
    );
    assert_eq!(observations[2].0, crate::facade::DiagnosticsTier::Forensic);
    assert_eq!(observations[0].1, observations[1].1);
    assert_eq!(observations[1].1, observations[2].1);
    assert!(observations[0].2.nodes.is_empty());
    assert!(!observations[1].2.nodes.is_empty());
    assert!(!observations[2].2.nodes.is_empty());
    assert_eq!(observations[0].2.profile, observations[0].0);
    assert_eq!(observations[1].2.profile, observations[1].0);
    assert_eq!(observations[2].2.profile, observations[2].0);
    assert_eq!(observations[0].3, observations[1].3);
    assert_eq!(observations[1].3, observations[2].3);
    assert_ne!(observations[0].4, observations[1].4);
    assert_ne!(observations[1].4, observations[2].4);
    assert_eq!(observations[0].5, (0, 0));
    assert!(observations[1].5 .0 > 0);
    assert!(observations[1].5 .1 > 0);
}

#[test]
fn ordinary_hot_path_worlds_match_their_independent_twenty_four_row_manifests() {
    let cases = ordinary_locality_cases();
    for scenario in [
        FinancialLocalityScenario::SparseBookFanout,
        FinancialLocalityScenario::PartitionedCurveUniverse,
        FinancialLocalityScenario::ConvergentFactorBatch,
        FinancialLocalityScenario::DenseMarketClose,
    ] {
        let case = cases
            .iter()
            .find(|case| case.scenario() == scenario)
            .copied()
            .unwrap();
        let definition = FinancialWorldDefinition::locality_case(41, case);
        let mut compiled = compile_financial_locality_world(definition).unwrap();
        let manifest = FinancialLocalityExpectationManifest::derive(
            compiled.locality_definition(),
            compiled.locality_graph_instance(),
        );
        let observation = compiled.run_locality_action_trace(0).unwrap();
        let expected = expected_counters(&manifest);
        for counter in InvalidationPerformedCounter::ALL {
            assert_eq!(
                observation.performed_counters.value(counter),
                expected.value(counter),
                "{:?} drifted for {}",
                case.scenario(),
                counter.name(),
            );
        }
    }
}

fn expected_counters(
    manifest: &FinancialLocalityExpectationManifest,
) -> SignalInvalidationRealizedCounters {
    let expected_rows = ExpectedLocalityCounterRow::ALL;
    SignalInvalidationRealizedCounters::from_values(std::array::from_fn(|index| {
        manifest.counter_manifest().value(expected_rows[index])
    }))
}

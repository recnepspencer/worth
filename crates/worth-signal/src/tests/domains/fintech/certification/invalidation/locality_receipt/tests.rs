use super::*;
use crate::tests::domains::fintech::world::{
    ordinary_locality_cases, retained_locality_benchmark_cases, scheduled_locality_cases,
    SparseFanoutAxis,
};

#[test]
fn checked_rejected_descendants_settle_at_the_declared_4096_output_scale() {
    let evidence = verify_locality_case(
        FinancialWorldDefinition::sparse_book_fanout(
            41,
            4_096,
            SparseFanoutAxis::RejectedDescendants,
        ),
        0,
        DiagnosticsTier::Operational,
        1,
    )
    .unwrap();
    assert_eq!(
        evidence.scenario(),
        FinancialLocalityScenario::SparseBookFanout
    );
    assert_eq!(
        evidence.scale(),
        LocalityScaleTuple::SparseBookFanout {
            total_outputs: 4_096,
            axis: SparseFanoutAxis::RejectedDescendants,
        }
    );
    assert!(evidence.necessary_evaluation_count() > 0);
}

#[test]
fn verified_case_binds_fresh_truth_necessity_work_and_performed_receipt() {
    let evidence = verify_locality_case(
        FinancialWorldDefinition::convergent_factor_batch(41, 0),
        0,
        DiagnosticsTier::Operational,
        1,
    )
    .unwrap();

    assert_eq!(
        evidence.scenario(),
        FinancialLocalityScenario::ConvergentFactorBatch
    );
    assert_eq!(evidence.lane(), LocalityLane::OrdinaryChangeGate);
    assert_ne!(evidence.canonical_work_items(), 0);
    assert!(!evidence.necessary_evaluations().is_empty());
    assert_ne!(evidence.identity().digest_bytes(), &[0; 32]);
}

#[test]
fn physical_ready_shape_and_counter_drift_are_rejected_independently() {
    let mut compiled =
        compile_financial_locality_world(FinancialWorldDefinition::convergent_factor_batch(41, 0))
            .unwrap();
    let manifest = FinancialLocalityExpectationManifest::derive_for_trace(
        compiled.locality_definition(),
        &compiled.locality_definition().action_traces()[0],
        compiled.locality_graph_instance(),
    );
    let fresh = FreshFinancialLocalityRecompute::run_for_trace(
        compiled.locality_definition(),
        &compiled.locality_definition().action_traces()[0],
    );
    let (mut observation, _) = compiled
        .observe_locality_action_trace_with_workers(0, 1)
        .unwrap();
    validate_case_results(&compiled, &manifest, &fresh, &observation).unwrap();

    observation.physical_ready_batches += 1;
    let wrong_shape =
        validate_case_results(&compiled, &manifest, &fresh, &observation).unwrap_err();
    assert!(wrong_shape.to_string().contains("batch_local_allocations"));
    observation.physical_ready_batches -= 1;

    let mut values = observation.performed_counters.values();
    values[InvalidationPerformedCounter::PeakBatchMemoryItems.index()] += 1;
    observation.performed_counters = SignalInvalidationRealizedCounters::from_values(values);
    let wrong_counter =
        validate_case_results(&compiled, &manifest, &fresh, &observation).unwrap_err();
    assert!(wrong_counter
        .to_string()
        .contains("peak_batch_memory_items"));
}

#[test]
fn ordinary_lifecycle_cases_bind_their_runtime_trace_to_the_manifest() {
    for scenario in [
        FinancialLocalityScenario::PortfolioDependencyChurn,
        FinancialLocalityScenario::BranchRestoreLocalityReplay,
    ] {
        let case = ordinary_locality_cases()
            .into_iter()
            .find(|case| case.scenario() == scenario)
            .unwrap();
        verify_locality_case(
            FinancialWorldDefinition::locality_case(41, case),
            0,
            DiagnosticsTier::Operational,
            1,
        )
        .unwrap();
    }
}

#[test]
fn partitioned_primary_trace_matches_independent_financial_truth() {
    verify_locality_case(
        FinancialWorldDefinition::partitioned_curve_universe(41, 16, 1, 1),
        0,
        DiagnosticsTier::Operational,
        1,
    )
    .unwrap();
}

#[test]
fn partitioned_family_matches_truth_and_necessity_at_every_ordinary_scale() {
    for case in ordinary_locality_cases()
        .into_iter()
        .filter(|case| case.scenario() == FinancialLocalityScenario::PartitionedCurveUniverse)
    {
        let definition = FinancialWorldDefinition::locality_case(41, case);
        let trace_count = definition.locality().unwrap().action_traces().len();
        for trace_index in 0..trace_count {
            verify_locality_case(
                FinancialWorldDefinition::locality_case(41, case),
                trace_index,
                DiagnosticsTier::Operational,
                1,
            )
            .unwrap();
        }
    }
}

#[test]
fn restore_case_crosses_a_real_checkpoint_boundary_before_completion() {
    let cases = ordinary_locality_cases()
        .into_iter()
        .filter(|case| case.scenario() == FinancialLocalityScenario::BranchRestoreLocalityReplay)
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), 3);
    for case in cases {
        let mut compiled =
            compile_financial_locality_world(FinancialWorldDefinition::locality_case(41, case))
                .unwrap();
        let _evidence = compiled.certify_restore_locality_lifecycle().unwrap();
    }
}

#[test]
#[ignore = "scheduled 100,000-output dense scale courtroom"]
fn scheduled_dense_quarter_case_seals_in_isolation() {
    let generation_started = std::time::Instant::now();
    let case = scheduled_locality_cases()
        .into_iter()
        .find(|case| {
            case.scale
                == LocalityScaleTuple::DenseMarketClose {
                    total_outputs: 100_000,
                    affected_ratio: crate::tests::domains::fintech::world::DensityRatio::OneInFour,
                }
        })
        .expect("scheduled scale contract must retain the 100K quarter-density case");
    let definition = FinancialWorldDefinition::locality_case(41, case);
    eprintln!(
        "M13 scheduled step: definition generated elapsed_ms={}",
        generation_started.elapsed().as_millis()
    );
    verify_locality_case(definition, 0, DiagnosticsTier::Operational, 1).unwrap();
}

#[test]
#[ignore = "retained 100,000-output restore benchmark artifact"]
fn retained_dense_restore_benchmark_covers_all_declared_seeds() {
    let case = retained_locality_benchmark_cases()[0];
    let LocalityScaleTuple::BranchRestoreLocalityReplay {
        canonical_seeds, ..
    } = case.scale
    else {
        panic!("retained benchmark must be a restore case")
    };
    for seed in 41..41 + u64::from(canonical_seeds) {
        let definition = FinancialWorldDefinition::locality_case(seed, case);
        verify_locality_case(
            FinancialWorldDefinition::locality_case(seed, case),
            0,
            DiagnosticsTier::Operational,
            1,
        )
        .unwrap();
        let mut compiled = compile_financial_locality_world(definition).unwrap();
        compiled.certify_restore_locality_lifecycle().unwrap();
    }
}

#[test]
fn mismatched_fresh_oracle_and_scale_manifest_are_denied() {
    let definition = FinancialWorldDefinition::convergent_factor_batch(41, 0);
    let mut compiled = compile_financial_locality_world(definition).unwrap();
    let manifest = FinancialLocalityExpectationManifest::derive(
        compiled.locality_definition(),
        compiled.locality_graph_instance(),
    );
    let wrong_definition = FinancialWorldDefinition::convergent_factor_batch(43, 1);
    let wrong_fresh = FreshFinancialLocalityRecompute::run_for_trace(
        wrong_definition.locality().unwrap(),
        &wrong_definition.locality().unwrap().action_traces()[0],
    );
    let wrong_manifest = FinancialLocalityExpectationManifest::derive_for_trace(
        wrong_definition.locality().unwrap(),
        &wrong_definition.locality().unwrap().action_traces()[0],
        compiled.locality_graph_instance(),
    );
    let (observation, _performed) = compiled
        .observe_locality_action_trace_with_workers(0, 1)
        .unwrap();

    assert!(validate_case_results(&compiled, &manifest, &wrong_fresh, &observation).is_err());
    assert!(validate_case_results(
        &compiled,
        &wrong_manifest,
        &FreshFinancialLocalityRecompute::run_for_trace(
            compiled.locality_definition(),
            &compiled.locality_definition().action_traces()[0],
        ),
        &observation,
    )
    .is_err());
}

use std::collections::BTreeSet;

use crate::facade::DiagnosticsTier;
use crate::tests::domains::fintech::certification::invalidation::{
    FinancialNecessityManifest, FreshFinancialRecompute,
};
use crate::tests::domains::fintech::scenarios::setup_world;
use crate::tests::domains::fintech::world::{
    compile_financial_world, CurveBucket, FinancialConsumerRole, FinancialWorldDefinition, FxPair,
    InstrumentId, MarketFactorKey, SemanticOutputKey,
};

#[test]
fn partitioned_curve_bucket_bump() {
    let base = FinancialWorldDefinition::partition_courtroom(41);
    let usd = MarketFactorKey::Curve(CurveBucket::UsdOneYear);
    let eur = MarketFactorKey::Curve(CurveBucket::EurOneYear);
    let after_usd = base.with_market_factor_delta(usd, 4);
    let after_both = after_usd.with_market_factor_delta(eur, 7);
    let instrument = InstrumentId("EURUSD-1Y-FWD");
    let baseline = compile_financial_world(base.clone()).unwrap();
    let mut compiled = baseline.into_compiled();
    let evidence = compiled
        .apply_factor_change_sequence(&[(after_usd, usd), (after_both.clone(), eur)], instrument)
        .unwrap();

    let details = evidence
        .pending_scopes()
        .iter()
        .filter_map(|scope| scope.path().segments().get(1).cloned())
        .collect::<BTreeSet<_>>();
    assert_eq!(details, BTreeSet::from(["usd-1y".into(), "eur-1y".into()]));
    assert!(evidence.gated_consumer_was_pending());
    assert_eq!(
        compiled.economic_snapshot(),
        &FreshFinancialRecompute::run(&after_both).economic_snapshot()
    );
    let required = [usd, eur]
        .into_iter()
        .flat_map(|factor| {
            FinancialNecessityManifest::derive(&base, factor)
                .required_work()
                .into_iter()
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(compiled.ledger().observed_work(), required);

    let mut legacy_locality_twin = setup_world();
    legacy_locality_twin.shock_rates_bucket_one(7).unwrap();
}

#[test]
fn gated_repricing_release() {
    let factor = MarketFactorKey::FxSpot(FxPair::EurUsd);
    let instrument = InstrumentId("EURUSD-1Y-FWD");
    let consumer = SemanticOutputKey::Consumer(FinancialConsumerRole::RiskThreshold);
    let base = FinancialWorldDefinition::gated_courtroom(41);

    let small_one = base.with_market_factor_delta(factor, 1_000);
    let small_two = base.with_market_factor_delta(factor, 2_000);
    let mut small = compile_financial_world(base.clone())
        .unwrap()
        .into_compiled();
    let small_evidence = small
        .apply_gated_factor_sequence(
            &[(small_one, factor), (small_two.clone(), factor)],
            instrument,
            FinancialConsumerRole::RiskThreshold,
        )
        .unwrap();
    assert_eq!(small_evidence.revision_delta(), 2);
    assert_eq!(
        small.economic_snapshot(),
        &FreshFinancialRecompute::run(&small_two).economic_snapshot()
    );
    assert_eq!(
        small.ledger().observed_work(),
        FinancialNecessityManifest::derive_for_revision_delta(&base, factor, 2).required_work()
    );
    assert!(!small.ledger().observed_work().contains(&consumer));

    let large_one = base.with_market_factor_delta(factor, 1_000);
    let large_two = base.with_market_factor_delta(factor, 2_000);
    let large_three = base.with_market_factor_delta(factor, 3_000);
    let mut large = compile_financial_world(base.clone())
        .unwrap()
        .into_compiled();
    let large_evidence = large
        .apply_gated_factor_sequence(
            &[
                (large_one, factor),
                (large_two, factor),
                (large_three.clone(), factor),
            ],
            instrument,
            FinancialConsumerRole::RiskThreshold,
        )
        .unwrap();
    assert_eq!(large_evidence.revision_delta(), 3);
    assert_eq!(
        large.economic_snapshot(),
        &FreshFinancialRecompute::run(&large_three).economic_snapshot()
    );
    assert_eq!(
        large.ledger().observed_work(),
        FinancialNecessityManifest::derive_for_revision_delta(&base, factor, 3).required_work()
    );
    assert!(large.ledger().observed_work().contains(&consumer));
}

#[test]
fn instrument_dependency_rewire() {
    let instrument = InstrumentId("EURUSD-1Y-FWD");
    let base = FinancialWorldDefinition::deterministic(41);
    let old_factor = MarketFactorKey::Curve(CurveBucket::UsdOneYear);
    let cause_definition = base.with_market_factor_delta(old_factor, 100);
    let final_definition =
        cause_definition.with_fx_forward_domestic_curve(instrument, CurveBucket::UsdTwoYear);
    let mut compiled = compile_financial_world(base).unwrap().into_compiled();
    let evidence = compiled
        .apply_instrument_dependency_rewire(
            cause_definition.clone(),
            old_factor,
            final_definition.clone(),
            instrument,
        )
        .unwrap();

    assert!(evidence.stale_cause_rejected());
    assert!(evidence.cycle_rejected());
    assert_ne!(evidence.stale_revision(), evidence.final_revision());
    assert_eq!(
        compiled.economic_snapshot(),
        &FreshFinancialRecompute::run(&final_definition).economic_snapshot()
    );
    let mut necessary_work = FinancialNecessityManifest::derive_dependency_rewire(
        &cause_definition,
        &final_definition,
        instrument,
    );
    necessary_work.insert(SemanticOutputKey::Factor(old_factor));
    assert_eq!(compiled.ledger().observed_work(), necessary_work);
}

#[test]
fn branch_shock_restore_replay() {
    let base = FinancialWorldDefinition::deterministic(41);
    let analysis_factor = MarketFactorKey::FxSpot(FxPair::EurUsd);
    let main_factor = MarketFactorKey::Curve(CurveBucket::UsdOneYear);
    let analysis = base.with_market_factor_delta(analysis_factor, 20_000);
    let main = base.with_market_factor_delta(main_factor, 100);
    let instrument = InstrumentId("EURUSD-1Y-FWD");
    let mut compiled = compile_financial_world(base.clone())
        .unwrap()
        .into_compiled();
    let evidence = compiled
        .exercise_branch_restore_replay(
            base.clone(),
            analysis.clone(),
            analysis_factor,
            main,
            main_factor,
            instrument,
            DiagnosticsTier::Development,
        )
        .unwrap();

    assert!(evidence.verifies_lifecycle(), "{evidence:#?}");
    assert!(!evidence.analysis_causes_before_capture.is_empty());
    assert_eq!(
        evidence.analysis_causes_after_restore,
        evidence.analysis_causes_before_capture
    );
    assert!(evidence.main_pending_isolated);
    assert!(evidence.async_dependency_blocked);
    assert!(evidence.replay_branch_local);
    assert!(evidence.replay_has_restore);
    assert_eq!(
        evidence.final_snapshot,
        FreshFinancialRecompute::run(&analysis).economic_snapshot()
    );
    assert_eq!(
        evidence.observed_work,
        FinancialNecessityManifest::derive(&base, analysis_factor).required_work()
    );
}

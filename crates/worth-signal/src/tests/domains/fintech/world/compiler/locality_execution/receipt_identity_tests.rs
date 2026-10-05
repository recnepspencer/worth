use crate::tests::domains::fintech::certification::invalidation::{
    verified_locality_case_identity, FinancialCanonicalCaseIdentity,
    FinancialCanonicalReportIdentity, FinancialLocalityExpectationManifest,
};
use crate::tests::domains::fintech::world::{
    compile_financial_locality_world, ordinary_locality_cases, FinancialLocalityScenario,
    FinancialWorldDefinition,
};

#[test]
fn canonical_case_identity_is_bound_to_verified_performed_execution() {
    let first = verified_case_identity(FinancialWorldDefinition::convergent_factor_batch(41, 0));
    let repeated = verified_case_identity(FinancialWorldDefinition::convergent_factor_batch(41, 0));
    let distinct = verified_case_identity(FinancialWorldDefinition::convergent_factor_batch(43, 0));

    assert_eq!(first.digest_bytes(), repeated.digest_bytes());
    assert_ne!(first.digest_bytes(), distinct.digest_bytes());
}

#[test]
fn canonical_case_identity_changes_with_tier_and_exact_trace() {
    let operational = verified_case_identity_for(
        FinancialWorldDefinition::convergent_factor_batch(41, 0),
        0,
        crate::facade::DiagnosticsTier::Operational,
    );
    let development = verified_case_identity_for(
        FinancialWorldDefinition::convergent_factor_batch(41, 0),
        0,
        crate::facade::DiagnosticsTier::Development,
    );
    let second_permutation = verified_case_identity_for(
        FinancialWorldDefinition::convergent_factor_batch(41, 0),
        1,
        crate::facade::DiagnosticsTier::Operational,
    );

    assert_ne!(operational.digest_bytes(), development.digest_bytes());
    assert_ne!(
        operational.digest_bytes(),
        second_permutation.digest_bytes()
    );
}

#[test]
fn canonical_report_identity_is_order_invariant() {
    let convergent =
        verified_case_identity(FinancialWorldDefinition::convergent_factor_batch(41, 0));
    let sparse_case = ordinary_locality_cases()
        .into_iter()
        .find(|case| case.scenario() == FinancialLocalityScenario::SparseBookFanout)
        .unwrap();
    let sparse = verified_case_identity(FinancialWorldDefinition::locality_case(41, sparse_case));

    let forward = FinancialCanonicalReportIdentity::from_cases([&convergent, &sparse]).unwrap();
    let reverse = FinancialCanonicalReportIdentity::from_cases([&sparse, &convergent]).unwrap();
    assert_eq!(forward.digest_bytes(), reverse.digest_bytes());
    assert!(FinancialCanonicalReportIdentity::from_cases([&sparse, &sparse]).is_err());
    assert!(
        FinancialCanonicalReportIdentity::from_cases(std::iter::empty::<
            &FinancialCanonicalCaseIdentity,
        >())
        .is_err()
    );
}

fn verified_case_identity(definition: FinancialWorldDefinition) -> FinancialCanonicalCaseIdentity {
    verified_case_identity_for(definition, 0, crate::facade::DiagnosticsTier::Operational)
}

fn verified_case_identity_for(
    definition: FinancialWorldDefinition,
    trace_index: usize,
    tier: crate::facade::DiagnosticsTier,
) -> FinancialCanonicalCaseIdentity {
    let mut compiled = compile_financial_locality_world(definition).unwrap();
    compiled
        .locality_mut()
        .runtime
        .graph_mut()
        .reset_runtime_policy_to_tier(tier);
    let manifest = FinancialLocalityExpectationManifest::derive_for_trace(
        compiled.locality_definition(),
        &compiled.locality_definition().action_traces()[trace_index],
        compiled.locality_graph_instance(),
    );
    let receipt_observation = compiled
        .locality_mut()
        .runtime
        .begin_invalidation_execution_observation()
        .unwrap();
    let observed = compiled.run_locality_action_trace(trace_index).unwrap();
    let performed = compiled
        .locality()
        .runtime
        .finish_invalidation_execution_observation(&receipt_observation)
        .unwrap();
    verified_locality_case_identity(
        compiled.locality_definition(),
        &manifest,
        compiled.locality().runtime.graph().runtime_policy().tier,
        performed,
        observed.physical_ready_batches,
        observed.physical_ready_peak,
    )
    .unwrap()
}

use super::super::world::{compile_financial_locality_world_at_tier, FinancialWorldDefinition};

#[test]
fn operational_authority_digest_is_independent_of_diagnostic_tier() {
    let definition = FinancialWorldDefinition::convergent_factor_batch(41, 0);
    let mut digests = Vec::new();
    for tier in [
        crate::facade::DiagnosticsTier::Operational,
        crate::facade::DiagnosticsTier::Development,
        crate::facade::DiagnosticsTier::Forensic,
    ] {
        let mut compiled = compile_financial_locality_world_at_tier(definition.clone(), tier)
            .expect("M10 tier world should compile");
        let (observation, _) = compiled
            .observe_locality_action_trace_with_workers(0, 1)
            .expect("M10 tier action should settle");
        digests.push(
            compiled
                .locality_operational_digest_with_work(&observation.performed_work)
                .expect("operational authority digest should derive"),
        );
    }
    assert_eq!(digests[0], digests[1]);
    assert_eq!(digests[1], digests[2]);
}

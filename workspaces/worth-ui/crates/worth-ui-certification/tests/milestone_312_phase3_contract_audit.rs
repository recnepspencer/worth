use crate::milestone_312_ledger as ledger;
use crate::repository_document;

#[path = "milestone_312_phase3_contract_audit/planning_cutover.rs"]
mod planning_cutover;

fn phase_3_contract() -> toml::Value {
    toml::from_str(&repository_document(
        "plans/worth-ui/milestone-3.12-phase-3-contract.toml",
    ))
    .expect("Milestone 3.12 Phase 3 contract is TOML")
}

#[test]
fn milestone_312_phase3_contract_and_ledger_are_exact() {
    let contract = phase_3_contract();
    assert_eq!(
        contract["schema"].as_str(),
        Some("worth-ui.milestone-3.12.phase-3-contract.v1")
    );
    assert_eq!(contract["milestone"].as_str(), Some("3.12"));
    assert_eq!(contract["phase"].as_integer(), Some(3));
    assert!(matches!(
        contract["status"].as_str(),
        Some("implementation" | "closed")
    ));
    assert!(contract["closure_claim"]
        .as_str()
        .is_some_and(|claim| !claim.trim().is_empty()));
    let text = repository_document("plans/worth-ui/milestone-3.12-phase-3-proof-ledger.csv");
    ledger::validate_phase_3(&contract, &text)
        .unwrap_or_else(|failure| panic!("Phase 3 ledger is invalid: {failure}"));
}

#[test]
fn milestone_312_phase3_ledger_mutations_cannot_manufacture_closure() {
    let mut contract = phase_3_contract();
    contract["status"] = toml::Value::String("closed".to_owned());
    let open = repository_document("plans/worth-ui/milestone-3.12-phase-3-proof-ledger.csv");
    let closed = ledger::prove_all(&open).expect("build a valid closed Phase 3 ledger");
    ledger::validate_phase_3(&contract, &closed).expect("closed Phase 3 fixture is valid");

    for (label, mutation) in ledger::hostile_mutations(&closed) {
        assert!(
            ledger::validate_phase_3(&contract, &mutation).is_err(),
            "{label} mutation manufactured Phase 3 closure"
        );
    }
}

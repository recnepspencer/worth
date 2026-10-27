use super::*;

#[test]
fn every_complexity_contract_proof_names_a_registered_test() {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .arg("--list")
        .output()
        .expect("list this exact test executable's registered proofs");
    assert!(output.status.success());
    let listing = String::from_utf8(output.stdout).unwrap();
    let registered = listing
        .lines()
        .filter_map(|line| line.strip_suffix(": test"))
        .collect::<std::collections::BTreeSet<_>>();
    let runtime = runtime_with_test_schema();
    for contract in runtime.performance_access().contracts() {
        assert!(
            !contract.proof_tests.is_empty(),
            "{} has no proof",
            contract.id
        );
        for proof in contract.proof_tests {
            assert!(
                registered.contains(proof),
                "{} cites missing test {proof}",
                contract.id
            );
        }
    }
}

#[test]
fn complexity_contract_registry_covers_runtime_hot_paths() {
    let runtime = runtime_with_test_schema();
    let contracts = runtime.performance_access().contracts();

    assert!(contracts.len() >= 6);
    assert!(contracts
        .iter()
        .all(|contract| !contract.proof_tests.is_empty()));
    assert!(contracts
        .iter()
        .any(|contract| contract.id == "runtime.partition_local_commit"));
    assert!(contracts
        .iter()
        .any(|contract| contract.id == "runtime.slot_local_mutation_journal"));
    assert!(contracts
        .iter()
        .any(|contract| contract.id == "runtime.relation_identity_validation"));
    assert!(contracts
        .iter()
        .any(|contract| contract.id == "runtime.unique_entity_invariant_lookup"));
    assert!(contracts
        .iter()
        .any(|contract| contract.id == "runtime.partition_edition.acquire"));
    assert!(contracts
        .iter()
        .any(|contract| contract.id == "runtime.snapshot_pin_maintenance"));
}

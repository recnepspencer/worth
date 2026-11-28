use super::super::support::*;

#[test]
fn branch_state_proof_is_versioned_and_stable_for_unchanged_branch_state() {
    let mut runtime = RuntimeCore::new(RuntimePolicySpec::default()).unwrap();
    runtime
        .define_source(SourceSpec {
            id: "gearTeeth".to_owned(),
            initial: SignalValue::Number(16.0),
            produces_aspects: None,
        })
        .unwrap();
    runtime
        .define_source(SourceSpec {
            id: "gearThickness".to_owned(),
            initial: SignalValue::Number(0.42),
            produces_aspects: None,
        })
        .unwrap();

    let branch = runtime.current_branch();
    let left = runtime.branch_state_proof(branch.id.0).unwrap();
    let right = runtime.branch_state_proof(branch.id.0).unwrap();

    assert!(left
        .proof_schema_version
        .starts_with(worth_signal::facade::adapters::BRANCH_STATE_PROOF_SCHEMA_VERSION));
    assert_eq!(left.proof_schema_version, right.proof_schema_version);
    assert_eq!(left.branch_id, right.branch_id);
    assert_eq!(left.state_digest, right.state_digest);
}

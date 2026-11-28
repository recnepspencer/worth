use super::super::support::*;

#[test]
fn replay_parity_proof_distinguishes_equivalent_and_divergent_branch_states() {
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

    let main = runtime.current_branch();
    let twin = runtime.create_branch("twin".to_owned()).unwrap();

    let parity = runtime.replay_parity_proof(main.id.0, twin.id.0).unwrap();
    assert_eq!(
        parity.proof_schema_version,
        worth_signal::facade::adapters::BRANCH_STATE_PROOF_SCHEMA_VERSION
    );
    assert!(parity.parity);
    assert_eq!(parity.expected_state_digest, parity.replayed_state_digest);

    runtime.switch_branch(twin.id.0).unwrap();
    runtime
        .apply_transaction(vec![TransactionOp::Set {
            id: "gearTeeth".to_owned(),
            value: SignalValue::Number(32.0),
            aspect: None,
            aspects: None,
        }])
        .unwrap();

    let divergent = runtime.replay_parity_proof(main.id.0, twin.id.0).unwrap();
    assert!(!divergent.parity);
    assert_ne!(
        divergent.expected_state_digest,
        divergent.replayed_state_digest
    );
}

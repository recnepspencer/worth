use super::*;
use fixture::{definition_world_with_bridge_budget, source_free_request};
use worth_runtime_bridge::facade::{
    BridgeConditionalDenial, BridgeConditionalDenialKind, BridgeConditionalRetentionBudget,
    BridgePreparedConditionalInstallationExtension,
};

fn budget(definitions: usize, bytes: u64) -> BridgeConditionalRetentionBudget {
    BridgeConditionalRetentionBudget {
        maximum_retained_definition_candidates: definitions,
        maximum_retained_bytes: bytes,
        ..BridgeConditionalRetentionBudget::development()
    }
}

fn prepare(
    fixture: &fixture::DefinitionWorld,
) -> Result<BridgePreparedConditionalInstallationExtension, BridgeConditionalDenial> {
    let predecessor = fixture
        .bridge
        .admit_conditional_signal_basis(&fixture.lowering, fixture.root.basis().signal_basis())?;
    fixture
        .bridge
        .prepare_owned_conditional_definition_successor(&predecessor, source_free_request(2))
}

#[test]
fn definition_candidate_slot_is_claimed_before_effect_and_released_on_drop() {
    let fixture = definition_world_with_bridge_budget(budget(1, 64 * 1024 * 1024));
    let lifecycle = fixture.bridge.conditional_lifecycle_probe();
    let candidate = prepare(&fixture).unwrap();
    let held = lifecycle.bridge_conditional_retention().unwrap();
    assert_eq!(held.retained_definition_candidates(), 1);
    assert!(held.retained_bytes() > 0);

    let denial = match prepare(&fixture) {
        Ok(_) => panic!("a one-slot budget admitted a second definition candidate"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        BridgeConditionalDenialKind::ConditionalRetentionCapacity
    );
    assert!(denial.detail().contains("DefinitionCandidatesExhausted"));
    drop(candidate);
    let released = lifecycle.bridge_conditional_retention().unwrap();
    assert_eq!(released.retained_definition_candidates(), 0);
    assert_eq!(released.retained_bytes(), 0);
    drop(prepare(&fixture).expect("dropping a candidate restores its exact slot"));
}

#[test]
fn definition_candidate_byte_denial_precedes_signal_or_product_movement() {
    let fixture = definition_world_with_bridge_budget(budget(1, 1));
    let lifecycle = fixture.bridge.conditional_lifecycle_probe();
    let before = fixture
        .owner
        .observation_port()
        .observe_product_branch(fixture.root.branch_identity())
        .unwrap();
    let denial = match prepare(&fixture) {
        Ok(_) => panic!("a one-byte budget admitted a real definition candidate"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        BridgeConditionalDenialKind::ConditionalRetentionCapacity
    );
    assert!(denial.detail().contains("BytesExhausted"));
    let after = fixture
        .owner
        .observation_port()
        .observe_product_branch(fixture.root.branch_identity())
        .unwrap();
    assert_eq!(before.snapshot(), after.snapshot());
    assert_eq!(fixture.lowering.signal_definition_generation(), 1);
    let released = lifecycle.bridge_conditional_retention().unwrap();
    assert_eq!(released.retained_definition_candidates(), 0);
    assert_eq!(released.retained_bytes(), 0);
}

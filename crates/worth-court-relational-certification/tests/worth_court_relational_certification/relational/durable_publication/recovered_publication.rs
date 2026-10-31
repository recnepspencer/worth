use super::*;
use worth_relational::facade::durability::RelationalRecoveredPublicationAdmissionCause as Cause;

#[test]
fn recovered_publication_transfers_real_empty_predecessor_and_original_successor_custody() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let (_source, medium, predecessor, successor, association, semantic) =
        source_journey(execution);
    let offers = medium.offered().len();
    let (runtime, authority) = recover(execution, &medium);
    let owner = runtime.durability_authority();
    let subject = owner
        .discover_associated_effects(execution, &association)
        .unwrap()
        .pop()
        .unwrap();
    let effect = owner
        .readmit_associated_effect(execution, &subject)
        .unwrap();
    let proof = authority
        .admit_recorded_publication(execution, &runtime, effect, &predecessor, &successor)
        .unwrap();
    assert_eq!(proof.custody().effect().semantic_result(), &semantic);
    assert_recorded_basis(&runtime, proof.custody().predecessor(), &predecessor);
    assert_recorded_basis(&runtime, proof.custody().successor(), &successor);
    assert!(
        runtime
            .owner_component_services()
            .basis_port()
            .compare_current_exact(proof.custody().successor())
            .is_err(),
        "later head is not substituted for the original effect"
    );
    let refused = runtime.prepare_close().unwrap_err();
    assert_eq!(
        refused.reason(),
        worth_relational::facade::runtime::RelationalRuntimeCloseDenial::ReadCustodyRetained
    );
    let runtime = refused.into_runtime();
    proof.consume(|custody| {
        let (effect, before, before_lease, after, after_lease) = custody.into_retained_parts();
        assert_eq!(effect.subject().association(), Some(&association));
        assert_eq!(effect.semantic_result(), &semantic);
        for (basis, lease) in [(before, before_lease), (after, after_lease)] {
            assert_eq!(
                runtime
                    .readmit_retained_branch_basis(basis.descriptor(), &lease)
                    .unwrap(),
                basis
            );
            let snapshot = runtime
                .snapshots()
                .snapshot_for_observation(&basis.observation())
                .unwrap();
            runtime.snapshots().release_snapshot(&snapshot).unwrap();
            runtime.release_component_basis(lease).unwrap();
        }
    });
    assert_eq!(
        medium.offered().len(),
        offers,
        "readmission never republishes the Native effect"
    );
}

#[test]
fn recovered_publication_refusal_preserves_source_and_partial_pin_for_fresh_request() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let (_source, medium, predecessor, successor, association, semantic) =
        source_journey(execution);
    let (runtime, authority) = recover(execution, &medium);
    let owner = runtime.durability_authority();
    let subject = owner
        .discover_associated_effects(execution, &association)
        .unwrap()
        .pop()
        .unwrap();
    let effect = owner
        .readmit_associated_effect(execution, &subject)
        .unwrap();
    let foreign_medium = TestDurableMedium::new();
    let foreign = durable_runtime(&foreign_medium);
    let refusal = authority
        .admit_recorded_publication(execution, &foreign, effect, &predecessor, &successor)
        .unwrap_err();
    assert!(matches!(refusal.cause(), Cause::ForeignRecoveryRuntime));
    let effect = refusal.into_effect();
    let mut changed = serde_json::to_value(&successor).unwrap();
    let original = changed["descriptor"]["visibility_commitment"][0]
        .as_u64()
        .unwrap();
    changed["descriptor"]["visibility_commitment"][0] = serde_json::json!(original ^ 1);
    let changed: Recorded = serde_json::from_value(changed).unwrap();
    let refusal = authority
        .admit_recorded_publication(execution, &runtime, effect, &predecessor, &changed)
        .unwrap_err();
    assert!(matches!(refusal.cause(), Cause::Component(_)));
    assert!(
        refusal.retains_acquired_custody(),
        "the genuine predecessor pin survives successor refusal"
    );
    assert_eq!(refusal.effect().semantic_result(), &semantic);
    let original_receipt = refusal.effect().subject().receipt().clone();
    let effect = refusal.into_effect();
    let fresh_host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let fresh = &ExecutionRequest::serial(&fresh_host);
    let proof = authority
        .admit_recorded_publication(fresh, &runtime, effect, &predecessor, &successor)
        .unwrap();
    assert_eq!(
        proof.custody().effect().subject().receipt(),
        &original_receipt
    );
    proof.consume(|custody| {
        let (_, _, before, _, after) = custody.into_retained_parts();
        runtime.release_component_basis(before).unwrap();
        runtime.release_component_basis(after).unwrap();
    });
}

#[test]
fn recovered_publication_requires_actual_settlement_and_this_recoverys_exact_images() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let (_source, medium, _, _, _, _) = source_journey(execution);
    let (runtime, authority) = recover(execution, &medium);
    let predecessor = current(execution, &runtime);
    let association = Association::from_bytes(b"performed-after-this-recovery").unwrap();
    let performed = perform(
        execution,
        &runtime,
        &association,
        "post-recovery-native-effect",
    );
    let successor = runtime
        .owner_component_services()
        .basis_port()
        .export_recorded_retained_component(execution, performed.next_basis())
        .unwrap();
    let receipt = performed.canonical_commit().commit.clone();
    let offers = medium.offered().len();
    let owner = runtime.durability_authority();
    let subject = owner
        .discover_associated_effects(execution, &association)
        .unwrap()
        .pop()
        .unwrap();
    let effect = owner
        .readmit_associated_effect(execution, &subject)
        .unwrap();
    let refusal = authority
        .admit_recorded_publication(execution, &runtime, effect, &predecessor, &successor)
        .unwrap_err();
    assert!(matches!(refusal.cause(), Cause::PendingSettlement));
    drop(performed);
    assert_eq!(
        runtime
            .repair_pending_publication_settlement(receipt.commit_id)
            .unwrap(),
        receipt
    );
    let effect = refusal.into_effect();
    let refusal = authority
        .admit_recorded_publication(execution, &runtime, effect, &predecessor, &successor)
        .unwrap_err();
    assert!(matches!(refusal.cause(), Cause::Component(_)), "same runtime and settled source alone do not qualify an effect outside this reconstruction");
    assert_eq!(refusal.effect().subject().receipt(), &receipt);
    assert_eq!(
        medium.offered().len(),
        offers,
        "actual repair and refused admission never republish"
    );
}

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
    assert_eq!(
        proof.custody().effect().semantic_result(),
        semantic.semantic_result()
    );
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
    assert!(matches!(
        refused.reason(),
        worth_relational::facade::runtime::RelationalRuntimeCloseDenial::ReadCustodyRetained(_)
    ));
    let runtime = refused.into_runtime();
    proof.consume(|custody| {
        let (effect, before, before_lease, after, after_lease) = custody.into_retained_parts();
        assert_eq!(effect.subject().association(), Some(&association));
        assert_eq!(effect.semantic_result(), semantic.semantic_result());
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
    let digest: worth_foundational::CanonicalDigestId =
        serde_json::from_value(changed["descriptor"]["visibility_commitment"].clone()).unwrap();
    let mut bytes = *digest.bytes();
    bytes[0] ^= 1;
    changed["descriptor"]["visibility_commitment"] =
        serde_json::to_value(worth_foundational::CanonicalDigestId::new(bytes)).unwrap();
    let changed: Recorded = serde_json::from_value(changed).unwrap();
    let refusal = authority
        .admit_recorded_publication(execution, &runtime, effect, &predecessor, &changed)
        .unwrap_err();
    assert!(matches!(refusal.cause(), Cause::Component(_)));
    assert!(
        refusal.retains_acquired_custody(),
        "the genuine predecessor pin survives successor refusal"
    );
    assert_eq!(
        refusal.effect().semantic_result(),
        semantic.semantic_result()
    );
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

#[test]
fn validate_recorded_publication_predecessor_accepts_exact_and_rejects_mismatched_predecessor() {
    use worth_testkit_relational::initial_input_source::{
        capture_external, home_image_input, initialized, receiver,
    };
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    for use_checkpoint in [false, true] {
        let (source, medium) = if use_checkpoint {
            initialized(execution)
        } else {
            let m = TestDurableMedium::new();
            (durable_runtime(&m), m)
        };
        // Publish a first commit so `before` is a real Canonical predecessor.
        create_entity(execution, &source, "canonical-predecessor");
        let before = current(execution, &source);
        let (_, before_basis) = source
            .observe_branch(&source.main_branch_identity())
            .unwrap();
        let before_lease = source.retain_component_basis(&before_basis).unwrap();
        let association = Association::from_bytes(b"canonical-predecessor-validation").unwrap();
        let performed = perform(execution, &source, &association, "canonical-successor");
        let after_basis = performed.next_basis().clone();
        let after = source
            .owner_component_services()
            .basis_port()
            .export_recorded_retained_component(execution, &after_basis)
            .unwrap();
        let after_lease = source.retain_component_basis(&after_basis).unwrap();
        let result = source.settle_performed_publication(performed).unwrap();
        release_test_commit_snapshot(&source, &result);
        drop(after_basis);
        let (runtime, authority) = if use_checkpoint {
            let complete = capture_external(&source, &medium, execution);
            medium.file_checkpoint(complete.checkpoint().clone());
            let input = home_image_input(&medium, &complete, execution).unwrap();
            source.release_component_basis(before_lease).unwrap();
            source.release_component_basis(after_lease).unwrap();
            drop((before_basis, complete, source));
            let mut runtime = receiver(&medium);
            let plan = runtime
                .durability()
                .recovery_plan_from_source(
                    execution,
                    input,
                    RecoveryVerificationMode::NormalRecoveryVerification,
                )
                .unwrap();
            let (_, authority) = runtime
                .durability_recovery()
                .recover_with_authority(execution, plan)
                .unwrap();
            (runtime, authority)
        } else {
            source.release_component_basis(before_lease).unwrap();
            source.release_component_basis(after_lease).unwrap();
            drop((before_basis, source));
            recover(execution, &medium)
        };
        let owner = runtime.durability_authority();
        let subject = owner
            .discover_associated_effects(execution, &association)
            .unwrap()
            .pop()
            .unwrap();
        let effect = owner
            .readmit_associated_effect(execution, &subject)
            .unwrap();
        assert_eq!(
            effect.settlement(),
            if use_checkpoint {
                Settlement::CheckpointReadmitted
            } else {
                Settlement::PerformedSettled
            }
        );
        let Target::Canonical(before_target) = before.target() else {
            panic!("expected canonical predecessor");
        };
        let before_branch = before.descriptor().branch_id();
        let before_commit = before_target.receipt().commit_id;

        // Exact full and pruned-predecessor validation both succeed on Original and Checkpoint.
        authority
            .validate_recorded_publication_predecessor(execution, &runtime, &effect, &before, &after)
            .expect("exact canonical predecessor and successor must pass");
        authority
            .validate_recorded_publication_pruned_predecessor(
                execution,
                &runtime,
                &effect,
                before_branch,
                before_commit,
                &after,
            )
            .expect("exact pruned predecessor and successor must pass");

        // Predecessor mismatches fail in the origin-specific arm.
        let mut wrong_prev_truth = serde_json::to_value(&before).unwrap();
        let prev_truth = wrong_prev_truth["descriptor"]["truth_version"].as_u64().unwrap();
        wrong_prev_truth["descriptor"]["truth_version"] = serde_json::json!(prev_truth + 1);
        let wrong_prev_truth: Recorded = serde_json::from_value(wrong_prev_truth).unwrap();
        assert!(matches!(
            authority.validate_recorded_publication_predecessor(
                execution,
                &runtime,
                &effect,
                &wrong_prev_truth,
                &after,
            ),
            Err(Cause::SubjectMismatch | Cause::AdmissionImage(_))
        ));
        let mut wrong_prev_branch = serde_json::to_value(&before).unwrap();
        wrong_prev_branch["descriptor"]["branch_id"] = serde_json::json!("forged-branch");
        let wrong_prev_branch: Recorded = serde_json::from_value(wrong_prev_branch).unwrap();
        assert!(matches!(
            authority.validate_recorded_publication_predecessor(
                execution,
                &runtime,
                &effect,
                &wrong_prev_branch,
                &after,
            ),
            Err(Cause::SubjectMismatch | Cause::AdmissionImage(_))
        ));

        // Successor axis mismatches (truth_version, generation, posture) pass the shared
        // receipt guard and MUST fail in the origin-specific Original & Checkpoint arms!
        let mut wrong_truth = serde_json::to_value(&after).unwrap();
        let truth = wrong_truth["descriptor"]["truth_version"].as_u64().unwrap();
        wrong_truth["descriptor"]["truth_version"] = serde_json::json!(truth + 1);
        let wrong_truth: Recorded = serde_json::from_value(wrong_truth).unwrap();

        let mut wrong_gen = serde_json::to_value(&after).unwrap();
        let generation = wrong_gen["descriptor"]["reference"]["generation"].as_u64().unwrap();
        wrong_gen["descriptor"]["reference"]["generation"] = serde_json::json!(generation + 1);
        let wrong_gen: Recorded = serde_json::from_value(wrong_gen).unwrap();

        let mut wrong_posture = serde_json::to_value(&after).unwrap();
        wrong_posture["descriptor"]["posture"] = serde_json::json!("Archived");
        let wrong_posture: Recorded = serde_json::from_value(wrong_posture).unwrap();

        for bad_after in [&wrong_truth, &wrong_gen, &wrong_posture] {
            assert!(matches!(
                authority.validate_recorded_publication_predecessor(
                    execution,
                    &runtime,
                    &effect,
                    &before,
                    bad_after,
                ),
                Err(Cause::SubjectMismatch | Cause::AdmissionImage(_))
            ));
            assert!(matches!(
                authority.validate_recorded_publication_pruned_predecessor(
                    execution,
                    &runtime,
                    &effect,
                    before_branch,
                    before_commit,
                    bad_after,
                ),
                Err(Cause::SubjectMismatch | Cause::AdmissionImage(_))
            ));
        }

        // Descriptive validation acquires zero read custody pins.
        drop(effect);
        let runtime = runtime
            .prepare_close()
            .expect("no pins acquired by descriptive validation");
        drop(runtime);
    }
}

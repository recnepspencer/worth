//! Actual Native recovery emits concrete custody, never a fabricated live result.
use crate::facade::branch::RecordedRelationalComponentReference as Recorded;
use crate::facade::durability::{
    RecoveryVerificationMode, RelationalRecoveredPublicationAdmissionCause as Cause,
};
use crate::facade::mvcc::{
    RelationalCallerAssociation as Association, RelationalPublicationOutcome,
};
use crate::tests::support::*;
use worth_execution::ExecutionRequest;

pub(super) fn current(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> Recorded {
    let port = runtime.owner_component_services().basis_port();
    let (_, basis) = port
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    port.export_recorded_current_component(execution, &basis)
        .unwrap()
}

pub(super) fn perform(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    association: &Association,
    key: &str,
) -> crate::mvcc::PerformedRelationalCommit {
    let mut transaction = test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(batch_create(key), execution.allocation_policy())
        .unwrap();
    let proposal = transaction
        .validate(execution, runtime)
        .unwrap()
        .bind_caller_association(association.clone())
        .unwrap();
    let candidate = runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("actual source effect performs")
    };
    performed
}

fn source_journey(
    execution: &ExecutionRequest<'_, '_>,
) -> (
    RelationalRuntime,
    Recorded,
    Recorded,
    Association,
    crate::transactions::data::RelationalCommitSemanticResult,
) {
    let runtime = durable_runtime_with_test_schema();
    let predecessor = current(execution, &runtime);
    let association = Association::from_bytes(b"original-release-subject\0\xff").unwrap();
    let performed = perform(
        execution,
        &runtime,
        &association,
        "original-associated-effect",
    );
    let original_basis = performed.next_basis().clone();
    let result = runtime.settle_performed_publication(performed).unwrap();
    let semantic = result.semantic_result().clone();
    release_test_commit_snapshot(&runtime, &result);
    create_entity(execution, &runtime, "later-native-head");
    let successor = runtime
        .owner_component_services()
        .basis_port()
        .export_recorded_retained_component(execution, &original_basis)
        .unwrap();
    (runtime, predecessor, successor, association, semantic)
}

#[test]
fn recovered_publication_transfers_real_empty_predecessor_and_original_successor_custody() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let (source, predecessor, successor, association, semantic) = source_journey(execution);
    let offers = attached_test_medium(&source).offered().len();
    let mut runtime = durable_runtime_with_test_schema();
    let plan = medium_recovery_plan(
        execution,
        &source,
        &mut runtime,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )
    .unwrap();
    let (_, authority) = runtime
        .durability_recovery()
        .recover_with_authority(execution, plan)
        .unwrap();
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
    assert_eq!(
        proof
            .custody()
            .predecessor()
            .descriptor()
            .in_canonical_frame()
            .unwrap(),
        *predecessor.descriptor()
    );
    assert_eq!(
        proof
            .custody()
            .successor()
            .descriptor()
            .in_canonical_frame()
            .unwrap(),
        *successor.descriptor()
    );
    assert!(
        runtime
            .compare_current_exact(proof.custody().successor())
            .is_err(),
        "later head is not substituted for the original effect"
    );
    let refused = runtime.prepare_close().unwrap_err();
    assert_eq!(
        refused.reason(),
        crate::runtime::RelationalRuntimeCloseDenial::ReadCustodyRetained
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
        attached_test_medium(&runtime).offered().len(),
        offers,
        "readmission never republishes the Native effect"
    );
}

#[test]
fn recovered_publication_refusal_preserves_source_and_partial_pin_for_fresh_request() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let (source, predecessor, successor, association, semantic) = source_journey(execution);
    let mut runtime = durable_runtime_with_test_schema();
    let plan = medium_recovery_plan(
        execution,
        &source,
        &mut runtime,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )
    .unwrap();
    let (_, authority) = runtime
        .durability_recovery()
        .recover_with_authority(execution, plan)
        .unwrap();
    let owner = runtime.durability_authority();
    let subject = owner
        .discover_associated_effects(execution, &association)
        .unwrap()
        .pop()
        .unwrap();
    let effect = owner
        .readmit_associated_effect(execution, &subject)
        .unwrap();
    let foreign = durable_runtime_with_test_schema();
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
    let fresh_host = crate::test_execution_policy::serial_request();
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
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let (source, _, _, _, _) = source_journey(execution);
    let mut runtime = durable_runtime_with_test_schema();
    let plan = medium_recovery_plan(
        execution,
        &source,
        &mut runtime,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )
    .unwrap();
    let (_, authority) = runtime
        .durability_recovery()
        .recover_with_authority(execution, plan)
        .unwrap();
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
    let offers = attached_test_medium(&runtime).offered().len();
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
        attached_test_medium(&runtime).offered().len(),
        offers,
        "actual repair and refused admission never republish"
    );
}

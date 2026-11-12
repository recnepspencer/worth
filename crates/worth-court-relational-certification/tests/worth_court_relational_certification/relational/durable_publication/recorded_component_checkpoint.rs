use super::*;
use worth_relational::facade::{
    durability::RelationalDurableMemberLocator, publication::PatchStreamPosition,
    storage::RelationalStorageUnavailableCause,
};
fn selected_member(
    medium: &TestDurableMedium,
    position: PatchStreamPosition,
) -> RelationalDurableMemberLocator {
    use sha2::{Digest, Sha256};
    let bytes = medium
        .artifact_bytes(position)
        .expect("original offered canonical frame");
    RelationalDurableMemberLocator {
        sha256: Sha256::digest(&bytes).into(),
        length: bytes.len() as u64,
    }
}

#[test]
fn recorded_component_checkpoint_missing_selected_source_refuses_with_original_subject() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let original = original(execution, false);
    let (runtime, authority) = recover(execution, &original.medium);
    let worth_relational::facade::branch::RecordedRelationalComponentTarget::Canonical(before) =
        original.before.target()
    else {
        panic!("canonical predecessor");
    };
    let member = selected_member(&original.medium, before.position());
    original.medium.drop_artifact(before.position());
    original
        .medium
        .refuse_member_read_for_test(member, RelationalStorageUnavailableCause::MissingMember);
    let journal_reads = original.medium.canonical_journal_reads();
    let refusal = authority
        .admit_recorded_component(execution, &runtime, &original.before)
        .unwrap_err();
    assert!(matches!(
        refusal.cause(),
        RelationalRecordedComponentAdmissionCause::Source(_)
    ));
    assert_eq!(original.medium.last_member_fault_for_test(), Some(member));
    assert_eq!(original.medium.canonical_journal_reads(), journal_reads);
    assert!(!refusal.retains_acquired_custody());
    assert_ne!(original.before, original.after);
}

#[test]
fn recorded_component_checkpoint_derives_original_successor_without_caller_after_reference() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let source = durable_runtime(&medium);
    create_entity(execution, &source, "original-product");
    let before = current(execution, &source);
    let (_, before_basis) = source
        .observe_branch(&source.main_branch_identity())
        .unwrap();
    let before_lease = source.retain_component_basis(&before_basis).unwrap();
    let association =
        Association::from_bytes(b"crash-before-world-occurrence-no-after-ref").unwrap();
    let performed = perform(execution, &source, &association, "original-effect");
    let original_receipt = performed.canonical_commit().commit.clone();
    let original_after = performed.next_basis().clone();
    let after_lease = source.retain_component_basis(&original_after).unwrap();
    let result = source.settle_performed_publication(performed).unwrap();
    release_test_commit_snapshot(&source, &result);
    drop(original_after);
    create_entity(execution, &source, "foreign-later-head");
    medium.file_checkpoint(capture(execution, &source));
    source.release_component_basis(before_lease).unwrap();
    source.release_component_basis(after_lease).unwrap();
    drop(before_basis);
    let (runtime, authority) = recover(execution, &medium);
    let subject = runtime
        .durability_authority()
        .discover_associated_effects(execution, &association)
        .unwrap()
        .pop()
        .unwrap();
    let effect = runtime
        .durability_authority()
        .readmit_associated_effect(execution, &subject)
        .unwrap();
    let foreign_medium = TestDurableMedium::new();
    let foreign = durable_runtime(&foreign_medium);
    assert!(matches!(
        authority.recorded_publication_successor(execution, &foreign, &effect),
        Err(worth_relational::facade::durability::RelationalRecordedSuccessorRefusal::ForeignRecoveryRuntime)
    ));
    let after = authority
        .recorded_publication_successor(execution, &runtime, &effect)
        .unwrap();
    let worth_relational::facade::branch::RecordedRelationalComponentTarget::Canonical(target) =
        after.target()
    else {
        panic!("actual original successor");
    };
    assert_eq!(target.receipt(), &original_receipt);
    assert_eq!(target.position(), subject.position());
    assert_eq!(target.artifact_digest(), subject.artifact_digest());
    let proof = authority
        .admit_recorded_publication(execution, &runtime, effect, &before, &after)
        .unwrap();
    assert!(runtime
        .owner_component_services()
        .basis_port()
        .compare_current_exact(proof.custody().successor())
        .is_err());
}

#[test]
fn recorded_component_checkpoint_body_alone_cannot_invent_original_successor_image() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let original = original_with_retained_successor(execution, false, false);
    let (runtime, authority) = recover(execution, &original.medium);
    let subject = runtime
        .durability_authority()
        .discover_associated_effects(execution, &original.association)
        .unwrap()
        .pop()
        .unwrap();
    let effect = runtime
        .durability_authority()
        .readmit_associated_effect(execution, &subject)
        .unwrap();
    assert!(matches!(
        authority.recorded_publication_successor(execution, &runtime, &effect),
        Err(worth_relational::facade::durability::RelationalRecordedSuccessorRefusal::MissingRecoveredImage)
    ));
    assert_eq!(
        effect.subject().receipt(),
        subject.receipt(),
        "refusal keeps the same actual source description"
    );
}

#[test]
fn recorded_component_checkpoint_reexports_the_actual_cold_historical_subject() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let original = original(execution, false);
    let recorded = original.before.clone();
    let medium = original.medium.clone();
    let offers = medium.offered().len();
    drop(original);
    let (runtime, authority) = recover(execution, &medium);
    let basis = authority
        .admit_recorded_component(execution, &runtime, &recorded)
        .unwrap()
        .into_basis();
    let port = runtime.owner_component_services().basis_port();
    assert!(port.compare_current_exact(&basis).is_err());
    assert_eq!(
        port.export_recorded_retained_component(execution, &basis)
            .unwrap(),
        recorded
    );
    assert_eq!(
        medium.offered().len(),
        offers,
        "cold export never republishes"
    );
}

#[test]
fn recorded_component_checkpoint_reexport_refuses_missing_original_source() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let original = original(execution, false);
    let recorded = original.before.clone();
    let medium = original.medium.clone();
    drop(original);
    let (runtime, authority) = recover(execution, &medium);
    let basis = authority
        .admit_recorded_component(execution, &runtime, &recorded)
        .unwrap()
        .into_basis();
    let Target::Canonical(target) = recorded.target() else {
        panic!("the source-issued predecessor is canonical")
    };
    let member = selected_member(&medium, target.position());
    medium.drop_artifact(target.position());
    medium.refuse_member_read_for_test(member, RelationalStorageUnavailableCause::MissingMember);
    let journal_reads = medium.canonical_journal_reads();
    assert!(matches!(
        runtime
            .owner_component_services()
            .basis_port()
            .export_recorded_retained_component(execution, &basis),
        Err(RelationalRecordedComponentRefusal::Source(_))
    ));
    assert_eq!(medium.last_member_fault_for_test(), Some(member));
    assert_eq!(medium.canonical_journal_reads(), journal_reads);
    assert_recorded_basis(&runtime, &basis, &recorded);
}

#[test]
fn recorded_component_checkpoint_reexport_refuses_corrupt_original_source() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let original = original(execution, false);
    let recorded = original.before.clone();
    let medium = original.medium.clone();
    drop(original);
    let (runtime, authority) = recover(execution, &medium);
    let basis = authority
        .admit_recorded_component(execution, &runtime, &recorded)
        .unwrap()
        .into_basis();
    let Target::Canonical(target) = recorded.target() else {
        panic!("the source-issued predecessor is canonical")
    };
    let member = selected_member(&medium, target.position());
    let final_byte = medium.artifact_bytes(target.position()).unwrap().len() - 1;
    medium.flip_artifact_byte(target.position(), final_byte);
    medium.corrupt_member_read_for_test(member);
    let journal_reads = medium.canonical_journal_reads();
    assert!(matches!(
        runtime
            .owner_component_services()
            .basis_port()
            .export_recorded_retained_component(execution, &basis),
        Err(RelationalRecordedComponentRefusal::Source(_))
    ));
    assert_eq!(medium.last_member_fault_for_test(), Some(member));
    assert_eq!(medium.canonical_journal_reads(), journal_reads);
    assert_recorded_basis(&runtime, &basis, &recorded);
}

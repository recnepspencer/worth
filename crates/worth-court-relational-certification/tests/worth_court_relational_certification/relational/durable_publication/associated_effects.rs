use super::*;

#[test]
fn associated_effects_read_exact_source_and_report_actual_pending_and_settled_routes() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let association = Association::from_bytes(b"actual-original-owner-attempt").unwrap();
    let performed = perform(execution, &runtime, &association, "source-backed");
    let original_receipt = performed.canonical_commit().commit.clone();
    let artifact = medium.offered().pop().unwrap();
    let owner = runtime.durability_authority();
    let mut subjects = owner
        .discover_associated_effects(execution, &association)
        .unwrap();
    assert_eq!(subjects.len(), 1);
    let subject = subjects.pop().unwrap();
    let pending = owner
        .readmit_associated_effect(execution, &subject)
        .unwrap();
    assert_eq!(pending.settlement(), Settlement::PerformedPending);
    assert_eq!(pending.subject().receipt(), &original_receipt);
    assert_eq!(pending.subject().position(), performed.patch_position());
    assert_eq!(
        pending.subject().artifact_digest(),
        artifact.payload_digest().as_bytes()
    );
    assert_eq!(
        pending.semantic_result(),
        performed.canonical_commit().semantic_result()
    );
    let committed = runtime.settle_performed_publication(performed).unwrap();
    let settled = owner
        .readmit_associated_effect(execution, &subject)
        .unwrap();
    assert_eq!(settled.settlement(), Settlement::PerformedSettled);
    assert_eq!(settled.semantic_result(), committed.semantic_result());
    let foreign_medium = TestDurableMedium::new();
    let foreign = durable_runtime(&foreign_medium);
    assert!(matches!(
        foreign
            .durability_authority()
            .readmit_associated_effect(execution, &subject),
        Err(Refusal::ForeignRuntime)
    ));
    // An already-resident body must not hide damage in the actual selected source.
    medium.flip_artifact_byte(artifact.position(), artifact.as_bytes().len() - 1);
    assert!(matches!(
        owner.readmit_associated_effect(execution, &subject),
        Err(Refusal::Recovery(_))
    ));
    assert_eq!(
        subject.receipt(),
        &original_receipt,
        "failed readmission leaves original evidence owned"
    );
    release_test_commit_snapshot(&runtime, &committed);
}

#[test]
fn associated_effects_observers_do_not_keep_the_closed_source_owner_alive() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let association = Association::from_bytes(b"descriptive-observer-after-owner-close").unwrap();
    let committed = runtime
        .settle_performed_publication(perform(execution, &runtime, &association, "observer"))
        .unwrap();
    let receipt = committed.commit.clone();
    release_test_commit_snapshot(&runtime, &committed);
    drop(committed);
    let weak = std::sync::Arc::downgrade(&medium);
    drop(medium);
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
    runtime.prepare_close().unwrap().finish();
    assert!(
        weak.upgrade().is_none(),
        "description observers cannot retain the actual source/Serving owner"
    );
    assert_eq!(subject.receipt(), &receipt);
    assert_eq!(effect.semantic_result().receipt(), &receipt);
}

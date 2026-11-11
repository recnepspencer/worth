//! Associated originals are discovered and readmitted from a selected checkpoint.
use super::*;
use worth_relational::facade::{
    durability::RelationalCheckpointArtifactPolicy, runtime::RelationalRuntime,
};

fn recover_external(
    request: &ExecutionRequest<'_, '_>,
    source: &RelationalRuntime,
    medium: &std::sync::Arc<TestDurableMedium>,
) -> RelationalRuntime {
    let checkpoint = source
        .durability_authority()
        .native_checkpoint_with_policy(
            request,
            request.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    medium.file_checkpoint(checkpoint);
    super::branch_history_recovery::recover(request, medium, durable_runtime(medium)).1
}

#[test]
fn associated_memory_refuses_durable_effect_discovery() {
    let host=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let association = Association::from_bytes(b"memory-is-not-a-durable-effect").unwrap();
    let memory = memory_runtime();
    let committed = memory
        .settle_performed_publication(perform(request, &memory, &association, "memory"))
        .unwrap();
    assert!(matches!(
        memory
            .durability_authority()
            .discover_associated_effects(request, &association),
        Err(Refusal::AssociatedMemory { .. })
    ));
    release_test_commit_snapshot(&memory, &committed);
}

#[test]
fn checkpoint_associated_subject_readmits_its_original_semantic_result() {
    let host=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let source = durable_runtime(&medium);
    let association = Association::from_bytes(b"one-selected-cold-original").unwrap();
    let committed = source
        .settle_performed_publication(perform(request, &source, &association, "selected"))
        .unwrap();
    let semantic = committed.semantic_result();
    release_test_commit_snapshot(&source, &committed);
    create_entity(request, &source, "unrelated-cold");
    create_entity(request, &source, "checkpoint-head");
    let recovered = recover_external(request, &source, &medium);
    let owner = recovered.durability_authority();
    let subjects = owner
        .discover_associated_effects(request, &association)
        .unwrap();
    assert_eq!(subjects.len(), 1);
    let effect = owner
        .readmit_associated_effect(request, &subjects[0])
        .unwrap();
    assert_eq!(effect.settlement(), Settlement::CheckpointReadmitted);
    assert_eq!(effect.semantic_result(), semantic);
    let frame = Artifact::readmit(
        &medium.artifact_bytes(effect.subject().position()).unwrap(),
        request,
    )
    .unwrap();
    assert_eq!(frame.envelope().semantic_result(), semantic);
}

#[test]
fn checkpoint_metadata_enumerates_known_orphan_and_unrelated_originals() {
    let host=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let source = durable_runtime(&medium);
    let known = Association::from_bytes(b"world:known-original-open").unwrap();
    let orphan = Association::from_bytes(b"world:orphan-effect-without-open").unwrap();
    let other = Association::from_bytes(b"another-owner:unrelated").unwrap();
    for (key, association) in [
        ("known", &known),
        ("orphan", &orphan),
        ("unrelated-associated", &other),
    ] {
        let pending = perform(request, &source, association, key);
        assert!(
            source
                .durability_authority()
                .discover_all_associated_effects(request)
                .unwrap()
                .iter()
                .any(|subject| subject.association() == Some(association)),
            "actual pending route is discoverable before catalog settlement"
        );
        let committed = source.settle_performed_publication(pending).unwrap();
        release_test_commit_snapshot(&source, &committed);
    }
    create_entity(request, &source, "unassociated-current-head");
    let runtime = recover_external(request, &source, &medium);
    let subjects = runtime
        .durability_authority()
        .discover_all_associated_effects(request)
        .unwrap();
    assert_eq!(subjects.len(), 3);
    for (subject, expected) in subjects.iter().zip([&known, &orphan, &other]) {
        assert_eq!(subject.association(), Some(expected));
    }
    assert_eq!(
        runtime
            .durability_authority()
            .discover_associated_effects(request, &known)
            .unwrap()
            .len(),
        1
    );
    let memory = memory_runtime();
    let result = memory
        .settle_performed_publication(perform(request, &memory, &known, "memory-associated"))
        .unwrap();
    assert!(matches!(
        memory
            .durability_authority()
            .discover_all_associated_effects(request),
        Err(Refusal::AssociatedMemory { .. })
    ));
    release_test_commit_snapshot(&memory, &result);
}

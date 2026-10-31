//! Orphan discovery inspects actual required metadata, not known-open guesses.
use super::associated_effects::perform;
use crate::facade::durability::{
    RelationalAssociatedEffectRefusal, RelationalCheckpointArtifactPolicy,
};
use crate::facade::mvcc::RelationalCallerAssociation as Association;
use crate::tests::support::*;
use worth_execution::ExecutionRequest;

#[test]
fn native_custody_correction_enumerates_known_orphan_and_unrelated_association_metadata() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let source = durable_runtime_with_test_schema();
    let known = Association::from_bytes(b"world:known-original-open").unwrap();
    let orphan = Association::from_bytes(b"world:orphan-effect-without-open").unwrap();
    let other = Association::from_bytes(b"another-owner:unrelated").unwrap();
    for (key, association) in [
        ("known", &known),
        ("orphan", &orphan),
        ("unrelated-associated", &other),
    ] {
        let pending = perform(execution, &source, key, association);
        assert!(
            source
                .durability_authority()
                .discover_all_associated_effects(execution)
                .unwrap()
                .iter()
                .any(|subject| subject.association() == Some(association)),
            "actual pending route is discoverable before catalog settlement"
        );
        let result = source.settle_performed_publication(pending).unwrap();
        release_test_commit_snapshot(&source, &result);
    }
    create_entity(execution, &source, "unassociated-current-head");
    let checkpoint = source
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    attached_test_medium(&source).file_checkpoint(checkpoint);
    let (_, runtime) = recover_with(execution, &source, durable_runtime_with_test_schema);
    let artifacts = attached_test_medium(&runtime).offered();
    for artifact in artifacts.iter().take(3) {
        attached_test_medium(&runtime).drop_artifact(artifact.position());
    }
    let subjects = runtime
        .durability_authority()
        .discover_all_associated_effects(execution)
        .unwrap();
    assert_eq!(subjects.len(), 3);
    for (subject, expected) in subjects.iter().zip([&known, &orphan, &other]) {
        assert_eq!(subject.association(), Some(expected));
        assert!(runtime
            .history
            .commit_artifact(subject.receipt().commit_id)
            .unwrap()
            .envelope_residency()
            .positioned_if_resident()
            .is_none());
    }
    assert_eq!(
        runtime
            .durability_authority()
            .discover_associated_effects(execution, &known)
            .unwrap()
            .len(),
        1
    );
    let memory = runtime_with_test_schema();
    let result = memory
        .settle_performed_publication(perform(execution, &memory, "memory-associated", &known))
        .unwrap();
    assert!(matches!(
        memory
            .durability_authority()
            .discover_all_associated_effects(execution),
        Err(RelationalAssociatedEffectRefusal::AssociatedMemory { .. })
    ));
    release_test_commit_snapshot(&memory, &result);
}

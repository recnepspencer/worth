//! The bound canonical port exists before publication, with genuine journal bytes.
use worth_execution::ExecutionRequest;
use worth_foundational::facade::FoundationalBoundaryAvailability as Availability;
use worth_relational::facade::{durability::*, transactions::RecordRef};
use worth_testkit_relational::{
    aspect_storage::publications::{changed_entities, release_test_commit_snapshot},
    durable_medium::{ScriptedPublication, TestDurableMedium},
    durable_publication::{create_entity_outcome, durable_runtime, memory_runtime},
};

#[test]
fn cold_source_is_bound_before_a_durable_runtime_publishes_its_first_commit() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    assert_eq!(
        runtime
            .durability()
            .external_canonical_source_availability(),
        Availability::Present
    );
    let committed = create_entity_outcome(request, &runtime, "attached-source-seed");
    let entity = changed_entities(&committed)[0];
    release_test_commit_snapshot(&runtime, &committed);
    let position = runtime.history().latest_patch_stream_position().unwrap();
    let source = medium.attachment().cold_source();
    let bytes = source.read_commit(position, 64 << 20).unwrap();
    let artifact = RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes).unwrap();
    assert_eq!(artifact.position(), position);
    let envelope =
        RelationalCanonicalCommitArtifact::readmit(artifact.as_bytes(), request).unwrap();
    let envelope = envelope.envelope();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let receipt = runtime
        .history()
        .branch_head_for_observation(&basis.observation())
        .unwrap()
        .unwrap();
    assert_eq!(envelope.commit.commit_id, receipt.commit_id);
    assert!(envelope
        .patch
        .authoritative_record_patches
        .iter()
        .any(|patch| patch.target == RecordRef::Entity(entity)));
    assert_eq!(
        runtime.history().latest_patch_stream_position(),
        Some(position)
    );
    assert_eq!(
        memory_runtime()
            .durability()
            .external_canonical_source_availability(),
        Availability::Unavailable
    );
}

#[test]
fn scripted_medium_answers_distinguish_physical_no_effect_and_unknown_effect() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let producer_medium = TestDurableMedium::new();
    let runtime = durable_runtime(&producer_medium);
    let committed = create_entity_outcome(request, &runtime, "scripted-original");
    release_test_commit_snapshot(&runtime, &committed);
    let artifacts = producer_medium.committed_artifacts().unwrap();
    assert_eq!(artifacts.len(), 1);
    let original = &artifacts[0];
    for answer in [
        ScriptedPublication::ProvenNoEffect,
        ScriptedPublication::IndeterminateWithoutEffect,
        ScriptedPublication::IndeterminateAfterEffect,
    ] {
        let medium = TestDurableMedium::new();
        medium.script_next_publication(answer);
        let attachment = medium.attachment();
        let fate = attachment.publish(original);
        match answer {
            ScriptedPublication::ProvenNoEffect => assert!(matches!(
                fate,
                DurablePublicationFate::ProvenNoEffect { .. }
            )),
            ScriptedPublication::IndeterminateWithoutEffect
            | ScriptedPublication::IndeterminateAfterEffect => {
                assert!(matches!(fate, DurablePublicationFate::Indeterminate { .. }))
            }
        }
        assert_eq!(medium.offered().len(), 1);
        assert_eq!(
            medium.committed_artifacts().unwrap().len(),
            usize::from(answer == ScriptedPublication::IndeterminateAfterEffect)
        );
        assert!(matches!(
            attachment.publish(original),
            DurablePublicationFate::Committed
        ));
        assert_eq!(medium.committed_artifacts().unwrap(), artifacts);
    }
}

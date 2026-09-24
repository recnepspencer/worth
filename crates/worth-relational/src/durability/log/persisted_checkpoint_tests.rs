use super::*;
use crate::history::data::BranchId;
use crate::identity::data::VersionId;
use crate::indexes::data::{
    DerivedIndexApplicability, DerivedIndexArtifacts, DerivedIndexEntries, DerivedIndexGeneration,
    DerivedIndexGenerationId, DerivedIndexId, DerivedIndexPublicationStatus,
};
use crate::tests::support::*;

#[test]
fn borrowed_checkpoint_wire_matches_owned_wire_and_omits_envelope_index_cache() {
    let runtime = persisted_runtime_with_test_schema();
    let committed = create_entity_outcome(&runtime, "borrowed-checkpoint-wire");
    release_test_commit_snapshot(&runtime, &committed);
    let mut checkpoint = runtime.durability_authority().checkpoint().unwrap();
    let envelope = checkpoint.envelopes[0].envelope_mut_for_test();
    envelope.derived_index_artifacts = DerivedIndexArtifacts::new(vec![DerivedIndexGeneration {
        generation_id: DerivedIndexGenerationId(9_001),
        index_id: DerivedIndexId(7_001),
        source_commit_id: committed.commit.commit_id,
        source_branch_id: BranchId("main".into()),
        applicability: DerivedIndexApplicability {
            branch_id: BranchId("main".into()),
            version_id: VersionId(committed.version_id.0),
            schema_version: envelope.schema_version,
        },
        status: DerivedIndexPublicationStatus::Published,
        entries: DerivedIndexEntries::EntityField(Default::default()),
    }]);

    let old_wire = rmp_serde::to_vec_named(&PersistedDurableCheckpointFile::from_checkpoint(
        checkpoint.clone(),
    ))
    .unwrap();
    let borrowed_wire =
        rmp_serde::to_vec_named(&PersistedDurableCheckpointFileRef::new(&checkpoint)).unwrap();
    assert_eq!(borrowed_wire, old_wire);
    assert!(
        !checkpoint.envelopes[0]
            .envelope()
            .derived_index_artifacts
            .is_empty(),
        "borrowed encoding does not mutate its input"
    );
    let restored: PersistedDurableCheckpointFile = rmp_serde::from_slice(&borrowed_wire).unwrap();
    let restored = restored.readmit().unwrap();
    assert!(restored.checkpoint.envelopes[0]
        .envelope()
        .derived_index_artifacts
        .is_empty());
}

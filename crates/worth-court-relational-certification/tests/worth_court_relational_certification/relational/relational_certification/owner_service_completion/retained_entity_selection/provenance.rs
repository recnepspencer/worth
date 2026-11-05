//! Intact digests cannot bless malformed selection meaning.
use super::occurrences::{fixture, perform, prepare};
use super::*;
use sha2::{Digest, Sha256};
use worth_relational::facade::{
    durability::RelationalCanonicalCommitArtifactRefusal as Refusal,
    durability::RelationalCanonicalPositionAdmissionDenial,
    history::{
        RelationalCommitArtifactDenial, RelationalRetainedSelectionArtifactDenial as Denial,
    },
};

#[test]
fn rehashed_selection_artifacts_refuse_wrong_source_identity_predecessor_and_mixed_profiles() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = fixture().build_runtime();
    let old = create(execution, &runtime, "model", "3");
    let model = changed_entities(&old)[0];
    let source = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let current = update(execution, &runtime, [(model, "5")]);
    let selected = perform(
        execution,
        &runtime,
        prepare(execution, &runtime, &source, model),
    );
    let artifact = runtime
        .durability()
        .canonical_commit_artifacts(&execution)
        .map(Result::unwrap)
        .find(|artifact| artifact.commit_id() == selected.commit.commit_id)
        .unwrap();
    RelationalCanonicalCommitArtifact::readmit(artifact.as_bytes(), execution).unwrap();
    let original = artifact.as_bytes();
    let head = value_after(original, 66, b"source_head");
    let entity = value_after(original, head, b"entity");
    let before = value_after(original, 66, b"branch_cell_checkpoint");
    let mut cases = Vec::new();
    let mut foreign_frame = original.to_vec();
    let runtime_offset = value_after(original, head, b"runtime_instance_id");
    assert_eq!(
        &foreign_frame[runtime_offset..runtime_offset + 5],
        &[0xce, 0x3b, 0x9a, 0xca, 0x00]
    );
    foreign_frame[runtime_offset + 4] = 1;
    cases.push((
        "foreign source frame",
        foreign_frame,
        Denial::SourceDescriptor,
    ));
    cases.push((
        "empty committed source",
        fixint(original, value_after(original, head, b"commit_id"), 0),
        Denial::SourceDescriptor,
    ));
    cases.push((
        "generation zero",
        fixint(original, value_after(original, entity, b"generation"), 0),
        Denial::EntityIdentity,
    ));
    cases.push((
        "relation kind used as entity kind",
        fixint(original, value_after(original, entity, b"kind"), 2),
        Denial::SchemaAgreement,
    ));
    let branch = value_after(original, before, b"branch_id");
    assert_eq!(&original[branch..branch + 5], b"\xa4main");
    let mut wrong_predecessor = original.to_vec();
    wrong_predecessor[branch + 1..branch + 5].copy_from_slice(b"evil");
    cases.push((
        "wrong target branch",
        wrong_predecessor,
        Denial::TargetPredecessor,
    ));
    let parents = value_after(original, 66, b"merge_parent_branches");
    assert_eq!(original[parents], 0x90);
    let mut merge = original.to_vec();
    merge.splice(parents..parents + 1, [0x91, 0xa4, b'm', b'a', b'i', b'n']);
    cases.push(("mixed merge profile", merge, Denial::UnsupportedProfile));
    let structural = value_after(original, 66, b"structural_change");
    assert_eq!(&original[structural..structural + 8], b"\xa7Updated");
    let mut created = original.to_vec();
    created[structural + 1..structural + 8].copy_from_slice(b"Created");
    cases.push((
        "selected entity recreated",
        created,
        Denial::SelectedRecordMeaning,
    ));
    for (name, mut bytes, expected) in cases {
        // Mutate a real artifact, preserving its original frame grammar and
        // producing a valid digest, so this reaches semantic readmission.
        let length = u64::try_from(bytes.len() - 66).unwrap();
        bytes[58..66].copy_from_slice(&length.to_le_bytes());
        let digest = Sha256::digest(&bytes[66..]);
        bytes[26..58].copy_from_slice(&digest);
        RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone().into_boxed_slice())
            .unwrap();
        let refusal = RelationalCanonicalCommitArtifact::readmit(&bytes, execution).unwrap_err();
        let Refusal::PositionAdmission(RelationalCanonicalPositionAdmissionDenial::CommitArtifact(
            RelationalCommitArtifactDenial::RetainedEntitySelection(actual),
        )) = refusal
        else {
            panic!("{name} failed at the wrong boundary: {refusal:?}")
        };
        assert_eq!(actual, expected, "{name}");
    }
    assert_eq!(
        runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .unwrap()
            .observation()
            .commit_id()
            .unwrap(),
        selected.commit.commit_id
    );
    for commit in [&old, &current, &selected] {
        release_test_commit_snapshot(&runtime, commit);
    }
}

fn value_after(bytes: &[u8], start: usize, key: &[u8]) -> usize {
    start
        + bytes[start..]
            .windows(key.len())
            .position(|candidate| candidate == key)
            .expect("the genuine fixture must contain this named field")
        + key.len()
}

fn fixint(bytes: &[u8], offset: usize, replacement: u8) -> Vec<u8> {
    assert!(
        bytes[offset] < 128 && replacement < 128,
        "fixture uses a positive MessagePack fixint"
    );
    let mut mutated = bytes.to_vec();
    mutated[offset] = replacement;
    mutated
}

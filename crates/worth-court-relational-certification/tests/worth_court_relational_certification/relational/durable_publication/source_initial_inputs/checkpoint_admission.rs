//! Real original capture, independently observed through the declared wire.
use super::*;
use sha2::{Digest, Sha256};
use worth_execution::ExecutionRequestArrayBuilder;
use worth_foundational::facade::{CheckpointAdmissionCut, CheckpointAdmissionScope, HomeIdentity};
use RelationalCheckpointAdmissionRefusal as Refusal;
mod capture_cost;
pub(crate) mod cold_materialization;
mod contract_work;
mod indexes;
mod operational;
mod references;
mod region_memory;
mod required_outputs;
mod retained_indexes;
mod root_projection;
mod selection_refusals;

fn cut(capture: &CompleteRelationalNativeCheckpoint) -> CheckpointAdmissionCut {
    CheckpointAdmissionCut::new(
        CheckpointAdmissionScope::Home {
            home: HomeIdentity::from_observed_bytes([1; 16]).unwrap(),
            native_origin: capture.source_artifact().initial_coordinate().1,
            complete_source_digest: Sha256::digest(capture.source_artifact().as_bytes()).into(),
            world_through: [0; 3],
            world_image_digest: [2; 32],
        },
        capture.source_artifact().covered().0,
        capture.source_artifact().covered_transitions().0,
        Sha256::digest(capture.checkpoint().bytes()).into(),
    )
}

fn parse(
    bytes: &[u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<RelationalCheckpointAdmissionArtifact, Refusal> {
    let mut owner = ExecutionRequestArrayBuilder::allocate(*request, bytes.len()).unwrap();
    owner.extend_from_slice(bytes).unwrap();
    RelationalCheckpointAdmissionArtifact::from_untrusted_region(
        owner.seal_immutable_bytes().unwrap(),
        0..bytes.len(),
        request,
    )
}

fn captured(request: &ExecutionRequest<'_, '_>) -> CompleteRelationalNativeCheckpoint {
    let (runtime, _) = initialized(request);
    let commit = create_entity_outcome(request, &runtime, "projected-original");
    release_test_commit_snapshot(&runtime, &commit);
    capture(&runtime, request)
}

#[test]
fn original_checkpoint_capture_seals_each_projected_role_independently() {
    let policy = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&policy);
    let captured = captured(request);
    let expected = cut(&captured);
    let (image, report) = request
        .run(worth_execution::ExecutionWorkCeiling::new(None), |_| {
            captured
                .admission_image()
                .seal_home_cut(&expected, request)
                .unwrap()
        })
        .unwrap();
    assert_eq!(image.cut(), &expected);
    let raw = image.as_bytes();
    assert_eq!(&raw[..8], b"WRTHCAI1");
    assert_eq!(u32::from_le_bytes(raw[8..12].try_into().unwrap()), 1);
    assert_eq!(u64::from_le_bytes(raw[16..24].try_into().unwrap()), 1);
    let cut_end = 64 + u32::from_le_bytes(raw[12..16].try_into().unwrap()) as usize;
    let mut offset = cut_end;
    let mut projected = Vec::new();
    let meaning_prefix = b"worth.relational.checkpoint-admission.meaning.v1\0".len()
        + 4
        + 8
        + (cut_end - 64)
        + 8
        + 8
        + 1
        + 8;
    // The root directory is independently authenticated as role nine.
    let root_length = u64::from_le_bytes(raw[offset + 1..offset + 9].try_into().unwrap()) as usize;
    assert_eq!(raw[offset], 9);
    projected.push((0, CheckpointAdmissionMeaning::RootDirectory, offset + 41));
    offset += 41 + root_length;
    let mut row_hash_work = 0;
    assert_eq!(
        u64::from_le_bytes(raw[offset..offset + 8].try_into().unwrap()),
        1
    );
    offset += 16;
    for (tag, meaning) in [
        CheckpointAdmissionMeaning::Semantic,
        CheckpointAdmissionMeaning::Axes,
        CheckpointAdmissionMeaning::Association,
        CheckpointAdmissionMeaning::Coordinates,
        CheckpointAdmissionMeaning::Schema,
        CheckpointAdmissionMeaning::Visibility,
        CheckpointAdmissionMeaning::Creations,
        CheckpointAdmissionMeaning::Outputs,
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(raw[offset], tag as u8 + 1);
        let length = u64::from_le_bytes(raw[offset + 1..offset + 9].try_into().unwrap()) as usize;
        assert!(length > 0);
        projected.push((1, meaning, offset + 41));
        row_hash_work += meaning_prefix + length;
        offset += 41 + length;
    }
    assert_eq!(offset, raw.len());
    let directory_work = b"worth.relational.checkpoint-admission.directory.v1\0".len()
        + 32
        + (cut_end - 64)
        + 41
        + 16
        + 8 * 41;
    // Count the actual single-emission work: two serializer traversals of
    // the cut, one directory hash, each meaning hash, every output copy,
    // the nine-entry size scan, and replacement of the directory digest.
    let minimum_work = 2 * (cut_end - 64)
        + 9
        + raw.len()
        + directory_work
        + meaning_prefix
        + root_length
        + row_hash_work
        + 32;
    assert!(report.charged_work() >= minimum_work as u64,
        "both cut traversals, the directory scan and every output copy must be charged in addition to hashing: {} < {minimum_work}", report.charged_work());
    assert_eq!(parse(raw, request).unwrap().as_bytes(), raw);
    for (position, meaning, payload) in projected {
        let mut changed = raw.to_vec();
        changed[payload] ^= 1;
        assert_eq!(
            parse(&changed, request).unwrap_err(),
            Refusal::AuthenticationMismatch(CheckpointAdmissionAuthenticationMismatch::Meaning {
                position,
                meaning
            },)
        );
    }
    let mut changed = raw.to_vec();
    changed[32] ^= 1;
    assert_eq!(
        parse(&changed, request).unwrap_err(),
        Refusal::AuthenticationMismatch(CheckpointAdmissionAuthenticationMismatch::Directory,)
    );
}

#[test]
fn checkpoint_image_sealing_refuses_a_different_original_source_or_cut() {
    let policy = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&policy);
    let captured = captured(request);
    let original = cut(&captured);
    let scope = *original.scope();
    let CheckpointAdmissionScope::Home {
        home,
        native_origin,
        complete_source_digest,
        world_through,
        world_image_digest,
    } = scope
    else {
        panic!("the fixture has a Home cut");
    };
    let changed_source = CheckpointAdmissionScope::Home {
        home,
        native_origin,
        complete_source_digest: [8; 32],
        world_through,
        world_image_digest,
    };
    let changed_origin = CheckpointAdmissionScope::Home {
        home,
        native_origin: [9; 32],
        complete_source_digest,
        world_through,
        world_image_digest,
    };
    for (scope, covered, digest, mismatch) in [
        (
            changed_origin,
            original.canonical_through(),
            *original.checkpoint_digest(),
            CheckpointAdmissionBindingMismatch::NativeOrigin,
        ),
        (
            changed_source,
            original.canonical_through(),
            *original.checkpoint_digest(),
            CheckpointAdmissionBindingMismatch::Source,
        ),
        (
            scope,
            original.canonical_through() + 1,
            *original.checkpoint_digest(),
            CheckpointAdmissionBindingMismatch::Cut,
        ),
        (
            scope,
            original.canonical_through(),
            [3; 32],
            CheckpointAdmissionBindingMismatch::Checkpoint,
        ),
    ] {
        let changed =
            CheckpointAdmissionCut::new(scope, covered, original.transitions_through(), digest);
        assert_eq!(
            captured
                .admission_image()
                .seal_home_cut(&changed, request)
                .unwrap_err(),
            Refusal::BindingMismatch(mismatch)
        );
    }
}

#[test]
fn checkpoint_image_framing_refuses_unknown_versions_and_stopped_requests() {
    use worth_execution::{ExecutionWorkCeiling, MapKernelStop, WorkCeilingDenial};
    let policy = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&policy);
    let captured = captured(request);
    let cut = cut(&captured);
    let image = captured
        .admission_image()
        .seal_home_cut(&cut, request)
        .unwrap();
    let mut changed = image.as_bytes().to_vec();
    changed[8..12].copy_from_slice(&2_u32.to_le_bytes());
    assert!(matches!(
        parse(&changed, request),
        Err(Refusal::UnsupportedVersion(_))
    ));
    changed[8..12].copy_from_slice(&0_u32.to_le_bytes());
    assert_eq!(
        parse(&changed, request).unwrap_err(),
        Refusal::InvalidVersion
    );
    let (stopped, _) = request
        .run(ExecutionWorkCeiling::new(Some(0)), |_| {
            request.run_native_with_checkpoint(|context| {
                assert_eq!(context.checkpoint(1), Err(MapKernelStop::WorkCeiling));
                assert_eq!(
                    captured
                        .admission_image()
                        .seal_home_cut(&cut, request)
                        .unwrap_err(),
                    Refusal::Request(WorkCeilingDenial::Stopped(MapKernelStop::WorkCeiling).into())
                );
            })
        })
        .unwrap();
    assert!(stopped.is_err());
}

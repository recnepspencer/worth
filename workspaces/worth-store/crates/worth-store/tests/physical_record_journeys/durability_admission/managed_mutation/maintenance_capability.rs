use std::fs;

use worth_proof::TransitionOutcome;
use worth_signal::facade::TemporalDuration;
use worth_store::physical_runtime::{
    PhysicalCheckpointCaptureFailureKind, PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey,
    PhysicalCheckpointOutcome, PhysicalCheckpointProvenNoEffectCause, PhysicalCheckpointRequest,
};
use worth_store_physical_format::{
    DurablePhysicalRootManifest, PhysicalCheckpointSource, CHECKPOINT_STREAM_HEADER_RECORD_BYTES,
};

use super::selected_segment_rewrite::prepare_rewrite;
use super::{completed, configuration, prepare, serving_from_initialization};

#[test]
fn a_rewrite_publishes_maintenance_protocol_metadata() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    completed(prepare(&serving, placement, [61; 32], b"maintenance-source").execute());
    completed(prepare_rewrite(&serving, placement, [62; 32]).execute());
    let published = fs::read(newest_root(&root)).unwrap();
    let (manifest, _) = DurablePhysicalRootManifest::decode(&published, u16::MAX).unwrap();
    assert!(manifest.requires_maintenance_protocol());
    assert!(DurablePhysicalRootManifest::decode_c9_legacy(&published, u16::MAX).is_err());

    let handle = match serving.checkpoints().start(checkpoint_request()).into_raw() {
        TransitionOutcome::Success(handle) => handle,
        _ => panic!("checkpoint admission did not produce a handle"),
    };
    match handle.wait() {
        PhysicalCheckpointOutcome::Completed(_) => {}
        _ => panic!("checkpoint after a rewrite must complete"),
    }
    let checkpoint = fs::read(root.join("families/checkpoint.current")).unwrap();
    let header = &checkpoint[..CHECKPOINT_STREAM_HEADER_RECORD_BYTES];
    assert!(PhysicalCheckpointSource::decode_c9_legacy_stream_header_record(header).is_err());
    assert!(PhysicalCheckpointSource::decode_stream_header_record(header)
        .unwrap()
        .requires_maintenance_protocol());

    drop(serving);
    let reopened = crate::serving_from_open(&root);
    assert!(reopened.records().is_ok());
}

#[test]
fn a_terminal_checkpoint_releases_its_background_head() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    completed(prepare(&serving, placement, [64; 32], b"checkpoint-head").execute());
    serving.certification_fail_next_checkpoint_admission();
    let handle = match serving.checkpoints().start(checkpoint_request()).into_raw() {
        TransitionOutcome::Success(handle) => handle,
        _ => panic!("checkpoint admission did not produce a handle"),
    };
    match handle.wait() {
        PhysicalCheckpointOutcome::ProvenNoEffect(fate) => assert_eq!(
            fate.cause(),
            PhysicalCheckpointProvenNoEffectCause::DeniedBeforeCandidate(
                PhysicalCheckpointCaptureFailureKind::SchedulerCapacityUnavailable
            )
        ),
        other => panic!("the armed admission failure must end the checkpoint: {other:?}"),
    }
    assert!(
        !serving.certification_foreground_is_blocked_by_background(),
        "a finished checkpoint must release the background head"
    );
}

#[test]
fn a_finished_checkpoint_releases_a_deferred_reclamation_head() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    completed(prepare(&serving, placement, [65; 32], b"reclamation-head").execute());
    serving.certification_note_reclamation_background_head();
    assert!(
        serving.certification_foreground_is_blocked_by_background(),
        "the reclamation head must block foreground while it is retained"
    );
    let handle = match serving.checkpoints().start(checkpoint_request()).into_raw() {
        TransitionOutcome::Success(handle) => handle,
        _ => panic!("checkpoint admission did not produce a handle"),
    };
    let _terminal = handle.wait();
    assert!(
        !serving.certification_foreground_is_blocked_by_background(),
        "a finished checkpoint must release a deferred reclamation head"
    );
}

fn newest_root(root: &std::path::Path) -> std::path::PathBuf {
    fs::read_dir(root.join("families/records/roots"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("manifest"))
        .max()
        .unwrap()
}

fn checkpoint_request() -> PhysicalCheckpointRequest {
    PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([63; 32]),
        PhysicalCheckpointDeadline::at(
            TemporalDuration::temporal_duration(1_000).expect("deadline is positive"),
        ),
    )
}

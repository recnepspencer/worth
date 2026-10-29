#[allow(unused_imports)] use crate::physical_record_journeys::durability_admission::managed_mutation::maintenance_capability::*;
use {std::fs};

use {worth_proof::TransitionOutcome};
use {worth_signal::facade::TemporalDuration};
use {worth_store::physical_runtime::PhysicalCheckpointCaptureFailureKind, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointProvenNoEffectCause, worth_store::physical_runtime::PhysicalCheckpointRequest};
use {worth_store_physical_format::DurablePhysicalRootManifest, worth_store_physical_format::PhysicalCheckpointSource, worth_store_physical_format::CHECKPOINT_STREAM_HEADER_RECORD_BYTES};

use {worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::selected_segment_rewrite::prepare_rewrite};
use {worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::completed, worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};

#[test]
fn a_rewrite_publishes_maintenance_protocol_metadata() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    completed(prepare(&serving, placement, [61; 32], b"maintenance-source").execute());
    completed(prepare_rewrite(&serving, placement, [62; 32]).execute());
    let published = std::fs::read(newest_root(&root)).unwrap();
    let (manifest, _) = worth_store_physical_format::DurablePhysicalRootManifest::decode(&published, u16::MAX).unwrap();
    assert!(manifest.requires_maintenance_protocol());

    // An ordinary append after a maintenance rewrite must not downgrade the
    // selected root's protocol bit while carrying its existing routes forward.
    completed(prepare(&serving, placement, [63; 32], b"post-rewrite-append").execute());
    let successor = std::fs::read(newest_root(&root)).unwrap();
    let (successor, _) = worth_store_physical_format::DurablePhysicalRootManifest::decode(&successor, u16::MAX).unwrap();
    assert!(successor.requires_maintenance_protocol());

    let handle = match serving.checkpoints().start(checkpoint_request()).into_raw() {
        worth_proof::TransitionOutcome::Success(handle) => handle,
        _ => panic!("checkpoint admission did not produce a handle"),
    };
    match handle.wait() {
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_) => {}
        _ => panic!("checkpoint after a rewrite must complete"),
    }
    let checkpoint = std::fs::read(root.join("families/checkpoint.current")).unwrap();
    let header = &checkpoint[..CHECKPOINT_STREAM_HEADER_RECORD_BYTES];
    assert!(
        worth_store_physical_format::PhysicalCheckpointSource::decode_stream_header_record(header)
            .unwrap()
            .requires_maintenance_protocol()
    );

    drop(serving);
    let reopened = worth_store_testkit_store::physical_record_journeys::serving_from_open(&root);
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
        worth_proof::TransitionOutcome::Success(handle) => handle,
        _ => panic!("checkpoint admission did not produce a handle"),
    };
    match handle.wait() {
        worth_store::physical_runtime::PhysicalCheckpointOutcome::ProvenNoEffect(fate) => assert_eq!(
            fate.cause(),
            worth_store::physical_runtime::PhysicalCheckpointProvenNoEffectCause::DeniedBeforeCandidate(
                worth_store::physical_runtime::PhysicalCheckpointCaptureFailureKind::SchedulerCapacityUnavailable
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
        worth_proof::TransitionOutcome::Success(handle) => handle,
        _ => panic!("checkpoint admission did not produce a handle"),
    };
    let _terminal = handle.wait();
    assert!(
        !serving.certification_foreground_is_blocked_by_background(),
        "a finished checkpoint must release a deferred reclamation head"
    );
}

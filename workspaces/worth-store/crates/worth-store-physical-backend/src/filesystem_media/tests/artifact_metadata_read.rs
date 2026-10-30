use super::super::*;
use crate::{
    BackendQueueExecutionAdaptation, BackendQueueExecutionBudgetBinding,
    BackendQueueExecutionPlanBinding, BackendQueueExecutionReplayBinding,
};
use worth_proof::TransitionOutcome;
use worth_store_aspect_native::StorePhysicalBoundaryWitness;
use worth_store_contracts::{StorePhysicalAuthorityWitness, ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE};
use worth_store_physical_format::RecordArtifactFile;
use worth_store_security::{
    StoreAuthenticityRequirement, StoreCustodyPosture, StoreKeyScope, StoreKeyVersionPosture,
    StoreSecurityScopeIdentity, StoreTenantScope,
};

fn qualified(root: &std::path::Path, schedule: MediaFaultSchedule) -> QualifiedFilesystemMedia {
    let request = FilesystemQualificationRequest::certification(
        root,
        FilesystemAccessPosture::CoordinatedServiceAccount,
    )
    .with_fault_schedule(schedule);
    match FilesystemMediaOwner::qualify(request).into_raw() {
        TransitionOutcome::Success(media) => media,
        _ => panic!("qualification failed"),
    }
}

fn binding(media: &QualifiedFilesystemMedia) -> BackendQueueExecutionPlanBinding {
    let authority = StorePhysicalAuthorityWitness::for_aspect_native_boundary(
        ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
    )
    .unwrap();
    let scope = StoreSecurityScopeIdentity::from_physical_security_scope(
        StorePhysicalBoundaryWitness::from_physical_authority(authority).unwrap(),
        StoreKeyScope::StoreManagedRoot,
        StoreKeyVersionPosture::Current,
        StoreTenantScope::StoreInternal,
        StoreAuthenticityRequirement::not_required(),
        StoreCustodyPosture::InternalStoreCustody,
    );
    let replay = BackendQueueExecutionReplayBinding::from_store_queue_replay(
        1,
        1,
        1,
        scope,
        scope.tenant_scope(),
        scope.key_scope(),
        scope.authenticity_requirement(),
        1,
        0,
        0,
        BackendQueueExecutionBudgetBinding::new(1, 1, 0, 0, 1, 1, 0, 1, 1, 0),
    );
    BackendQueueExecutionPlanBinding::from_store_replay_binding(
        replay,
        None,
        media.execution_capability().profile(),
        media.execution_capability().evidence_class(),
        0,
    )
}

fn seed_metadata_artifact(media: &QualifiedFilesystemMedia) -> ArtifactTreeFile {
    let tree = media.artifact_tree();
    let records = ArtifactTreeDirectory::families().child("records").unwrap();
    tree.create_directory(&records).unwrap();
    let artifact = RecordArtifactFile::BootstrapCatalog;
    let file = records.file(&artifact.file_name()).unwrap();
    tree.write_new(&file, b"catalog").unwrap();
    file
}

fn setup_metadata_read_ordinal(root: &std::path::Path) -> u64 {
    let media = qualified(root, MediaFaultSchedule::default());
    seed_metadata_artifact(&media);
    let ordinal = media
        .counters()
        .attempts_for(MediaOperationRole::ReadMetadata);
    media.close();
    ordinal
}

#[expect(
    clippy::disallowed_methods,
    reason = "category: coordinated storage-owner unit-test harness; purpose: scheduled metadata receipt and counter share one identified operation; source: worth-store-physical-backend::filesystem_media::tests::artifact_metadata_read::scheduled_metadata_receipt_and_counter_share_one_identified_operation; lexical counts: std::thread::spawn=0, std::thread::scope=1, std::thread::Scope::spawn=1, available_parallelism=0; barriers, joins and original bounds coordinate the storage-owner claim"
)]
#[test]
fn scheduled_metadata_receipt_and_counter_share_one_identified_operation() {
    let parent = tempfile::tempdir().unwrap();
    let setup_ordinal = setup_metadata_read_ordinal(&parent.path().join("baseline"));
    let gate = MediaPauseGate::for_certification();
    let schedule = MediaFaultSchedule::for_certification(vec![MediaFaultRule::for_certification(
        MediaOperationRole::ReadMetadata,
        setup_ordinal.saturating_add(1),
        MediaFaultDirective::PauseBefore(gate.clone()),
    )])
    .unwrap();
    let media = qualified(&parent.path().join("identified-metadata"), schedule);
    let tree = media.artifact_tree();
    let artifact = RecordArtifactFile::BootstrapCatalog;
    let file = seed_metadata_artifact(&media);
    let before = media.counters();
    let plan = binding(&media);

    let (attempted_operation, completed) = std::thread::scope(|scope| {
        let read = scope.spawn(move || {
            tree.read_scheduled_file_length(
                &file,
                artifact,
                plan,
                BackendQueueExecutionAdaptation::None,
            )
        });
        gate.wait_until_reached();
        let attempted_operation = gate
            .reached_context()
            .expect("metadata read reached the installed pause")
            .operation();
        gate.release();
        let completed = match read.join().unwrap() {
            ScheduledArtifactMetadataReadOutcome::Completed(completed) => completed.physical(),
            outcome => panic!("scheduled metadata read did not complete: {outcome:?}"),
        };
        (attempted_operation, completed)
    });
    let after = media.counters();

    assert_eq!(completed.file_length(), 7);
    if attempted_operation != Some(completed.operation()) {
        panic!("MUTANT_PREDICATE:scheduled-metadata-receipt-identity-diverged");
    }
    assert_eq!(
        after.identified_operation_attempts_for(MediaOperationRole::ReadMetadata),
        before
            .identified_operation_attempts_for(MediaOperationRole::ReadMetadata)
            .saturating_add(1)
    );
    assert_eq!(
        after.completed_operations_for(MediaOperationRole::ReadMetadata),
        before
            .completed_operations_for(MediaOperationRole::ReadMetadata)
            .saturating_add(1)
    );
    media.close();
}

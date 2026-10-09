use worth_store_physical_format::store_namespace::{
    NamespaceInitializationAttempt, StagedNamespaceName,
};
#[cfg(feature = "recovery-runtime-owner")]
use worth_store_physical_format::RecordArtifactFile;

use super::qualify_existing_recovery;
use crate::filesystem_media::{
    namespace_identity_admission, AllocationRequest, AppendRequest, CompletedMediaEffect,
    FilesystemMediaAdmissionAuthority, FilesystemMediaOwner, MediaAllocationMode,
    MediaAllocationResult, MediaEffectStatus, MediaOperationResult, NamespaceFileOpenKind,
    NamespaceFileOpenResult, PositionedWriteRequest, TruncateRequest,
};

#[cfg(feature = "recovery-runtime-owner")]
use crate::filesystem_media::{ArtifactTreeDirectory, ArtifactTreePublicationEffect};

#[test]
fn successful_backend_mutation_invalidates_zero_effect_evidence() {
    let parent = tempfile::tempdir().expect("test parent");
    let root = parent.path().join("store");
    let name = StagedNamespaceName::for_identity(
        NamespaceInitializationAttempt::from_nonzero_bytes([71; 16]).expect("nonzero attempt"),
    );
    let owner = FilesystemMediaOwner::admit(&root, FilesystemMediaAdmissionAuthority::for_test())
        .expect("ordinary media owner");
    let _ = namespace_identity_admission::admit_store_identity(&owner).expect("persisted identity");
    let path = owner.staged_identity_path(&name);
    let handle = match owner.create_new(&path).into_result() {
        NamespaceFileOpenResult::Opened {
            kind: NamespaceFileOpenKind::CreatedNew,
            handle,
        } => handle,
        outcome => panic!("create mutation target: {outcome:?}"),
    };
    drop(handle);
    let _ = owner.close();

    let admitted = qualify_existing_recovery(&root)
        .expect("qualified recovery")
        .admit_persisted_store()
        .expect("admitted persisted Store");
    assert_eq!(admitted.recovery_effect_count(), 0);
    let path = admitted.owner.staged_identity_path(&name);
    let handle = match admitted
        .owner
        .open_existing_for_mutation(&path)
        .into_result()
    {
        NamespaceFileOpenResult::Opened { handle, .. } => handle,
        outcome => panic!("open mutation target: {outcome:?}"),
    };
    assert!(matches!(
        handle
            .positioned_write(PositionedWriteRequest::new(0, b"effect"))
            .result(),
        MediaOperationResult::Completed(CompletedMediaEffect::PositionedWriteCompleted(_))
    ));
    assert!(matches!(
        handle.append(AppendRequest::new(b"append")).result(),
        MediaOperationResult::Completed(CompletedMediaEffect::AppendCompleted(_))
    ));
    assert_eq!(
        handle.truncate(TruncateRequest::new(4)).effect_status(),
        MediaEffectStatus::CompletedEffect
    );
    assert!(matches!(
        handle
            .allocate(AllocationRequest::new(
                8,
                8,
                MediaAllocationMode::LogicalLengthOnly,
            ))
            .result(),
        MediaAllocationResult::Completed(_)
    ));
    drop(handle);
    assert_eq!(admitted.recovery_effect_count(), 4);
    let namespace_effect = admitted
        .owner
        .staged_identity_path(&StagedNamespaceName::for_identity(
            NamespaceInitializationAttempt::from_nonzero_bytes([72; 16])
                .expect("nonzero namespace-effect attempt"),
        ));
    let created = match admitted.owner.create_new(&namespace_effect).into_result() {
        NamespaceFileOpenResult::Opened {
            kind: NamespaceFileOpenKind::CreatedNew,
            handle,
        } => handle,
        outcome => panic!("create namespace effect: {outcome:?}"),
    };
    drop(created);
    assert_eq!(admitted.recovery_effect_count(), 5);
    crate::filesystem_media::artifact_tree_effects::create_directory(
        &admitted.owner,
        admitted.owner.namespace_directory().directory(),
        "recovery-effect-directory",
    )
    .expect("create directory effect");
    assert_eq!(admitted.recovery_effect_count(), 6);
    let _ = admitted.owner.close();
}

#[cfg(feature = "recovery-runtime-owner")]
#[test]
fn recovery_root_protocol_plan_rejects_a_live_catalog_destination() {
    assert_eq!(
        crate::RecoveryRootProtocolPublicationPlan::from_catalog_candidate(
            RecordArtifactFile::BootstrapCatalog,
        ),
        Err(crate::RecoveryRootProtocolPublicationDenial::CatalogCandidateRequired)
    );
}

#[cfg(feature = "recovery-runtime-owner")]
#[test]
fn recovery_root_protocol_and_namespace_return_exact_scheduled_receipts() {
    let parent = tempfile::tempdir().expect("test parent");
    let root = parent.path().join("store");
    initialize_store(&root);
    let parts = qualify_existing_recovery(&root)
        .expect("qualified recovery media")
        .admit_persisted_store()
        .expect("admitted recovery media");
    let plan = crate::RecoveryRootProtocolPublicationPlan::from_catalog_candidate(
        RecordArtifactFile::CatalogCandidate { publication: 7 },
    )
    .expect("catalog candidate");
    materialize_protocol_files(&parts, plan);
    let binding = publication_binding(&parts);
    let media = crate::AdmittedRecoveryFilesystemMedia::from_parts(parts);

    let completed = match media.replace_recovery_root_protocol_scheduled(plan, binding) {
        crate::ScheduledArtifactTreePublicationEffectOutcome::Completed(completed) => completed,
        outcome => panic!("root protocol replacement failed: {outcome:?}"),
    };
    assert!(matches!(
        completed.physical().effect(),
        ArtifactTreePublicationEffect::RootProtocolReplacement { .. }
    ));
    assert_protocol_bytes(&root);

    let synchronized = match media.synchronize_recovery_record_namespace_scheduled(binding) {
        crate::ScheduledArtifactTreePublicationEffectOutcome::Completed(completed) => completed,
        outcome => panic!("record namespace synchronization failed: {outcome:?}"),
    };
    let records = ArtifactTreeDirectory::families()
        .child("records")
        .expect("records directory");
    assert!(matches!(
        synchronized.physical().effect(),
        ArtifactTreePublicationEffect::DirectorySynchronization(directory)
            if directory == &records
    ));
}

#[cfg(feature = "recovery-runtime-owner")]
fn initialize_store(root: &std::path::Path) {
    let owner = FilesystemMediaOwner::admit(root, FilesystemMediaAdmissionAuthority::for_test())
        .expect("ordinary media owner");
    namespace_identity_admission::admit_store_identity(&owner).expect("persisted identity");
    let _ = owner.close();
}

#[cfg(feature = "recovery-runtime-owner")]
fn materialize_protocol_files(
    parts: &super::AdmittedRecoveryParts,
    plan: crate::RecoveryRootProtocolPublicationPlan,
) {
    let tree = parts.artifact_tree();
    let records = ArtifactTreeDirectory::families()
        .child("records")
        .expect("records directory");
    let candidates = ArtifactTreeDirectory::staging()
        .child("records")
        .expect("candidate directory");
    tree.create_directory(&records).expect("records directory");
    tree.create_directory(&candidates)
        .expect("candidate directory");
    for (artifact, bytes) in [
        (plan.previous_candidate(), b"new-previous".as_slice()),
        (plan.current_candidate(), b"new-current".as_slice()),
        (plan.catalog_candidate(), b"new-catalog".as_slice()),
    ] {
        let path = candidates
            .file(&artifact.file_name())
            .expect("candidate path");
        tree.write_new(&path, bytes).expect("candidate bytes");
    }
    for (artifact, bytes) in [
        (
            RecordArtifactFile::PreviousRootSelector,
            b"old-previous".as_slice(),
        ),
        (
            RecordArtifactFile::CurrentRootSelector,
            b"old-current".as_slice(),
        ),
        (
            RecordArtifactFile::BootstrapCatalog,
            b"old-catalog".as_slice(),
        ),
    ] {
        let path = records
            .file(&artifact.file_name())
            .expect("destination path");
        tree.write_new(&path, bytes).expect("destination bytes");
    }
}

#[cfg(feature = "recovery-runtime-owner")]
fn assert_protocol_bytes(root: &std::path::Path) {
    let records = root.join("families").join("records");
    assert_eq!(
        std::fs::read(records.join("root-previous.selector")).expect("previous selector"),
        b"new-previous"
    );
    assert_eq!(
        std::fs::read(records.join("root-current.selector")).expect("current selector"),
        b"new-current"
    );
    assert_eq!(
        std::fs::read(records.join("bootstrap.catalog")).expect("catalog"),
        b"new-catalog"
    );
}

#[cfg(feature = "recovery-runtime-owner")]
fn publication_binding(
    parts: &super::AdmittedRecoveryParts,
) -> crate::BackendQueueExecutionPlanBinding {
    use worth_store_aspect_native::StorePhysicalBoundaryWitness;
    use worth_store_contracts::{
        StorePhysicalAuthorityWitness, ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
    };
    use worth_store_security::{
        StoreAuthenticityRequirement, StoreCustodyPosture, StoreKeyScope, StoreKeyVersionPosture,
        StoreSecurityScopeIdentity, StoreTenantScope,
    };

    let authority = StorePhysicalAuthorityWitness::for_aspect_native_boundary(
        ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
    )
    .expect("physical authority");
    let scope = StoreSecurityScopeIdentity::from_physical_security_scope(
        StorePhysicalBoundaryWitness::from_physical_authority(authority)
            .expect("physical boundary"),
        StoreKeyScope::StoreManagedRoot,
        StoreKeyVersionPosture::Current,
        StoreTenantScope::StoreInternal,
        StoreAuthenticityRequirement::not_required(),
        StoreCustodyPosture::InternalStoreCustody,
    );
    let replay = crate::BackendQueueExecutionReplayBinding::from_store_queue_replay(
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
        crate::BackendQueueExecutionBudgetBinding::new(1, 1, 0, 0, 1, 1, 0, 1, 1, 0),
    );
    crate::BackendQueueExecutionPlanBinding::from_store_replay_binding(
        replay,
        None,
        parts.execution_capability.profile(),
        parts.execution_capability.evidence_class(),
        0,
    )
}

#[cfg(all(feature = "recovery-runtime-owner", feature = "store-runtime-owner"))]
#[test]
fn checkpoint_residue_cleanup_revalidates_current_and_candidate_before_unlink() {
    use crate::{
        execute_recovery_cleanup_removal, AdmittedRecoveryFilesystemMedia,
        BackendRecoveryArtifactExpectation, BackendRecoveryCleanupArtifactRevalidationDenial,
        BackendRecoveryCleanupRemovalDenialCause, BackendRecoveryCleanupRemovalOutcome,
        BackendRecoveryCleanupRemovalRequest,
    };

    for changed_current in [true, false] {
        let parent = tempfile::tempdir().expect("checkpoint residue test parent");
        let root = parent.path().join("store");
        initialize_store(&root);
        std::fs::create_dir_all(root.join("families")).expect("families directory");
        std::fs::create_dir_all(root.join("staging")).expect("staging directory");
        let current_path = root.join("families/checkpoint.current");
        let candidate_path = root.join("staging/checkpoint-0000000000000008.candidate");
        std::fs::write(&current_path, [0x31; 32]).expect("selected checkpoint fixture");
        std::fs::write(&candidate_path, [0x41; 32]).expect("unselected candidate fixture");

        let parts = qualify_existing_recovery(&root)
            .expect("qualified recovery media")
            .admit_persisted_store()
            .expect("admitted persisted Store");
        let binding = publication_binding(&parts);
        let media = AdmittedRecoveryFilesystemMedia::from_parts(parts);
        let current = media
            .observe_current_checkpoint_for_residue(33)
            .expect("bounded selected checkpoint observation");
        let candidate = media
            .observe_next_checkpoint_candidate(7, 33)
            .expect("bounded exact next candidate observation")
            .expect("candidate exists");
        let request = BackendRecoveryCleanupRemovalRequest::new(
            media.store_identity(),
            [0x51; 16],
            [0x61; 32],
            BackendRecoveryArtifactExpectation::from_checkpoint_observation(&current),
            BackendRecoveryArtifactExpectation::from_checkpoint_observation(&candidate),
            [0x71; 32],
        )
        .expect("exact cleanup request");
        if changed_current {
            std::fs::write(&current_path, [0x32; 32]).expect("replace current bytes");
        } else {
            std::fs::write(&candidate_path, [0x42; 32]).expect("replace candidate bytes");
        }

        let outcome = execute_recovery_cleanup_removal(&media, request, binding);
        let BackendRecoveryCleanupRemovalOutcome::DeniedBeforeEffect(denied) = outcome else {
            panic!("changed checkpoint evidence must deny before unlink: {outcome:?}")
        };
        assert!(
            matches!(
                denied.cause(),
                BackendRecoveryCleanupRemovalDenialCause::Revalidation(
                    BackendRecoveryCleanupArtifactRevalidationDenial::CheckpointDigestMismatch { .. }
                ) if changed_current
            ) || matches!(
                denied.cause(),
                BackendRecoveryCleanupRemovalDenialCause::Revalidation(
                    BackendRecoveryCleanupArtifactRevalidationDenial::DigestMismatch { .. }
                ) if !changed_current
            )
        );
        assert!(
            candidate_path.exists(),
            "unselected candidate was not unlinked"
        );
    }
}

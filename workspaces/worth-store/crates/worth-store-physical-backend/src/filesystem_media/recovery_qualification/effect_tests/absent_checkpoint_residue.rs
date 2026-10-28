//! Removal of the first checkpoint candidate when recovery selected no
//! checkpoint: the published slot must still be absent at unlink.

use super::{initialize_store, publication_binding, qualify_existing_recovery};
use crate::{
    execute_recovery_cleanup_removal, AdmittedRecoveryFilesystemMedia,
    BackendRecoveryArtifactExpectation, BackendRecoveryCleanupArtifactRevalidationDenial,
    BackendRecoveryCleanupRemovalDenialCause, BackendRecoveryCleanupRemovalOutcome,
    BackendRecoveryCleanupRemovalRequest,
};

#[test]
fn first_checkpoint_residue_is_removed_only_while_no_checkpoint_is_published() {
    for published_meanwhile in [false, true] {
        let parent = tempfile::tempdir().expect("checkpoint residue test parent");
        let root = parent.path().join("store");
        initialize_store(&root);
        std::fs::create_dir_all(root.join("families")).expect("families directory");
        std::fs::create_dir_all(root.join("staging")).expect("staging directory");
        let current_path = root.join("families/checkpoint.current");
        let candidate_path = root.join("staging/checkpoint-0000000000000001.candidate");
        std::fs::write(&candidate_path, [0x41; 32]).expect("first candidate fixture");

        let parts = qualify_existing_recovery(&root)
            .expect("qualified recovery media")
            .admit_persisted_store()
            .expect("admitted persisted Store");
        let binding = publication_binding(&parts);
        let media = AdmittedRecoveryFilesystemMedia::from_parts(parts);
        let current = media
            .observe_absent_checkpoint_for_residue()
            .expect("no checkpoint is published");
        let candidate = media
            .observe_next_checkpoint_candidate(0, 33)
            .expect("bounded first candidate observation")
            .expect("candidate exists");
        let request = BackendRecoveryCleanupRemovalRequest::without_selected_checkpoint(
            media.store_identity(),
            [0x51; 16],
            [0x61; 32],
            current,
            BackendRecoveryArtifactExpectation::from_checkpoint_observation(&candidate),
            [0x71; 32],
        )
        .expect("exact cleanup request");
        if published_meanwhile {
            std::fs::write(&current_path, [0x31; 32]).expect("publish a checkpoint");
        }

        let outcome = execute_recovery_cleanup_removal(&media, request, binding);
        if published_meanwhile {
            let BackendRecoveryCleanupRemovalOutcome::DeniedBeforeEffect(denied) = outcome else {
                panic!("a published checkpoint must deny before unlink: {outcome:?}")
            };
            assert!(matches!(
                denied.cause(),
                BackendRecoveryCleanupRemovalDenialCause::Revalidation(
                    BackendRecoveryCleanupArtifactRevalidationDenial::CheckpointPresent { .. }
                )
            ));
            assert!(candidate_path.exists(), "the candidate was unlinked");
        } else {
            assert!(
                matches!(outcome, BackendRecoveryCleanupRemovalOutcome::Completed(_)),
                "{outcome:?}"
            );
            assert!(!candidate_path.exists(), "the candidate remains");
        }
    }
}

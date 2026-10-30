//! C8 claim-to-Store Serving journey with governed media substitutions.

pub use std::path::PathBuf;
pub use {
    crate::certified_release_serving::tamper::alter_selected_checkpoint,
    crate::certified_release_serving::tamper::alter_selected_control,
    crate::certified_release_serving::tamper::alter_selected_file,
    crate::certified_release_serving::tamper::alter_selected_routing_root,
    crate::production_entry::certified_release_serving::open_serving_inner,
    crate::production_entry::certified_release_serving::open_serving_with_seal,
    crate::production_entry::certified_release_serving::recovery_request::request,
    crate::production_entry::certified_release_serving::Mutation,
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::WorthStoreRecovery,
};
pub use {
    std::sync::atomic::AtomicBool, std::sync::atomic::Ordering, std::sync::Arc, std::sync::Mutex,
};
pub use {
    worth_store::physical_runtime::PhysicalRecoverySelectedRejoinMismatch,
    worth_store::physical_runtime::RecoveredPhysicalRuntimeConstructionDenial,
};

pub fn recover(root: PathBuf, mutation: Mutation) {
    let request = request(&root);
    let selected_checkpoint = root.join("families/checkpoint.current");
    let after_claim_reached = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let final_rejoin_reached = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let after_claim_flag = std::sync::Arc::clone(&after_claim_reached);
    let final_rejoin_flag = std::sync::Arc::clone(&final_rejoin_reached);
    let selected_descriptor = std::sync::Arc::new(std::sync::Mutex::new(None));
    let descriptor_for_claim = std::sync::Arc::clone(&selected_descriptor);
    let descriptor_for_rejoin = std::sync::Arc::clone(&selected_descriptor);
    let final_checkpoint = selected_checkpoint.clone();
    let final_root = root.clone();
    let outcome =
        worth_store_recovery_runtime::WorthStoreRecovery::certification_recover_with_custody_pauses(
            request,
            move |descriptor| {
                after_claim_flag.store(true, std::sync::atomic::Ordering::SeqCst);
                *descriptor_for_claim.lock().unwrap() = Some(descriptor);
                if mutation == crate::production_entry::certified_release_serving::Mutation::AfterClaimCheckpoint {
                alter_selected_checkpoint(&selected_checkpoint);
            }
            },
            move || {
                final_rejoin_flag.store(true, std::sync::atomic::Ordering::SeqCst);
                if mutation == crate::production_entry::certified_release_serving::Mutation::BeforeFinalRejoinCheckpoint {
                alter_selected_checkpoint(&final_checkpoint);
            } else if mutation == crate::production_entry::certified_release_serving::Mutation::BeforeFinalRejoinControl {
                alter_selected_control(
                    &final_root,
                    descriptor_for_rejoin
                        .lock()
                        .unwrap()
                        .expect("selected descriptor route"),
                );
            }
            },
        );
    assert!(
        after_claim_reached.load(std::sync::atomic::Ordering::SeqCst),
        "C8 never minted the claim"
    );
    if mutation != crate::production_entry::certified_release_serving::Mutation::None
        && mutation
            != crate::production_entry::certified_release_serving::Mutation::AfterSealCheckpoint
        && mutation
            != crate::production_entry::certified_release_serving::Mutation::AfterSealControl
        && mutation != crate::production_entry::certified_release_serving::Mutation::AfterSealWal
        && mutation != crate::production_entry::certified_release_serving::Mutation::AfterSealRoute
    {
        if mutation == crate::production_entry::certified_release_serving::Mutation::BeforeFinalRejoinCheckpoint {
            assert!(
                final_rejoin_reached.load(std::sync::atomic::Ordering::SeqCst),
                "Store initial join was not reached"
            );
        }
        let worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(
            indeterminate,
        ) = outcome
        else {
            panic!("altered selected media must deny at Store handoff: {outcome:?}")
        };
        let expected = match mutation {
            crate::production_entry::certified_release_serving::Mutation::BeforeFinalRejoinControl => {
                worth_store::physical_runtime::PhysicalRecoverySelectedRejoinMismatch::ControlFrame
            }
            _ => worth_store::physical_runtime::PhysicalRecoverySelectedRejoinMismatch::CheckpointBinding,
        };
        assert_eq!(indeterminate.recovery_effects(), 0);
        assert_eq!(
            indeterminate.handoff_failure(),
            Some(worth_store::physical_runtime::RecoveredPhysicalRuntimeConstructionDenial::RejoinSelectedMedia(expected)),
        );
        return;
    }
    assert!(
        final_rejoin_reached.load(std::sync::atomic::Ordering::SeqCst),
        "Store final reread was not reached"
    );
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("genuine selected custody failed Store rejoin: {outcome:?}")
    };
    let seal = handoff
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("selected C8 custody seal");
    if mutation == crate::production_entry::certified_release_serving::Mutation::AfterSealCheckpoint
    {
        alter_selected_checkpoint(&root.join("families/checkpoint.current"));
        open_serving_inner(&root, Some(seal), Some(worth_store::physical_runtime::RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch), true);
    } else if mutation
        == crate::production_entry::certified_release_serving::Mutation::AfterSealControl
    {
        alter_selected_control(
            &root,
            selected_descriptor
                .lock()
                .unwrap()
                .expect("selected descriptor route"),
        );
        open_serving_inner(&root, Some(seal), Some(worth_store::physical_runtime::RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch), true);
    } else if mutation == crate::production_entry::certified_release_serving::Mutation::AfterSealWal
    {
        alter_selected_file(&root.join("families/wal"), |name| name.ends_with(".wal"));
        open_serving_inner(&root, Some(seal), Some(worth_store::physical_runtime::RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch), true);
    } else if mutation
        == crate::production_entry::certified_release_serving::Mutation::AfterSealRoute
    {
        alter_selected_routing_root(&root);
        open_serving_inner(
            &root,
            Some(seal),
            Some(worth_store::physical_runtime::RecordBootstrapDenial::CurrentRootDamaged),
            true,
        );
    } else {
        open_serving_with_seal(&root, seal);
    }
}

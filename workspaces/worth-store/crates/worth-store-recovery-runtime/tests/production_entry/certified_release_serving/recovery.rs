//! C8 claim-to-Store Serving journey with governed media substitutions.

use super::*;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use worth_store::physical_runtime::{
    PhysicalRecoverySelectedRejoinMismatch, RecoveredPhysicalRuntimeConstructionDenial,
};

pub(super) fn recover(root: PathBuf, mutation: Mutation) {
    let request = request(&root);
    let selected_checkpoint = root.join("families/checkpoint.current");
    let after_claim_reached = Arc::new(AtomicBool::new(false));
    let final_rejoin_reached = Arc::new(AtomicBool::new(false));
    let after_claim_flag = Arc::clone(&after_claim_reached);
    let final_rejoin_flag = Arc::clone(&final_rejoin_reached);
    let selected_descriptor = Arc::new(Mutex::new(None));
    let descriptor_for_claim = Arc::clone(&selected_descriptor);
    let descriptor_for_rejoin = Arc::clone(&selected_descriptor);
    let final_checkpoint = selected_checkpoint.clone();
    let final_root = root.clone();
    let outcome = WorthStoreRecovery::certification_recover_with_custody_pauses(
        request,
        move |descriptor| {
            after_claim_flag.store(true, Ordering::SeqCst);
            *descriptor_for_claim.lock().unwrap() = Some(descriptor);
            if mutation == Mutation::AfterClaimCheckpoint {
                alter_selected_checkpoint(&selected_checkpoint);
            }
        },
        move || {
            final_rejoin_flag.store(true, Ordering::SeqCst);
            if mutation == Mutation::BeforeFinalRejoinCheckpoint {
                alter_selected_checkpoint(&final_checkpoint);
            } else if mutation == Mutation::BeforeFinalRejoinControl {
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
        after_claim_reached.load(Ordering::SeqCst),
        "C8 never minted the claim"
    );
    if mutation != Mutation::None
        && mutation != Mutation::AfterSealCheckpoint
        && mutation != Mutation::AfterSealControl
        && mutation != Mutation::AfterSealWal
        && mutation != Mutation::AfterSealRoute
    {
        if mutation == Mutation::BeforeFinalRejoinCheckpoint {
            assert!(
                final_rejoin_reached.load(Ordering::SeqCst),
                "Store initial join was not reached"
            );
        }
        let PhysicalRecoveryOutcome::PublicationIndeterminate(indeterminate) = outcome else {
            panic!("altered selected media must deny at Store handoff: {outcome:?}")
        };
        let expected = match mutation {
            Mutation::BeforeFinalRejoinControl => {
                PhysicalRecoverySelectedRejoinMismatch::ControlFrame
            }
            _ => PhysicalRecoverySelectedRejoinMismatch::CheckpointBinding,
        };
        assert_eq!(indeterminate.recovery_effects(), 0);
        assert_eq!(
            indeterminate.handoff_failure(),
            Some(RecoveredPhysicalRuntimeConstructionDenial::RejoinSelectedMedia(expected)),
        );
        return;
    }
    assert!(
        final_rejoin_reached.load(Ordering::SeqCst),
        "Store final reread was not reached"
    );
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("genuine selected custody failed Store rejoin: {outcome:?}")
    };
    let seal = handoff
        .into_core()
        .into_checkpoint_custody()
        .expect("selected C8 custody seal");
    if mutation == Mutation::AfterSealCheckpoint {
        alter_selected_checkpoint(&root.join("families/checkpoint.current"));
        open_serving_inner(&root, Some(seal), Some(worth_store::physical_runtime::RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch), true);
    } else if mutation == Mutation::AfterSealControl {
        alter_selected_control(
            &root,
            selected_descriptor
                .lock()
                .unwrap()
                .expect("selected descriptor route"),
        );
        open_serving_inner(&root, Some(seal), Some(worth_store::physical_runtime::RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch), true);
    } else if mutation == Mutation::AfterSealWal {
        alter_selected_file(&root.join("families/wal"), |name| name.ends_with(".wal"));
        open_serving_inner(&root, Some(seal), Some(worth_store::physical_runtime::RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch), true);
    } else if mutation == Mutation::AfterSealRoute {
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

use worth_store_physical_isolation::{PhysicalPublicationDenial, PublicationCrashRecoveryOutcome};
use worth_store_physical_isolation::{PublicationCrashStage, RecoveredPublicationStructureKind};

use super::publication_support::{
    admitted_copy_on_write_plan, execute_mixed_tree_recovery_replay,
    execute_publication_recovery_replay, publication_inputs,
    publication_inputs_for_successor_receipt, publication_inputs_with_root_generation,
};

#[test]
fn crash_matrix_recovers_old_or_new_stable_structure_never_mixed_tree() {
    let inputs = publication_inputs();
    let receipt = admitted_copy_on_write_plan(&inputs).complete();

    let before = PublicationCrashRecoveryOutcome::admit_recovery_receipt(
        &receipt,
        execute_publication_recovery_replay(PublicationCrashStage::BeforePublication),
    )
    .unwrap();
    assert_eq!(
        before.recovered().kind(),
        RecoveredPublicationStructureKind::OldStableStructure
    );
    assert_eq!(
        before.recovered().stable_root_epoch(),
        Some(inputs.old_root.epoch().get())
    );

    let during = PublicationCrashRecoveryOutcome::admit_recovery_receipt(
        &receipt,
        execute_publication_recovery_replay(PublicationCrashStage::DuringPublication),
    )
    .unwrap();
    assert_eq!(
        during.recovered().kind(),
        RecoveredPublicationStructureKind::NewStableStructure
    );
    assert_eq!(
        during.recovered().stable_root_epoch(),
        Some(inputs.new_root.epoch().get())
    );

    let after = PublicationCrashRecoveryOutcome::admit_recovery_receipt(
        &receipt,
        execute_publication_recovery_replay(PublicationCrashStage::AfterPublication),
    )
    .unwrap();
    assert_eq!(
        after.recovered().kind(),
        RecoveredPublicationStructureKind::NewStableStructure
    );
    assert_eq!(
        after.recovered().stable_root_epoch(),
        Some(inputs.new_root.epoch().get())
    );

    assert_eq!(
        PublicationCrashRecoveryOutcome::admit_recovery_receipt(
            &receipt,
            execute_mixed_tree_recovery_replay(),
        )
        .unwrap_err(),
        PhysicalPublicationDenial::MixedTreeAfterCrash
    );
}

#[test]
fn recovery_receipt_binds_to_each_publication_receipt_roots() {
    let first = publication_inputs_with_root_generation(721);
    let first_receipt = admitted_copy_on_write_plan(&first).complete();
    let second = publication_inputs_for_successor_receipt(&first_receipt, 722);
    let second_receipt = admitted_copy_on_write_plan(&second).complete();
    let recovery_receipt =
        execute_publication_recovery_replay(PublicationCrashStage::AfterPublication);

    let first_outcome =
        PublicationCrashRecoveryOutcome::admit_recovery_receipt(&first_receipt, recovery_receipt)
            .unwrap();
    let second_outcome =
        PublicationCrashRecoveryOutcome::admit_recovery_receipt(&second_receipt, recovery_receipt)
            .unwrap();

    assert_eq!(
        first_outcome.recovered().stable_root_epoch(),
        Some(first.new_root.epoch().get())
    );
    assert_eq!(
        second_outcome.recovered().stable_root_epoch(),
        Some(second.new_root.epoch().get())
    );
    assert_ne!(
        first_outcome.recovered().stable_root_epoch(),
        second_outcome.recovered().stable_root_epoch()
    );
}

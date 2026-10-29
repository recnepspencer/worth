#[allow(unused_imports)] use crate::c11_arena_copy_crash::historical_displacement::*;
use {worth_store_physical_format::DurableExtentRecordPlacement, worth_store_testkit_recovery_runtime::support::EXTENT_BYTES, std::path::Path, std::path::PathBuf, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationRequest, worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_proof::TransitionOutcome, worth_store_recovery_runtime::WorthStoreRecovery, worth_store_testkit_recovery_runtime::support::append, worth_store_testkit_recovery_runtime::support::configuration, crate::copy_frame, worth_store_testkit_recovery_runtime::support::generation, crate::c11_arena_copy_crash::kill_after_final_wal, crate::c11_arena_copy_crash::load_identity, worth_store_testkit_recovery_runtime::support::open, crate::c11_arena_copy_crash::pre_final, crate::c11_arena_copy_crash::recover, worth_store_testkit_recovery_runtime::support::recovery_request, worth_store_testkit_recovery_runtime::support::scan, crate::c11_arena_copy_crash::selected_extent};
use {std::fs};
use {worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalRetirementDenial};
use {worth_store_physical_format::decode_data_frame_page_lsn, worth_store_physical_format::DurableExtentManifest, worth_store_physical_format::DurableFrameKind, worth_store_physical_format::ExtentArenaFrameLayout, worth_store_physical_format::RecordArtifactFile, worth_store_physical_format::EXTENT_ARENA_MANIFEST_FRAME_BYTES};
use {worth_store_recovery_runtime::PhysicalRecoveryBlockKind};

#[test]
fn historical_copy_destination_is_retained_and_checked_after_later_rewrite() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("store");
    kill_after_final_wal(directory.path());
    let record = load_identity(directory.path());
    crate::copy_frame::current_copy_publication_lsn(&root);
    crate::copy_frame::assert_no_published_copy_resolution(&root);
    let intents = crate::pre_final::independent_copy_intent::produced_copy_intents(&root);
    assert_eq!(intents.len(), 1);

    let first = recover(&root);
    let source = selected_extent(&first, record);
    drop(first);
    let copied = recover(&root);
    let destination = selected_extent(&copied, record);
    let copy_result_root = copied
        .selected_sources()
        .root()
        .selected()
        .selector()
        .root_generation();
    assert_eq!(
        destination.arena_range().arena().get(),
        intents[0].destination[0]
    );
    assert_eq!(
        destination.arena_range().offset(),
        intents[0].destination[1]
    );
    assert_eq!(
        destination.arena_range().length(),
        intents[0].destination[2]
    );
    assert_eq!(
        destination.extent_generation(),
        source.extent_generation() + 1
    );
    drop(copied);

    let retained_destination = arena_range_bytes(&root, destination);
    let serving = open(&root);
    let (_, placement, _) = configuration();
    let selected = scan(&serving)
        .into_iter()
        .filter(|(id, bytes)| {
            id.allocation_epoch() == record.allocation_epoch()
                && id.ordinal() == record.ordinal()
                && bytes.len() == EXTENT_BYTES
                && bytes.iter().all(|&byte| byte == 93)
        })
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), 1, "copy must be selected exactly once");
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([221; 32]))
        .unwrap();
    let request = worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
        key,
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(rewrite)) =
        submission
            .rewrite_selected_extent_record(placement, request, selected[0].0)
            .into_raw()
    else {
        panic!("selected copied extent must admit an ordinary rewrite")
    };
    assert!(matches!(
        rewrite.execute(),
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_)
    ));
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(worth_store::physical_runtime::PhysicalRetirementDenial::Protected),
        "an unresolved copy obligation must keep its protected history"
    );
    append(&serving, placement, [222; 32], b"later-root");
    assert_eq!(
        arena_range_prefix(&root, destination, retained_destination.len()),
        retained_destination
    );
    serving.close();
    let selected_root =
        generation(&std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap());
    assert!(
        selected_root >= copy_result_root + 2,
        "the copy-result root must be historical, not selected or retained-previous"
    );

    let historical_root = root.join("families/records/roots").join(
        worth_store_physical_format::RecordArtifactFile::RootManifest {
            generation: copy_result_root,
        }
        .file_name(),
    );
    assert!(
        historical_root.is_file(),
        "copy-result root must remain addressable"
    );
    let hidden_root = directory.path().join("withheld-copy-result-root");
    std::fs::rename(&historical_root, &hidden_root).unwrap();
    assert_redo_blocked_before_effect(&root);
    std::fs::rename(&hidden_root, &historical_root).unwrap();

    alter_retained_destination_page_lsn(&root, destination);
    assert_redo_blocked_before_effect(&root);
    write_arena_range(&root, destination, &retained_destination);

    let recovered = recover(&root);
    let selected = selected_extent(&recovered, record);
    assert_ne!(selected.arena_range(), destination.arena_range());
    assert_eq!(
        selected.extent_generation(),
        destination.extent_generation() + 1
    );
    drop(recovered);
    let serving = open(&root);
    assert_eq!(
        scan(&serving)
            .iter()
            .filter(|(_, bytes)| bytes.len() == EXTENT_BYTES && bytes.iter().all(|&b| b == 93))
            .count(),
        1,
        "historical copy must not replay into the selected replacement"
    );
    serving.close();
    crate::copy_frame::assert_no_published_copy_resolution(&root);
}

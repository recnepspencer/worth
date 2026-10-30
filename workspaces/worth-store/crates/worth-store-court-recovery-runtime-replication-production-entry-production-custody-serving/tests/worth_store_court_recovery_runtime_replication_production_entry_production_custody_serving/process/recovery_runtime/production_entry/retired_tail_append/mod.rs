//! An ordinary append after an earlier release must not copy an unselected
//! physical tail slot into its new WAL projection.

#[allow(unused_imports)]
use crate::production_entry::retired_tail_append::*;

#[test]
fn retired_physical_tail_slot_forces_fresh_page_with_integrity_admissible_wal() {
    let world = worth_store_testkit_recovery_runtime::production_entry::pending_wal_world::first();
    world.kill_distinct_release_before_checkpoint();
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(world.root()));
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("two real released batches must recover before append: {outcome:?}");
    };
    let seal = handoff
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("selected two-batch custody seal");
    let serving = worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal(world.root(), seal);
    let before = crate::production_entry::retired_tail_append::SelectedRoot::read(world.root());
    assert_eq!(
        before
            .routes
            .iter()
            .filter(|route| route.content_class()
                == worth_store_physical_format::SelectedRecordContentClass::Blob(
                    worth_store_physical_format::BlobRecordKind::ReclaimDescriptorV3
                ))
            .count(),
        2,
        "two genuine V3 release descriptors must precede the ordinary append",
    );
    let tail_record = before
        .manifest
        .last_inline_record()
        .expect("selected inline tail");
    let worth_store_physical_format::CurrentPhysicalRecordPlacement::Inline(tail) = before
        .routes
        .iter()
        .find(|route| route.record() == tail_record)
        .expect("last inline record has a selected route")
    else {
        panic!("last inline record must be inline");
    };
    let prior_page = tail.page_cell();
    let old_page_records = before.tail_page_records(world.root(), prior_page);
    let stale = old_page_records
        .iter()
        .filter(|physical| {
            !before
                .routes
                .iter()
                .any(|route| route.record() == physical.record())
        })
        .map(|physical| physical.record())
        .collect::<Vec<_>>();
    assert!(
        !stale.is_empty(),
        "baseline must have an earlier-retired physical tail slot"
    );
    assert!(old_page_records
        .iter()
        .any(|physical| physical.record() == tail_record));

    let placement = worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(64).unwrap())
        .admit(
            worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
                worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
                    .admit()
                    .unwrap(),
            ),
        )
        .unwrap();
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([0xb2; 32]),
        )
        .unwrap();
    let worth_proof::TransitionOutcome::Success(
        worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
    ) = submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([
                b"append-after-retired-tail".as_slice(),
            ])
            .unwrap(),
            placement,
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                    .unwrap(),
            ),
        )
        .into_raw()
    else {
        panic!("ordinary append after retired tail must prepare");
    };
    assert!(matches!(
        prepared.execute(),
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_)
    ));

    let after = crate::production_entry::retired_tail_append::SelectedRoot::read(world.root());
    for prior in &before.routes {
        assert!(
            after.routes.contains(prior),
            "surviving selected route changed"
        );
    }
    for retired in stale {
        assert!(
            !after.routes.iter().any(|route| route.record() == retired),
            "append resurrected an earlier-retired physical slot"
        );
    }
    let new_routes = after
        .routes
        .iter()
        .filter(|route| {
            !before
                .routes
                .iter()
                .any(|old| old.record() == route.record())
        })
        .collect::<Vec<_>>();
    assert_eq!(
        new_routes.len(),
        1,
        "ordinary batch appends exactly one record"
    );
    let worth_store_physical_format::CurrentPhysicalRecordPlacement::Inline(appended) =
        new_routes[0]
    else {
        panic!("short appended record must be inline");
    };
    assert_ne!(
        appended.page_cell(),
        prior_page,
        "retired physical tail must not be copied"
    );

    serving.close();
    let root = world.root().to_path_buf();
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(&root));
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(_) = outcome else {
        panic!("C9 must admit the real producer WAL after fresh-page fallback: {outcome:?}");
    };
}

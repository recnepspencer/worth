#[allow(unused_imports)]
use crate::physical_record_journeys::durability_admission::checkpoint_wal_reclamation::*;

use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_with_wal_policy, worth_store_testkit_store::physical_record_journeys::success};
use {worth_store_testkit_store::physical_record_journeys::durability_admission::checkpoint_retained_wal_tail::durable_group, worth_store_testkit_store::physical_record_journeys::durability_admission::checkpoint_retained_wal_tail::publish_after_foreground_rotations, worth_store_testkit_store::physical_record_journeys::durability_admission::checkpoint_retained_wal_tail::rotation_world};
use {worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::segment_inventory::inspect_wal_inventory};
use {worth_store_testkit_store::physical_record_journeys::durability_admission::wal_rotation::append_group, worth_store_testkit_store::physical_record_journeys::durability_admission::wal_rotation::wal_policy};

#[test]
fn reclaimed_wal_prefix_remains_absent_after_fresh_process_reopen() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let (serving, placement) = rotation_world(&store_root);
    let submission = serving.certification_record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([51; 32]),
        )
        .unwrap();
    let base = prepare_with_key(&submission, placement, key.clone(), b"reopen-base");
    let base_identity = base.mutation_identity();
    let base = append_group(&submission, vec![base]);
    assert!(matches!(
        submission.synchronize_appended_wal_group(base),
        worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome::Durable(_)
    ));
    let (completed, _) = publish_after_foreground_rotations(&serving, submission, placement);
    assert_reclaimed_prefix(&completed.wal_reclamation(), 1);
    assert_unmaterialized_candidates(&store_root, &serving);
    assert_eq!(
        serving
            .certification_record_submission()
            .wal_observation()
            .unwrap()
            .appended_frames(),
        5
    );
    let retained_charge = serving.certification_charged_growth_bytes();
    serving.close();

    assert_eq!(
        inspect_wal_inventory(&store_root).unwrap().segments(),
        &[(2, 1), (3, 1)]
    );
    let media_owner = worth_store_testkit_store::physical_record_journeys::media(&store_root);
    let durability = durability_with_wal_policy(&media_owner, wal_policy(6));
    let (format, placement, access) = configuration();
    let reopened = success(media_owner.open_record_store(
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability),
    ));
    let observation = reopened
        .certification_record_submission()
        .wal_observation()
        .unwrap();
    assert_eq!(observation.active_segment_count(), 2);
    assert_eq!(observation.reopened_frames(), 4);
    assert!(!observation.sealed_for_inspection());
    assert_unmaterialized_candidates(&store_root, &reopened);
    // These five single-page mutations reached WAL durability, not data
    // dispatch. Reopen restores retained media, not their transient candidate
    // reservations. Completed-publication parity is a separate journey.
    let pending_candidate_bytes = 5 * u64::from(format.declaration().page_size().bytes());
    assert_eq!(
        retained_charge,
        reopened.certification_charged_growth_bytes() + pending_candidate_bytes,
        "reopen must not resurrect unwritten candidate reservations or reclaimed WAL"
    );
    let replay = prepare_with_key(
        &reopened.certification_record_submission(),
        placement,
        key,
        b"reopen-base",
    );
    assert_eq!(
        replay.disposition(),
        worth_store::physical_runtime::PhysicalMutationAdmissionDisposition::DuplicateUnresolved
    );
    assert_eq!(replay.mutation_identity(), base_identity);
    reopened.close();
}

#[test]
fn denied_reclamation_preserves_the_exact_live_inventory_for_retry() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let (serving, placement, activation) = fault_rotation_world(
        &store_root,
        worth_store::physical_runtime::certification::MediaFaultDirective::FailBefore {
            kind: std::io::ErrorKind::Other,
            raw_os_error: None,
        },
    );
    let submission = serving.certification_record_submission();
    durable_group(&submission, placement, &[(61, b"denied-base")]);
    activation.arm().unwrap();
    let (completed, _) =
        publish_after_foreground_rotations(&serving, submission.clone(), placement);

    match completed.wal_reclamation() {
        worth_store::physical_runtime::PhysicalWalReclamationObservation::DeferredBeforeEffect(
            report,
        ) => {
            assert_eq!(report.planned_segments(), 1);
            assert_eq!(report.reclaimed_segments(), 0);
            assert_eq!(report.reclaimed_bytes(), 0);
            assert!(report.first_unreclaimed().is_some());
        }
        other => panic!("denied reclamation was not reported as preeffect: {other:?}"),
    }
    assert!(activation.is_consumed());
    assert_eq!(
        inspect_wal_inventory(&store_root).unwrap().segments(),
        &[(1, 1), (2, 1), (3, 1)]
    );
    let wal = submission.wal_observation().unwrap();
    assert_eq!(wal.active_segment_count(), 3);
    assert_eq!(wal.reclaimed_segments(), 0);
    assert!(!wal.sealed_for_inspection());
    serving.close();
}

#[test]
fn second_delete_denial_preserves_the_exact_partially_reclaimed_prefix() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let world = second_delete_fault_rotation_world(
        &store_root,
        worth_store::physical_runtime::certification::MediaFaultDirective::FailBefore {
            kind: std::io::ErrorKind::Other,
            raw_os_error: None,
        },
    );
    let submission = world.serving.certification_record_submission();
    durable_group(&submission, world.placement, &[(81, b"partial-base")]);
    durable_group(&submission, world.placement, &[(82, b"partial-second")]);
    world.pause_activation.arm().unwrap();
    let (completed, _) = std::thread::scope(|scope| {
        let checkpoint = scope.spawn(|| {
            publish_after_foreground_rotations(&world.serving, submission.clone(), world.placement)
        });
        world.first_delete_gate.wait_until_reached();
        world.fail_activation.arm().unwrap();
        world.first_delete_gate.release();
        checkpoint.join().unwrap()
    });

    match completed.wal_reclamation() {
        worth_store::physical_runtime::PhysicalWalReclamationObservation::DeferredBeforeEffect(
            report,
        ) => {
            assert_eq!(report.planned_segments(), 2);
            assert_eq!(report.reclaimed_segments(), 1);
            assert!(report.reclaimed_bytes() > 0);
            let first_unreclaimed = report.first_unreclaimed().unwrap();
            assert_eq!(first_unreclaimed.segment().get(), 2);
            assert_eq!(first_unreclaimed.generation().get(), 1);
        }
        other => panic!("partial reclamation did not preserve retry truth: {other:?}"),
    }
    assert_eq!(
        inspect_wal_inventory(&store_root).unwrap().segments(),
        &[(2, 1), (3, 1), (4, 1)]
    );
    let wal = submission.wal_observation().unwrap();
    assert_eq!(wal.active_segment_count(), 3);
    assert_eq!(wal.reclaimed_segments(), 1);
    assert!(!wal.sealed_for_inspection());
    assert!(world.pause_activation.is_consumed());
    assert!(world.fail_activation.is_consumed());
    world.serving.close();
}

#[test]
fn indeterminate_reclamation_seals_without_consuming_inventory_truth() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let (serving, placement, activation) = fault_rotation_world(
        &store_root,
        worth_store::physical_runtime::certification::MediaFaultDirective::IndeterminateAfterEffect,
    );
    let submission = serving.certification_record_submission();
    durable_group(&submission, placement, &[(71, b"indeterminate-base")]);
    activation.arm().unwrap();
    let (completed, _) =
        publish_after_foreground_rotations(&serving, submission.clone(), placement);

    match completed.wal_reclamation() {
        worth_store::physical_runtime::PhysicalWalReclamationObservation::InspectionRequired(
            report,
        ) => {
            assert_eq!(report.planned_segments(), 1);
            assert_eq!(report.reclaimed_segments(), 0);
            assert!(report.first_unreclaimed().is_some());
        }
        other => panic!("indeterminate reclamation did not require inspection: {other:?}"),
    }
    assert!(activation.is_consumed());
    let wal = submission.wal_observation().unwrap();
    assert_eq!(wal.active_segment_count(), 3);
    assert_eq!(wal.reclaimed_segments(), 0);
    assert!(wal.sealed_for_inspection());
    assert_eq!(
        inspect_wal_inventory(&store_root).unwrap().segments(),
        &[(2, 1), (3, 1)]
    );
    serving.close();
}

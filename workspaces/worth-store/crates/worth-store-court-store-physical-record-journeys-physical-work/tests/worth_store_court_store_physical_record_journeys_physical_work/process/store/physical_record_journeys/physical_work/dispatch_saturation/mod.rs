#[allow(unused_imports)] use crate::physical_record_journeys::physical_work::dispatch_saturation::*;
use {std::time::Duration};

use {tempfile::tempdir};
use {worth_proof::NonEmpty};
use {worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress, worth_store::physical_runtime::ManagedPhysicalIntegrityScrubRequest, worth_store::physical_runtime::PhysicalExecutorCommand, worth_store::physical_runtime::PhysicalIntegrityScrubDeferral, worth_store::physical_runtime::PhysicalIntegrityScrubReadDeferral, worth_store::physical_runtime::PhysicalIntegrityScrubTarget, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalRootPublicationPreparationOutcome, worth_store::physical_runtime::PhysicalRootPublicationWorkFailureCause, worth_store::physical_runtime::PhysicalRootReplacementFailureCause, worth_store::physical_runtime::PhysicalRootReplacementOutcome, worth_store::physical_runtime::PhysicalSchedulerDenial, worth_store::physical_runtime::PhysicalStoreCloseOutcome, worth_store::physical_runtime::PhysicalWorkCapacity, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordSchedulerReservationDenial};
use {worth_store_io_scheduler::foreground_reservation::BandwidthToken, worth_store_io_scheduler::foreground_reservation::DirtyPageBudget, worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration, worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope, worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget, worth_store_io_scheduler::foreground_reservation::QueueSlot, worth_store_io_scheduler::foreground_reservation::WorkerPermit, worth_store_io_scheduler::foreground_reservation::WriteBackWindow};
use {worth_store_physical_backend::MediaOperationRole};
use {worth_store_physical_format::PhysicalArtifactReadTarget, worth_store_physical_format::PhysicalRecordFormatDeclaration, worth_store_physical_format::RecordArtifactFile};
use {worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalByteRange};

use {worth_store_testkit_store::physical_record_journeys::physical_work::executor::admitted_write, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::mutation_pair::foreground_saturation_fixture, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_open_with_work_profile, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::mutation_pair::whole_catalog_mutation_fixture, worth_store_testkit_store::physical_work::scheduler::policy_evidence::policy_receipt, worth_store_testkit_store::physical_record_journeys::physical_work::scheduler::ready_work, worth_store_testkit_store::physical_record_journeys::physical_work::scheduler::secure_demand, worth_store_testkit_store::physical_record_journeys::physical_work::scheduler::write_demand};

#[test]
fn ready_scrub_bounds_foreground_refill_across_real_io() {
    let root = tempdir().unwrap();
    let (profile, writes) = foreground_saturation_fixture();
    serving_from_initialization_with_work_profile(root.path(), profile.clone()).close();
    let serving = serving_from_open_with_work_profile(root.path(), profile);
    let length = std::fs::metadata(root.path().join("families/records/root-current.selector"))
        .unwrap()
        .len();
    let target = worth_store::physical_runtime::PhysicalIntegrityScrubTarget::new(
        worth_store_physical_format::PhysicalArtifactReadTarget::Record(worth_store_physical_format::RecordArtifactFile::CurrentRootSelector),
        worth_store_physical_integrity::PhysicalArtifactScope::current_root_selector(
            serving.store_identity(),
            worth_store_physical_format::PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
            worth_store_physical_integrity::PhysicalByteRange::new(0, length).unwrap(),
        ),
    )
    .unwrap();
    let request = worth_store::physical_runtime::ManagedPhysicalIntegrityScrubRequest::new(
        serving.store_identity(),
        [target],
        65_536,
        65_536,
        std::time::Duration::from_secs(30),
    )
    .unwrap();
    let before = serving.media_counters();
    let [first, second, third, fourth] = writes;
    for (request, bytes) in [first, second, third].into_iter().zip([
        b"write-01".as_slice(),
        b"write-02".as_slice(),
        b"write-03".as_slice(),
    ]) {
        execute_write(&serving, request, bytes);
    }
    let slots = serving
        .physical_scheduler_capacity()
        .configured()
        .queue_slots();
    let blocker = worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration::artifact_metadata_read()
        .with_latency_envelope(worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope::bounded_interference(
            "scrub-saturation-blocker",
            1,
        ))
        .with_budget(
            worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget::new()
                .with_queue_slots(worth_store_io_scheduler::foreground_reservation::QueueSlot::new(slots / 2 + 1).unwrap())
                .with_worker_permits(worth_store_io_scheduler::foreground_reservation::WorkerPermit::new(1).unwrap()),
        );
    let blocker = serving
        .reserve_physical_scheduler_foreground(blocker)
        .expect("foreground can hold the background share before the scrub is ready");
    let mut scrub = serving.start_physical_integrity_scrub(request).unwrap();
    assert!(
        matches!(
            scrub.next_window(),
            worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::Deferred(
                worth_store::physical_runtime::PhysicalIntegrityScrubDeferral::SchedulerOrDependency(
                    worth_store::physical_runtime::PhysicalIntegrityScrubReadDeferral::Capacity(_)
                )
            )
        ),
        "the ready scrub stays queued when its quantum does not fit"
    );
    match serving.reserve_physical_scheduler_foreground(page_write_lane()) {
        Err(worth_store::physical_runtime::RecordSchedulerReservationDenial::OwedBackgroundTurn) => {}
        Err(denial) => {
            panic!("a refilled foreground write must wait for the ready scrub, got {denial:?}")
        }
        Ok(_) => panic!("a refilled foreground write must wait for the ready scrub"),
    }
    drop(blocker);
    let progress = scrub.next_window();
    assert!(
        matches!(
            progress,
            worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::WindowInspected(_)
        ),
        "the owed scrub window must read real bytes, got {progress:?}"
    );
    execute_write(&serving, fourth, b"write-04");
    let after = serving.media_counters();
    assert_eq!(
        after.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
            - before.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        4
    );
    assert!(
        after.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedRead)
            > before.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedRead)
    );
    drop(scrub);
    assert!(matches!(
        serving.close_plan().execute(),
        worth_store::physical_runtime::PhysicalStoreCloseOutcome::Closed { .. }
    ));
}

#[test]
fn whole_catalog_write_conflicts_before_a_second_effect() {
    let root = tempdir().unwrap();
    let (profile, range, whole) = whole_catalog_mutation_fixture();
    serving_from_initialization_with_work_profile(root.path(), profile.clone()).close();
    let serving = serving_from_open_with_work_profile(root.path(), profile);
    let before = serving.media_counters();
    let held = admitted_write(&serving, range);
    let ready = ready_work(&serving, whole);
    let demand = write_demand(&serving, ready);
    let requested_budget = demand.queue_work().requested_budget();
    let backend = serving
        .admit_physical_scheduler_capability(demand.queue_work().backend_requirement())
        .unwrap();
    let demand = secure_demand(demand, &backend);
    match serving.admit_physical_scheduler_demand(
        demand,
        &backend,
        policy_receipt(requested_budget),
    ) {
        Err(worth_store::physical_runtime::PhysicalSchedulerDenial::EffectConflict) => {}
        Err(denial) => {
            panic!("a whole-catalog write must conflict with a live range, got {denial:?}")
        }
        Ok(_) => panic!("a whole-catalog write must conflict with a live range"),
    }
    serving
        .execute_physical_work(
            worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(held, b"catalog!".as_slice()).unwrap(),
        )
        .unwrap();
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
            - before.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        1
    );
    assert!(matches!(
        serving.close_plan().execute(),
        worth_store::physical_runtime::PhysicalStoreCloseOutcome::Closed { .. }
    ));
}

#[test]
fn one_permit_capacity_denies_the_second_write_before_its_effect() {
    let root = tempdir().unwrap();
    let (profile, writes) = foreground_saturation_fixture();
    let profile = profile.with_capacity(worth_store::physical_runtime::PhysicalWorkCapacity::new(1, 1, 1, 4_096, 4_096).unwrap());
    serving_from_initialization_with_work_profile(root.path(), profile.clone()).close();
    let serving = serving_from_open_with_work_profile(root.path(), profile);
    let before = serving.media_counters();
    let [first, _, _, _] = writes;
    let held = admitted_write(&serving, first);
    match serving.reserve_physical_scheduler_foreground(page_write_lane()) {
        Err(worth_store::physical_runtime::RecordSchedulerReservationDenial::Admission(_)) => {}
        Err(denial) => panic!("the second write must exhaust the single permit, got {denial:?}"),
        Ok(_) => panic!("the second write must exhaust the single permit"),
    }
    let command = worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(held, b"write-01".as_slice()).unwrap();
    serving.execute_physical_work(command).unwrap();
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
            - before.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        1
    );
    assert!(matches!(
        serving.close_plan().execute(),
        worth_store::physical_runtime::PhysicalStoreCloseOutcome::Closed { .. }
    ));
}

#[test]
fn root_catalog_replacement_conflicts_with_a_live_catalog_range() {
    let root = tempdir().unwrap();
    let (profile, range, _) = whole_catalog_mutation_fixture();
    serving_from_initialization_with_work_profile(root.path(), profile.clone()).close();
    let serving = serving_from_open_with_work_profile(root.path(), profile);
    let (_, placement, _) = worth_store_testkit_store::physical_record_journeys::configuration();
    let submission = serving.certification_record_submission();
    let settled = worth_store_testkit_store::durable_publication::wal_data::settle_single(
        &submission,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([71; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"root-conflict".as_slice()]).unwrap(),
    );
    let joined = submission
        .join_data_settled_group(settled.basis, worth_proof::NonEmpty::new(settled.member, Vec::new()))
        .unwrap_or_else(|_| panic!("the settled singleton must join"));
    let prepared = match submission.prepare_root_publication(joined) {
        worth_store::physical_runtime::PhysicalRootPublicationPreparationOutcome::Prepared(prepared) => prepared,
        worth_store::physical_runtime::PhysicalRootPublicationPreparationOutcome::NotStarted(failure) => {
            panic!(
                "root preparation must reach the replacement boundary, got {:?}",
                failure.cause()
            )
        }
        worth_store::physical_runtime::PhysicalRootPublicationPreparationOutcome::InspectionRequired(failure) => {
            panic!(
                "root preparation must reach the replacement boundary, got {:?}",
                failure.cause()
            )
        }
    };
    let before = serving.media_counters();
    let held = admitted_write(&serving, range);
    // The refused publication still owns the prepared transition. Dropping it
    // abandons a candidate effect that already started and requires inspection.
    // Hold it across the live catalog write so the conflict itself is the proof.
    let refused = match submission.replace_prepared_root(prepared) {
        worth_store::physical_runtime::PhysicalRootReplacementOutcome::NotStarted(failure) => failure,
        worth_store::physical_runtime::PhysicalRootReplacementOutcome::Replaced(_) => {
            panic!("root replacement must not pass a live catalog range")
        }
        worth_store::physical_runtime::PhysicalRootReplacementOutcome::InspectionRequired(_) => {
            panic!("root replacement must be refused before an effect")
        }
    };
    assert_eq!(
        refused.cause(),
        worth_store::physical_runtime::PhysicalRootReplacementFailureCause::Work(
            worth_store::physical_runtime::PhysicalRootPublicationWorkFailureCause::EffectConflict
        )
    );
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        before.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
    );
    serving
        .execute_physical_work(
            worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(held, b"catalog!".as_slice()).unwrap(),
        )
        .unwrap();
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
            - before.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        1
    );
    assert!(matches!(
        serving.close_plan().execute(),
        worth_store::physical_runtime::PhysicalStoreCloseOutcome::Closed { .. }
    ));
    drop(refused);
}

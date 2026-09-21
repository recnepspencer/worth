use std::time::Duration;

use tempfile::tempdir;
use worth_proof::NonEmpty;
use worth_store::physical_runtime::{
    ManagedPhysicalIntegrityScrubProgress, ManagedPhysicalIntegrityScrubRequest,
    PhysicalExecutorCommand, PhysicalIntegrityScrubDeferral, PhysicalIntegrityScrubReadDeferral,
    PhysicalIntegrityScrubTarget, PhysicalMutationIdempotencyMaterial,
    PhysicalRootPublicationPreparationOutcome, PhysicalRootPublicationWorkFailureCause,
    PhysicalRootReplacementFailureCause, PhysicalRootReplacementOutcome, PhysicalSchedulerDenial,
    PhysicalStoreCloseOutcome, PhysicalWorkCapacity, RecordAppendBatch,
    RecordSchedulerReservationDenial,
};
use worth_store_io_scheduler::foreground_reservation::{
    BandwidthToken, DirtyPageBudget, ForegroundLaneDeclaration, ForegroundLatencyEnvelope,
    ForegroundResourceBudget, QueueSlot, WorkerPermit, WriteBackWindow,
};
use worth_store_physical_backend::MediaOperationRole;
use worth_store_physical_format::{
    PhysicalArtifactReadTarget, PhysicalRecordFormatDeclaration, RecordArtifactFile,
};
use worth_store_physical_integrity::{PhysicalArtifactScope, PhysicalByteRange};

use super::{
    executor::admitted_write,
    fixture::{
        foreground_saturation_fixture, serving_from_initialization_with_work_profile,
        serving_from_open_with_work_profile, whole_catalog_mutation_fixture,
    },
    policy_receipt,
    scheduler::{ready_work, secure_demand, write_demand},
};

fn page_write_lane() -> ForegroundLaneDeclaration {
    ForegroundLaneDeclaration::ordinary_page_write()
        .with_latency_envelope(ForegroundLatencyEnvelope::bounded_interference(
            "physical-work-saturation-write",
            2,
        ))
        .with_budget(
            ForegroundResourceBudget::new()
                .with_queue_slots(QueueSlot::new(1).unwrap())
                .with_bandwidth(BandwidthToken::bytes(4_096).unwrap())
                .with_write_back(WriteBackWindow::pages(1).unwrap())
                .with_dirty_pages(DirtyPageBudget::pages(1).unwrap())
                .with_worker_permits(WorkerPermit::new(1).unwrap()),
        )
}

fn execute_write(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    request: worth_store::physical_runtime::PhysicalMutationWorkRequest,
    bytes: &[u8],
) {
    let command = PhysicalExecutorCommand::exact_write(admitted_write(serving, request), bytes)
        .unwrap();
    serving.execute_physical_work(command).unwrap();
}

#[test]
fn ready_scrub_bounds_foreground_refill_across_real_io() {
    let root = tempdir().unwrap();
    let (profile, writes) = foreground_saturation_fixture();
    serving_from_initialization_with_work_profile(root.path(), profile.clone()).close();
    let serving = serving_from_open_with_work_profile(root.path(), profile);
    let length = std::fs::metadata(root.path().join("families/records/root-current.selector"))
        .unwrap()
        .len();
    let target = PhysicalIntegrityScrubTarget::new(
        PhysicalArtifactReadTarget::Record(RecordArtifactFile::CurrentRootSelector),
        PhysicalArtifactScope::current_root_selector(
            serving.store_identity(),
            PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
            PhysicalByteRange::new(0, length).unwrap(),
        ),
    )
    .unwrap();
    let request = ManagedPhysicalIntegrityScrubRequest::new(
        serving.store_identity(),
        [target],
        65_536,
        65_536,
        Duration::from_secs(30),
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
    let blocker = ForegroundLaneDeclaration::artifact_metadata_read()
        .with_latency_envelope(ForegroundLatencyEnvelope::bounded_interference(
            "scrub-saturation-blocker",
            1,
        ))
        .with_budget(
            ForegroundResourceBudget::new()
                .with_queue_slots(QueueSlot::new(slots / 2 + 1).unwrap())
                .with_worker_permits(WorkerPermit::new(1).unwrap()),
        );
    let blocker = serving
        .reserve_physical_scheduler_foreground(blocker)
        .expect("foreground can hold the background share before the scrub is ready");
    let mut scrub = serving.start_physical_integrity_scrub(request).unwrap();
    assert!(
        matches!(
            scrub.next_window(),
            ManagedPhysicalIntegrityScrubProgress::Deferred(
                PhysicalIntegrityScrubDeferral::SchedulerOrDependency(
                    PhysicalIntegrityScrubReadDeferral::Capacity(_)
                )
            )
        ),
        "the ready scrub stays queued when its quantum does not fit"
    );
    match serving.reserve_physical_scheduler_foreground(page_write_lane()) {
        Err(RecordSchedulerReservationDenial::OwedBackgroundTurn) => {}
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
            ManagedPhysicalIntegrityScrubProgress::WindowInspected(_)
        ),
        "the owed scrub window must read real bytes, got {progress:?}"
    );
    execute_write(&serving, fourth, b"write-04");
    let after = serving.media_counters();
    assert_eq!(
        after.attempts_for(MediaOperationRole::PositionedWrite)
            - before.attempts_for(MediaOperationRole::PositionedWrite),
        4
    );
    assert!(
        after.attempts_for(MediaOperationRole::PositionedRead)
            > before.attempts_for(MediaOperationRole::PositionedRead)
    );
    drop(scrub);
    assert!(matches!(
        serving.close_plan().execute(),
        PhysicalStoreCloseOutcome::Closed { .. }
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
        Err(PhysicalSchedulerDenial::EffectConflict) => {}
        Err(denial) => panic!("a whole-catalog write must conflict with a live range, got {denial:?}"),
        Ok(_) => panic!("a whole-catalog write must conflict with a live range"),
    }
    serving
        .execute_physical_work(
            PhysicalExecutorCommand::exact_write(held, b"catalog!".as_slice()).unwrap(),
        )
        .unwrap();
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite)
            - before.attempts_for(MediaOperationRole::PositionedWrite),
        1
    );
    assert!(matches!(
        serving.close_plan().execute(),
        PhysicalStoreCloseOutcome::Closed { .. }
    ));
}

#[test]
fn one_permit_capacity_denies_the_second_write_before_its_effect() {
    let root = tempdir().unwrap();
    let (profile, writes) = foreground_saturation_fixture();
    let profile = profile.with_capacity(PhysicalWorkCapacity::new(1, 1, 1, 4_096, 4_096).unwrap());
    serving_from_initialization_with_work_profile(root.path(), profile.clone()).close();
    let serving = serving_from_open_with_work_profile(root.path(), profile);
    let before = serving.media_counters();
    let [first, _, _, _] = writes;
    let held = admitted_write(&serving, first);
    match serving.reserve_physical_scheduler_foreground(page_write_lane()) {
        Err(RecordSchedulerReservationDenial::Admission(_)) => {}
        Err(denial) => panic!("the second write must exhaust the single permit, got {denial:?}"),
        Ok(_) => panic!("the second write must exhaust the single permit"),
    }
    let command = PhysicalExecutorCommand::exact_write(held, b"write-01".as_slice()).unwrap();
    serving.execute_physical_work(command).unwrap();
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite)
            - before.attempts_for(MediaOperationRole::PositionedWrite),
        1
    );
    assert!(matches!(
        serving.close_plan().execute(),
        PhysicalStoreCloseOutcome::Closed { .. }
    ));
}

#[test]
fn root_catalog_replacement_conflicts_with_a_live_catalog_range() {
    let root = tempdir().unwrap();
    let (profile, range, _) = whole_catalog_mutation_fixture();
    serving_from_initialization_with_work_profile(root.path(), profile.clone()).close();
    let serving = serving_from_open_with_work_profile(root.path(), profile);
    let (_, placement, _) = super::super::configuration();
    let submission = serving.certification_record_submission();
    let settled = crate::durable_publication::settle_single(
        &submission,
        placement,
        PhysicalMutationIdempotencyMaterial::new([71; 32]),
        RecordAppendBatch::try_from_iter([b"root-conflict".as_slice()]).unwrap(),
    );
    let joined = submission
        .join_data_settled_group(settled.basis, NonEmpty::new(settled.member, Vec::new()))
        .unwrap_or_else(|_| panic!("the settled singleton must join"));
    let prepared = match submission.prepare_root_publication(joined) {
        PhysicalRootPublicationPreparationOutcome::Prepared(prepared) => prepared,
        PhysicalRootPublicationPreparationOutcome::NotStarted(failure) => {
            panic!(
                "root preparation must reach the replacement boundary, got {:?}",
                failure.cause()
            )
        }
        PhysicalRootPublicationPreparationOutcome::InspectionRequired(failure) => {
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
        PhysicalRootReplacementOutcome::NotStarted(failure) => failure,
        PhysicalRootReplacementOutcome::Replaced(_) => {
            panic!("root replacement must not pass a live catalog range")
        }
        PhysicalRootReplacementOutcome::InspectionRequired(_) => {
            panic!("root replacement must be refused before an effect")
        }
    };
    assert_eq!(
        refused.cause(),
        PhysicalRootReplacementFailureCause::Work(
            PhysicalRootPublicationWorkFailureCause::EffectConflict
        )
    );
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite),
        before.attempts_for(MediaOperationRole::PositionedWrite)
    );
    serving
        .execute_physical_work(
            PhysicalExecutorCommand::exact_write(held, b"catalog!".as_slice()).unwrap(),
        )
        .unwrap();
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite)
            - before.attempts_for(MediaOperationRole::PositionedWrite),
        1
    );
    assert!(matches!(
        serving.close_plan().execute(),
        PhysicalStoreCloseOutcome::Closed { .. }
    ));
    drop(refused);
}

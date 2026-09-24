use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    PhysicalReadWorkRequest, PhysicalWorkCapacity, PhysicalWorkCapacityDimension,
    PhysicalWorkReadiness, RecordSchedulerReservationDenial, ServingPhysicalRuntime,
};
use worth_store_io_scheduler::foreground_reservation::{
    BandwidthToken, DirtyPageBudget, ForegroundLaneDeclaration, ForegroundLatencyEnvelope,
    ForegroundReservationAdmissionDenial, ForegroundReservationResourceShortfall,
    ForegroundResourceBudget, PhysicalInstanceForegroundAdmissionDenial, QueueSlot, WorkerPermit,
    WriteBackWindow,
};
use worth_store_physical_backend::MediaOperationRole;

use super::super::physical_work::{serving_from_initialization_with_work_profile, work_fixture};

#[test]
fn canonical_profile_keeps_eight_ready_slots_and_two_background_dispatches() {
    let parent = tempfile::tempdir().unwrap();
    let (profile, request, _) = work_fixture();
    let capacity = PhysicalWorkCapacity::new(8, 1, 1_024, 1024 * 1024, 4 * 1024 * 1024)
        .unwrap()
        .with_dispatch_permits(4)
        .unwrap();
    let serving = serving_from_initialization_with_work_profile(
        &parent.path().join("store"),
        profile.with_capacity(capacity),
    );
    let configured = serving.physical_scheduler_capacity().configured();
    assert_eq!(configured.queue_slots(), 8);
    assert_eq!(configured.worker_permits(), 4);

    let submission = serving.physical_read_submission();
    for _ in 0..8 {
        match submission.submit(request.clone()).into_raw() {
            TransitionOutcome::Success(_) => {}
            other => panic!("eight ready slots must admit, saw {other:?}"),
        }
    }
    let before = serving.media_counters();
    assert!(matches!(
        submission.submit(request).into_raw(),
        TransitionOutcome::Deferred(deferred)
            if deferred.capacity() == 8
                && deferred.dimension() == PhysicalWorkCapacityDimension::Commands
    ));
    assert_eq!(serving.media_counters(), before);

    let mut foreground = Vec::new();
    for _ in 0..4 {
        foreground.push(
            serving
                .reserve_physical_scheduler_foreground(dispatch_lane())
                .expect("four idle dispatch permits must admit"),
        );
    }
    let before = serving.media_counters();
    assert!(matches!(
        serving.reserve_physical_scheduler_foreground(dispatch_lane()),
        Err(RecordSchedulerReservationDenial::Admission(
            PhysicalInstanceForegroundAdmissionDenial::Foreground(
                ForegroundReservationAdmissionDenial::InsufficientCapacity(
                    ForegroundReservationResourceShortfall::WorkerPermit {
                        requested: 1,
                        available: 0,
                    },
                ),
            ),
        ))
    ));
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite),
        before.attempts_for(MediaOperationRole::PositionedWrite)
    );
    drop(foreground);

    let mut background = Vec::new();
    for _ in 0..2 {
        background.push(
            serving
                .reserve_physical_scheduler_background(dispatch_lane())
                .expect("two background dispatches fit beside the foreground floor"),
        );
    }
    let before = serving.media_counters();
    assert!(matches!(
        serving.reserve_physical_scheduler_background(dispatch_lane()),
        Err(RecordSchedulerReservationDenial::Admission(
            PhysicalInstanceForegroundAdmissionDenial::Foreground(
                ForegroundReservationAdmissionDenial::InsufficientCapacity(
                    ForegroundReservationResourceShortfall::WorkerPermit { requested: 1, .. },
                ),
            ),
        ))
    ));
    let mut protected = Vec::new();
    for _ in 0..2 {
        protected.push(
            serving
                .reserve_physical_scheduler_foreground(dispatch_lane())
                .expect("two dispatch permits stay available to foreground"),
        );
    }
    assert!(matches!(
        serving.reserve_physical_scheduler_foreground(dispatch_lane()),
        Err(RecordSchedulerReservationDenial::Admission(
            PhysicalInstanceForegroundAdmissionDenial::Foreground(
                ForegroundReservationAdmissionDenial::InsufficientCapacity(
                    ForegroundReservationResourceShortfall::WorkerPermit {
                        requested: 1,
                        available: 0,
                    },
                ),
            ),
        ))
    ));
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite),
        before.attempts_for(MediaOperationRole::PositionedWrite)
    );
    drop(protected);
    drop(background);
    serving.close();
}

pub(super) struct HeldReadySlots<'a> {
    serving: &'a ServingPhysicalRuntime,
    receipts: Vec<worth_store::physical_runtime::PhysicalWorkSubmissionReceipt>,
}

impl HeldReadySlots<'_> {
    pub(super) fn release_one(&mut self) {
        let Some(receipt) = self.receipts.pop() else {
            return;
        };
        let admitted = self
            .serving
            .admit_physical_work(receipt)
            .expect("an admitted ready read can be cancelled");
        match self.serving.request_physical_work(admitted).unwrap() {
            PhysicalWorkReadiness::Ready(ready) => {
                self.serving
                    .cancel_physical_work(ready.consumer_handle())
                    .expect("cancelling a ready read releases its slot");
                drop(ready);
            }
            PhysicalWorkReadiness::Blocked(_) => panic!("ready read blocked before dispatch"),
        }
    }
}

impl Drop for HeldReadySlots<'_> {
    fn drop(&mut self) {
        for receipt in self.receipts.drain(..) {
            let admitted = self
                .serving
                .admit_physical_work(receipt)
                .expect("an admitted ready read can be cancelled");
            match self.serving.request_physical_work(admitted).unwrap() {
                PhysicalWorkReadiness::Ready(ready) => {
                    self.serving
                        .cancel_physical_work(ready.consumer_handle())
                        .expect("cancelling a ready read releases its slot");
                    drop(ready);
                }
                PhysicalWorkReadiness::Blocked(_) => panic!("ready read blocked before dispatch"),
            }
        }
    }
}

/// Holds every ready slot left beside `in_flight` commands already occupying
/// the queue, and proves the next submission is deferred at capacity eight.
pub(super) fn hold_ready_slots(
    serving: &ServingPhysicalRuntime,
    request: PhysicalReadWorkRequest,
    in_flight: usize,
) -> HeldReadySlots<'_> {
    let configured = serving.physical_scheduler_capacity().configured();
    assert_eq!(configured.queue_slots(), 8);
    assert_eq!(configured.worker_permits(), 4);
    let mut held = HeldReadySlots {
        serving,
        receipts: Vec::new(),
    };
    for _ in in_flight..8 {
        held.admit(request.clone());
    }
    held.assert_full(request);
    held
}

impl HeldReadySlots<'_> {
    fn admit(&mut self, request: PhysicalReadWorkRequest) {
        match self
            .serving
            .physical_read_submission()
            .submit(request)
            .into_raw()
        {
            TransitionOutcome::Success(receipt) => self.receipts.push(receipt),
            other => panic!("a free ready slot must admit, saw {other:?}"),
        }
    }

    fn assert_full(&self, request: PhysicalReadWorkRequest) {
        let before = self.serving.media_counters();
        assert!(matches!(
            self.serving.physical_read_submission().submit(request).into_raw(),
            TransitionOutcome::Deferred(deferred)
                if deferred.capacity() == 8
                    && deferred.dimension() == PhysicalWorkCapacityDimension::Commands
        ));
        assert_eq!(self.serving.media_counters(), before);
    }
}

pub(super) fn dispatch_lane() -> ForegroundLaneDeclaration {
    ForegroundLaneDeclaration::ordinary_page_write()
        .with_latency_envelope(ForegroundLatencyEnvelope::bounded_interference(
            "canonical-dispatch",
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

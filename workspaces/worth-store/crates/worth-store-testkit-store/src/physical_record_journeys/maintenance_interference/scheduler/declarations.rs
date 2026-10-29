pub use {worth_proof::TransitionOutcome};
pub use {worth_store::physical_runtime::PhysicalReadWorkRequest, worth_store::physical_runtime::PhysicalWorkCapacity, worth_store::physical_runtime::PhysicalWorkCapacityDimension, worth_store::physical_runtime::PhysicalWorkReadiness, worth_store::physical_runtime::RecordSchedulerReservationDenial, worth_store::physical_runtime::ServingPhysicalRuntime};
pub use {worth_store_io_scheduler::foreground_reservation::BandwidthToken, worth_store_io_scheduler::foreground_reservation::DirtyPageBudget, worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration, worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope, worth_store_io_scheduler::foreground_reservation::ForegroundReservationAdmissionDenial, worth_store_io_scheduler::foreground_reservation::ForegroundReservationResourceShortfall, worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget, worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundAdmissionDenial, worth_store_io_scheduler::foreground_reservation::QueueSlot, worth_store_io_scheduler::foreground_reservation::WorkerPermit, worth_store_io_scheduler::foreground_reservation::WriteBackWindow};
pub use {worth_store_physical_backend::MediaOperationRole};

pub use {crate::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile, crate::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture};

pub struct HeldReadySlots<'a> {
    pub serving: &'a ServingPhysicalRuntime,
    pub receipts: Vec<worth_store::physical_runtime::PhysicalWorkSubmissionReceipt>,
}

impl HeldReadySlots<'_> {
    pub fn release_one(&mut self) {
        let Some(receipt) = self.receipts.pop() else {
            return;
        };
        let admitted = self
            .serving
            .admit_physical_work(receipt)
            .expect("an admitted ready read can be cancelled");
        match self.serving.request_physical_work(admitted).unwrap() {
            worth_store::physical_runtime::PhysicalWorkReadiness::Ready(ready) => {
                self.serving
                    .cancel_physical_work(ready.consumer_handle())
                    .expect("cancelling a ready read releases its slot");
                drop(ready);
            }
            worth_store::physical_runtime::PhysicalWorkReadiness::Blocked(_) => panic!("ready read blocked before dispatch"),
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
                worth_store::physical_runtime::PhysicalWorkReadiness::Ready(ready) => {
                    self.serving
                        .cancel_physical_work(ready.consumer_handle())
                        .expect("cancelling a ready read releases its slot");
                    drop(ready);
                }
                worth_store::physical_runtime::PhysicalWorkReadiness::Blocked(_) => panic!("ready read blocked before dispatch"),
            }
        }
    }
}

/// Holds every ready slot left beside `in_flight` commands already occupying
/// the queue, and proves the next submission is deferred at capacity eight.
pub fn hold_ready_slots(
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
    pub fn admit(&mut self, request: PhysicalReadWorkRequest) {
        match self
            .serving
            .physical_read_submission()
            .submit(request)
            .into_raw()
        {
            worth_proof::TransitionOutcome::Success(receipt) => self.receipts.push(receipt),
            other => panic!("a free ready slot must admit, saw {other:?}"),
        }
    }

    pub fn assert_full(&self, request: PhysicalReadWorkRequest) {
        let before = self.serving.media_counters();
        assert!(matches!(
            self.serving.physical_read_submission().submit(request).into_raw(),
            worth_proof::TransitionOutcome::Deferred(deferred)
                if deferred.capacity() == 8
                    && deferred.dimension() == worth_store::physical_runtime::PhysicalWorkCapacityDimension::Commands
        ));
        assert_eq!(self.serving.media_counters(), before);
    }
}

pub fn dispatch_lane() -> ForegroundLaneDeclaration {
    worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration::ordinary_page_write()
        .with_latency_envelope(worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope::bounded_interference(
            "canonical-dispatch",
            2,
        ))
        .with_budget(
            worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget::new()
                .with_queue_slots(worth_store_io_scheduler::foreground_reservation::QueueSlot::new(1).unwrap())
                .with_bandwidth(worth_store_io_scheduler::foreground_reservation::BandwidthToken::bytes(4_096).unwrap())
                .with_write_back(worth_store_io_scheduler::foreground_reservation::WriteBackWindow::pages(1).unwrap())
                .with_dirty_pages(worth_store_io_scheduler::foreground_reservation::DirtyPageBudget::pages(1).unwrap())
                .with_worker_permits(worth_store_io_scheduler::foreground_reservation::WorkerPermit::new(1).unwrap()),
        )
}
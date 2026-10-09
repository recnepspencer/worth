use super::{background_head::BackgroundHeadKind, PhysicalSchedulerAdmissionOwner};
use worth_store_io_scheduler::foreground_reservation::*;
use worth_store_io_scheduler::*;

/// One bounded SourceCopy destination frame admitted as blob migration work.
pub(in crate::physical_runtime) struct BlobMovementFrameAdmission {
    bytes: u64,
    pub(in crate::physical_runtime) lease: BackgroundIdleCapacityLease,
    pub(in crate::physical_runtime) capacity: PhysicalInstanceForegroundCapacityLease,
    pub(in crate::physical_runtime) backend: IoSchedulerBackendCapabilityAdmission,
    pub(in crate::physical_runtime) policy: worth_foundational::FoundationalPolicyAdmissionReceipt,
}

impl BlobMovementFrameAdmission {
    pub(in crate::physical_runtime) fn require_length(
        self,
        bytes: u64,
    ) -> Result<Self, crate::physical_runtime::PhysicalSchedulerDenial> {
        if self.bytes != bytes {
            return Err(
                crate::physical_runtime::PhysicalSchedulerDenial::BlobMovementAdmissionMismatch,
            );
        }
        Ok(self)
    }
}

impl PhysicalSchedulerAdmissionOwner {
    pub(in crate::physical_runtime) fn blob_movement_background(
        &self,
        security: &IoSchedulerSecurityScopeAdmission,
        bytes: u64,
        foreground_pressure_events: u64,
    ) -> Result<BlobMovementFrameAdmission, crate::physical_runtime::PhysicalSchedulerDenial> {
        use crate::physical_runtime::PhysicalSchedulerDenial as Denial;
        if bytes == 0 {
            return Err(Denial::BlobMovementAdmissionMismatch);
        }
        let kind = BackgroundHeadKind::BlobMovement;
        self.heads.note(kind, &self.dispatch);
        let held = super::foreground_budget::write_budget(bytes, true, true);
        let lane = ForegroundLaneDeclaration::ordinary_page_write()
            .with_latency_envelope(ForegroundLatencyEnvelope::bounded_interference(
                "physical-blob-movement",
                1,
            ))
            .with_budget(held);
        let reservation = self
            .foreground
            .reserve_background(lane, &self.buffered_file, security)
            .map_err(Denial::BackgroundCapacity)?;
        let (receipt, capacity) = reservation.into_parts();
        let budget = BackgroundResourceBudget::new()
            .with_queue_slots(QueueSlot::new(1).expect("one frame"))
            .with_worker_permits(WorkerPermit::new(1).expect("one frame"))
            .with_bandwidth(BandwidthToken::bytes(bytes).expect("nonempty frame"))
            .with_write_back(WriteBackWindow::pages(1).expect("one written frame"))
            .with_dirty_pages(DirtyPageBudget::pages(1).expect("one dirty frame"))
            .with_flush_permits(FlushPermit::new(1).expect("one preservation permit"))
            .with_sync_debt(SyncDebt::units(1).expect("one preservation debt"));
        let policy =
            crate::physical_runtime::record_serving::admit_blob_movement_background_policy(budget);
        let secure_io = admit_secure_io_scope_for_scheduler(SecureIoPreservationRequest::new(
            SecureIoOperation::BackgroundLease,
            security,
            &self.buffered_file,
        ))
        .map_err(|cause| Denial::BackgroundPacing(BackgroundPacingDenial::SecureIoDenied(cause)))?;
        let admission = match admit_background_capacity(
            BackgroundCapacityAdmissionRequest::new(
                BackgroundIoPressureShape::buffered_file_blob_migration_pressure()
                    .requesting(budget),
                &receipt,
                &self.buffered_file,
                policy.clone(),
            )
            .with_idle_available(super::capacity::background_idle_for_held_quantum(
                self.capacity_snapshot().available(),
                held,
            ))
            .with_policy_admitted(budget)
            .with_debt_limit(budget)
            .with_secure_io_scope(secure_io),
        ) {
            Ok(admission) => admission,
            Err(denial) => {
                if !matches!(denial, BackgroundPacingDenial::InsufficientIdleCapacity(_)) {
                    self.heads.cancel(kind, &self.dispatch);
                }
                return Err(Denial::BackgroundPacing(denial));
            }
        };
        let pacing = admit_background_pacing(
            BackgroundIdleCapacityLeaseRequest::new(admission)
                .with_foreground_pressure_events(foreground_pressure_events),
        );
        self.heads.settle(kind, &self.dispatch, &pacing);
        match pacing {
            BackgroundPacingOutcome::AdmittedWithDebt(admitted) => Ok(BlobMovementFrameAdmission {
                bytes,
                lease: admitted.into_lease(),
                capacity,
                backend: self.buffered_file,
                policy,
            }),
            _ => Err(Denial::OwedBackgroundTurn),
        }
    }

    pub(in crate::physical_runtime) fn cancel_blob_movement_background_head(&self) {
        self.heads
            .cancel(BackgroundHeadKind::BlobMovement, &self.dispatch);
    }
}

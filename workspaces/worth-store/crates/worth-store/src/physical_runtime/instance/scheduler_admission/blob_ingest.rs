use super::PhysicalSchedulerAdmissionOwner;
use worth_store_io_scheduler::foreground_reservation::*;
use worth_store_io_scheduler::*;

/// A single bounded chunk write's Store-owned background pressure admission.
pub(in crate::physical_runtime) struct BlobIngestFrameAdmission {
    bytes: u64,
    pub(in crate::physical_runtime) lease: BackgroundIdleCapacityLease,
    pub(in crate::physical_runtime) capacity: PhysicalInstanceForegroundCapacityLease,
    pub(in crate::physical_runtime) backend: IoSchedulerBackendCapabilityAdmission,
    pub(in crate::physical_runtime) policy: worth_foundational::FoundationalPolicyAdmissionReceipt,
}

impl BlobIngestFrameAdmission {
    pub(in crate::physical_runtime) fn require_length(
        self,
        bytes: u64,
    ) -> Result<Self, crate::physical_runtime::PhysicalSchedulerDenial> {
        if self.bytes != bytes {
            return Err(
                crate::physical_runtime::PhysicalSchedulerDenial::BlobIngestAdmissionMismatch,
            );
        }
        Ok(self)
    }
}

impl PhysicalSchedulerAdmissionOwner {
    /// Admit one chunk's buffered write and durability footprint. Each
    /// synchronous attempt owns its own fairness head: a denied attempt has
    /// no retry owner, while concurrent blob producers remain independently
    /// visible to foreground selection.
    pub(in crate::physical_runtime) fn blob_ingest_background(
        &self,
        security: &IoSchedulerSecurityScopeAdmission,
        bytes: u64,
        foreground_pressure_events: u64,
    ) -> Result<BlobIngestFrameAdmission, crate::physical_runtime::PhysicalSchedulerDenial> {
        use crate::physical_runtime::PhysicalSchedulerDenial as Denial;
        if bytes == 0 {
            return Err(Denial::BlobIngestAdmissionMismatch);
        }
        let dispatch_attempt = self.dispatch.begin_background();
        let held = super::foreground_budget::write_budget(bytes, true, true);
        let lane = ForegroundLaneDeclaration::ordinary_page_write()
            .with_latency_envelope(ForegroundLatencyEnvelope::bounded_interference(
                "physical-blob-ingest",
                1,
            ))
            .with_budget(held);
        let reservation = self
            .foreground
            .reserve_background(lane, &self.buffered_file, security)
            .map_err(Denial::BackgroundCapacity)?;
        let (receipt, capacity) = reservation.into_parts();
        let budget = BackgroundResourceBudget::new()
            .with_queue_slots(QueueSlot::new(1).expect("one chunk"))
            .with_worker_permits(WorkerPermit::new(1).expect("one chunk"))
            .with_bandwidth(BandwidthToken::bytes(bytes).expect("nonempty chunk"))
            .with_write_back(WriteBackWindow::pages(1).expect("one written chunk"))
            .with_dirty_pages(DirtyPageBudget::pages(1).expect("one dirty chunk"))
            .with_flush_permits(FlushPermit::new(1).expect("one durability barrier"))
            .with_sync_debt(SyncDebt::units(1).expect("one durability debt"));
        let policy =
            crate::physical_runtime::record_serving::admit_blob_ingest_background_policy(budget);
        let secure_io = admit_secure_io_scope_for_scheduler(SecureIoPreservationRequest::new(
            SecureIoOperation::BackgroundLease,
            security,
            &self.buffered_file,
        ))
        .map_err(|cause| Denial::BackgroundPacing(BackgroundPacingDenial::SecureIoDenied(cause)))?;
        let admission = admit_background_capacity(
            BackgroundCapacityAdmissionRequest::new(
                BackgroundIoPressureShape::buffered_file_blob_ingest_pressure().requesting(budget),
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
        )
        .map_err(Denial::BackgroundPacing)?;
        let pacing = admit_background_pacing(
            BackgroundIdleCapacityLeaseRequest::new(admission)
                .with_foreground_pressure_events(foreground_pressure_events),
        );
        match pacing {
            BackgroundPacingOutcome::AdmittedWithDebt(admitted) => {
                dispatch_attempt.commit();
                Ok(BlobIngestFrameAdmission {
                    bytes,
                    lease: admitted.into_lease(),
                    capacity,
                    backend: self.buffered_file,
                    policy,
                })
            }
            _ => Err(Denial::OwedBackgroundTurn),
        }
    }
}

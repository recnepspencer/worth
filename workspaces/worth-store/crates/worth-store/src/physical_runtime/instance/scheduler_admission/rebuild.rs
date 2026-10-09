use std::num::NonZeroU64;

use super::PhysicalSchedulerAdmissionOwner;
use crate::physical_runtime::{record_serving::RebuildReadShape, PhysicalSchedulerDenial};
use worth_store_io_scheduler::foreground_reservation::*;
use worth_store_io_scheduler::*;

/// One selected Rebuild read. The lease and capacity remain owned until the
/// corresponding ready work is lowered, or are released on admission failure.
pub(in crate::physical_runtime) struct RebuildReadAdmission {
    pub(in crate::physical_runtime) shape: RebuildReadShape,
    pub(in crate::physical_runtime) bytes: Option<NonZeroU64>,
    pub(in crate::physical_runtime) lease: BackgroundIdleCapacityLease,
    pub(in crate::physical_runtime) capacity: PhysicalInstanceForegroundCapacityLease,
    pub(in crate::physical_runtime) backend: IoSchedulerBackendCapabilityAdmission,
    pub(in crate::physical_runtime) policy: worth_foundational::FoundationalPolicyAdmissionReceipt,
}

impl PhysicalSchedulerAdmissionOwner {
    /// Admit one bounded selected-root Rebuild read against shared instance
    /// capacity. Metadata has a queue/worker unit but no invented byte demand.
    pub(in crate::physical_runtime) fn rebuild_read(
        &self,
        shape: RebuildReadShape,
        security: &IoSchedulerSecurityScopeAdmission,
        bytes: Option<NonZeroU64>,
        foreground_pressure_events: u64,
    ) -> Result<RebuildReadAdmission, PhysicalSchedulerDenial> {
        let dispatch_attempt = self.dispatch.begin_background();
        let held = match bytes {
            Some(bytes) => super::read_budget(bytes.get()),
            None => super::metadata_budget(),
        };
        let lane = match bytes {
            Some(_) => ForegroundLaneDeclaration::buffered_file_internal_foreground_read()
                .expect("Store-owned buffered Rebuild read"),
            None => ForegroundLaneDeclaration::artifact_metadata_read(),
        }
        .with_latency_envelope(ForegroundLatencyEnvelope::bounded_interference(
            "physical-rebuild-read",
            1,
        ))
        .with_budget(held);
        let reservation = self
            .foreground
            .reserve_background(lane, &self.buffered_file, security)
            .map_err(PhysicalSchedulerDenial::BackgroundCapacity)?;
        let (receipt, capacity) = reservation.into_parts();
        let mut budget = BackgroundResourceBudget::new()
            .with_queue_slots(QueueSlot::new(1).expect("one Rebuild read"))
            .with_worker_permits(WorkerPermit::new(1).expect("one Rebuild read"));
        if let Some(bytes) = bytes {
            budget = budget
                .with_bandwidth(BandwidthToken::bytes(bytes.get()).expect("nonzero Rebuild range"));
        }
        let policy =
            crate::physical_runtime::record_serving::admit_rebuild_background_policy(budget);
        let secure_io = admit_secure_io_scope_for_scheduler(SecureIoPreservationRequest::new(
            SecureIoOperation::RepairScan,
            security,
            &self.buffered_file,
        ))
        .map_err(|cause| {
            PhysicalSchedulerDenial::BackgroundPacing(BackgroundPacingDenial::SecureIoDenied(cause))
        })?;
        let admission = admit_background_capacity(
            BackgroundCapacityAdmissionRequest::new(
                BackgroundIoPressureShape::buffered_file_repair_scan().requesting(budget),
                &receipt,
                &self.buffered_file,
                policy.clone(),
            )
            .with_idle_available(super::capacity::background_idle_for_held_quantum(
                self.capacity_snapshot().available(),
                held,
            ))
            .with_policy_admitted(budget)
            .with_secure_io_scope(secure_io),
        )
        .map_err(PhysicalSchedulerDenial::BackgroundPacing)?;
        let pacing = admit_background_pacing(
            BackgroundIdleCapacityLeaseRequest::new(admission)
                .with_foreground_pressure_events(foreground_pressure_events),
        );
        match pacing {
            BackgroundPacingOutcome::AdmittedWithDebt(admitted) => {
                dispatch_attempt.commit();
                Ok(RebuildReadAdmission {
                    shape,
                    bytes,
                    lease: admitted.into_lease(),
                    capacity,
                    backend: self.buffered_file,
                    policy,
                })
            }
            _ => Err(PhysicalSchedulerDenial::OwedBackgroundTurn),
        }
    }
}

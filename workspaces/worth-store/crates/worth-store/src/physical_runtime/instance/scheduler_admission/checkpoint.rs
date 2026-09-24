use super::PhysicalSchedulerAdmissionOwner;
use worth_store_io_scheduler::IoSchedulerBackendCapabilityAdmission;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum PhysicalCheckpointSchedulerAdmissionDenial {
    Foreground(
        worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundAdmissionDenial,
    ),
    OwedBackgroundTurn,
    Background(worth_store_io_scheduler::BackgroundPacingDenial),
}

impl PhysicalSchedulerAdmissionOwner {
    pub(in crate::physical_runtime) fn checkpoint_background(
        &self,
        security: &worth_store_io_scheduler::IoSchedulerSecurityScopeAdmission,
        bytes: u64,
        foreground_pressure_events: u64,
    ) -> Result<
        (
            worth_store_io_scheduler::BackgroundPacingOutcome,
            IoSchedulerBackendCapabilityAdmission,
            worth_foundational::FoundationalPolicyAdmissionReceipt,
            worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundCapacityLease,
        ),
        PhysicalCheckpointSchedulerAdmissionDenial,
    >{
        let preservation = worth_store_io_scheduler::foreground_reservation::
            ForegroundLaneDeclaration::filesystem_admitted_wal_barrier()
            .expect("filesystem-admitted checkpoint preservation is a Store-owned lane")
            .with_latency_envelope(
                worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope::
                    bounded_interference("checkpoint-foreground-preservation", 2),
            )
            .with_budget(super::capacity::foreground_quantum_budget(bytes));
        let kind = super::background_head::BackgroundHeadKind::Checkpoint;
        let already_retained = self.heads.is_retained(kind);
        let preservation =
            match super::background_head::RetainedBackgroundHeads::reserve_preservation(
                self,
                kind,
                preservation,
                security,
            ) {
                Ok(reservation) => reservation,
                Err(denial) => {
                    if super::capacity::reservation_shortage_retains_background_head(&denial) {
                        self.heads.note(kind, &self.dispatch);
                    } else if already_retained {
                        self.heads.cancel(kind, &self.dispatch);
                    }
                    return Err(match denial {
                        super::RecordSchedulerReservationDenial::Admission(cause) => {
                            PhysicalCheckpointSchedulerAdmissionDenial::Foreground(cause)
                        }
                        super::RecordSchedulerReservationDenial::OwedBackgroundTurn => {
                            PhysicalCheckpointSchedulerAdmissionDenial::OwedBackgroundTurn
                        }
                    });
                }
            };
        let (foreground_receipt, foreground_capacity) = preservation.into_parts();
        self.heads.note(kind, &self.dispatch);

        let budget = checkpoint_background_budget(bytes);
        let idle = super::capacity::background_idle_for_held_quantum(
            self.capacity_snapshot().available(),
            super::capacity::foreground_quantum_budget(bytes),
        );
        let pressure = worth_store_io_scheduler::BackgroundIoPressureShape::
            filesystem_admitted_checkpoint_flush()
            .requesting(budget);
        let policy =
            crate::physical_runtime::record_serving::admit_checkpoint_background_policy(budget);
        let capacity = match worth_store_io_scheduler::admit_background_capacity(
            worth_store_io_scheduler::BackgroundCapacityAdmissionRequest::new(
                pressure,
                &foreground_receipt,
                &self.fsync,
                policy.clone(),
            )
            .with_idle_available(idle)
            .with_policy_admitted(budget)
            .with_debt_limit(budget),
        ) {
            Ok(capacity) => capacity,
            Err(denial) => {
                drop(foreground_capacity);
                if !matches!(
                    denial,
                    worth_store_io_scheduler::BackgroundPacingDenial::InsufficientIdleCapacity(_)
                ) {
                    self.heads.cancel(kind, &self.dispatch);
                }
                return Err(PhysicalCheckpointSchedulerAdmissionDenial::Background(
                    denial,
                ));
            }
        };
        let pacing = worth_store_io_scheduler::admit_background_pacing(
            worth_store_io_scheduler::BackgroundIdleCapacityLeaseRequest::new(capacity)
                .with_foreground_pressure_events(foreground_pressure_events),
        );
        self.heads.settle(
            super::background_head::BackgroundHeadKind::Checkpoint,
            &self.dispatch,
            &pacing,
        );
        Ok((pacing, self.fsync, policy, foreground_capacity))
    }
}

fn checkpoint_background_budget(bytes: u64) -> worth_store_io_scheduler::BackgroundResourceBudget {
    use worth_store_io_scheduler::{
        BandwidthToken, FlushPermit, QueueSlot, SyncDebt, WorkerPermit,
    };
    worth_store_io_scheduler::BackgroundResourceBudget::new()
        .with_queue_slots(QueueSlot::new(1).expect("one checkpoint action is nonzero"))
        .with_bandwidth(
            BandwidthToken::bytes(bytes).expect("checkpoint action accounting is nonzero"),
        )
        .with_flush_permits(FlushPermit::new(1).expect("one checkpoint flush is nonzero"))
        .with_sync_debt(SyncDebt::units(1).expect("one checkpoint sync debt is nonzero"))
        .with_worker_permits(WorkerPermit::new(1).expect("one checkpoint worker is nonzero"))
}

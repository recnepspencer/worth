pub(super) fn foreground_capacity(
    capacity: crate::physical_runtime::PhysicalWorkCapacity,
) -> worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget {
    use worth_store_io_scheduler::{
        BandwidthToken, CacheResidencyHint, DirtyPageBudget, FlushPermit, QueueSlot,
        ReadAheadWindow, ReclaimPermit, SyncDebt, WorkerPermit, WriteBackWindow,
    };
    let commands = u64::try_from(capacity.commands()).expect("usize fits the scheduler counter");
    let bytes =
        u64::try_from(capacity.total_semantic_bytes()).expect("usize fits the scheduler counter");
    worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget::new()
        .with_queue_slots(QueueSlot::new(commands).expect("work capacity is nonzero"))
        .with_bandwidth(BandwidthToken::bytes(bytes).expect("semantic capacity is nonzero"))
        .with_flush_permits(FlushPermit::new(commands).expect("work capacity is nonzero"))
        .with_sync_debt(SyncDebt::units(commands).expect("work capacity is nonzero"))
        .with_read_ahead(ReadAheadWindow::pages(commands).expect("work capacity is nonzero"))
        .with_write_back(WriteBackWindow::pages(commands).expect("work capacity is nonzero"))
        .with_dirty_pages(DirtyPageBudget::pages(commands).expect("work capacity is nonzero"))
        .with_worker_permits(WorkerPermit::new(commands).expect("work capacity is nonzero"))
        .with_cache_residency(
            CacheResidencyHint::frames(commands).expect("work capacity is nonzero"),
        )
        .with_reclaim_permits(ReclaimPermit::new(commands).expect("work capacity is nonzero"))
}

pub(super) fn foreground_quantum_budget(
    bytes: u64,
) -> worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget {
    use worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget;
    use worth_store_io_scheduler::{BandwidthToken, FlushPermit, QueueSlot, SyncDebt, WorkerPermit};
    ForegroundResourceBudget::new()
        .with_queue_slots(QueueSlot::new(1).expect("one background quantum is nonzero"))
        .with_bandwidth(
            BandwidthToken::bytes(bytes.max(1)).expect("a background quantum accounts bytes"),
        )
        .with_flush_permits(FlushPermit::new(1).expect("one background flush is nonzero"))
        .with_sync_debt(SyncDebt::units(1).expect("one background sync is nonzero"))
        .with_worker_permits(WorkerPermit::new(1).expect("one background worker is nonzero"))
}

pub(super) fn reclamation_quantum_budget(
    bytes: u64,
) -> worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget {
    use worth_store_io_scheduler::ReclaimPermit;
    foreground_quantum_budget(bytes)
        .with_reclaim_permits(ReclaimPermit::new(1).expect("one reclaim permit is nonzero"))
}

pub(super) fn background_idle_for_held_quantum(
    available_after_reservation: worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget,
    held: worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget,
) -> worth_store_io_scheduler::BackgroundResourceBudget {
    use worth_store_io_scheduler::{
        BackgroundResourceBudget, BandwidthToken, FlushPermit, QueueSlot, ReclaimPermit, SyncDebt,
        WorkerPermit,
    };
    let mut idle = BackgroundResourceBudget::new();
    if let Ok(unit) = QueueSlot::new(
        available_after_reservation
            .queue_slots()
            .saturating_add(held.queue_slots()),
    ) {
        idle = idle.with_queue_slots(unit);
    }
    if let Ok(unit) = BandwidthToken::bytes(
        available_after_reservation
            .bandwidth_tokens()
            .saturating_add(held.bandwidth_tokens()),
    ) {
        idle = idle.with_bandwidth(unit);
    }
    if let Ok(unit) = FlushPermit::new(
        available_after_reservation
            .flush_permits()
            .saturating_add(held.flush_permits()),
    ) {
        idle = idle.with_flush_permits(unit);
    }
    if let Ok(unit) = SyncDebt::units(
        available_after_reservation
            .sync_debt()
            .saturating_add(held.sync_debt()),
    ) {
        idle = idle.with_sync_debt(unit);
    }
    if let Ok(unit) = WorkerPermit::new(
        available_after_reservation
            .worker_permits()
            .saturating_add(held.worker_permits()),
    ) {
        idle = idle.with_worker_permits(unit);
    }
    if let Ok(unit) = ReclaimPermit::new(
        available_after_reservation
            .reclaim_permits()
            .saturating_add(held.reclaim_permits()),
    ) {
        idle = idle.with_reclaim_permits(unit);
    }
    idle
}

const QUEUE_PROGRESS_HEADROOM: u64 = 1;

/// Ordinary foreground growth must leave one queue slot for checkpoint or retirement.
///
/// A profile with only that slot cannot withhold it; the caller still reserves.
pub(super) fn deny_queue_headroom(
    snapshot: worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundCapacitySnapshot,
    requested: worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget,
) -> Result<(), super::RecordSchedulerReservationDenial> {
    let configured = snapshot.configured().queue_slots();
    if configured <= QUEUE_PROGRESS_HEADROOM {
        return Ok(());
    }
    let usable = snapshot
        .available()
        .queue_slots()
        .saturating_sub(QUEUE_PROGRESS_HEADROOM);
    if requested.queue_slots() <= usable {
        return Ok(());
    }
    Err(super::RecordSchedulerReservationDenial::Admission(
        worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundAdmissionDenial::Foreground(
            worth_store_io_scheduler::foreground_reservation::ForegroundReservationAdmissionDenial::InsufficientCapacity(
                worth_store_io_scheduler::foreground_reservation::ForegroundReservationResourceShortfall::QueueSlot {
                    requested: requested.queue_slots(),
                    available: usable,
                },
            ),
        ),
    ))
}

pub(super) fn reservation_shortage_retains_background_head(
    denial: &super::RecordSchedulerReservationDenial,
) -> bool {
    use worth_store_io_scheduler::foreground_reservation::{
        ForegroundReservationAdmissionDenial, PhysicalInstanceForegroundAdmissionDenial,
    };
    match denial {
        super::RecordSchedulerReservationDenial::OwedBackgroundTurn => true,
        super::RecordSchedulerReservationDenial::Admission(
            PhysicalInstanceForegroundAdmissionDenial::Foreground(
                ForegroundReservationAdmissionDenial::InsufficientCapacity(_),
            ),
        ) => true,
        super::RecordSchedulerReservationDenial::Admission(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use worth_store_io_scheduler::foreground_reservation::{
        ForegroundResourceBudget, PhysicalInstanceForegroundCapacity, QueueSlot,
    };

    fn slots(count: u64) -> ForegroundResourceBudget {
        ForegroundResourceBudget::new().with_queue_slots(QueueSlot::new(count).unwrap())
    }

    #[test]
    fn ordinary_growth_leaves_one_queue_slot() {
        let capacity = PhysicalInstanceForegroundCapacity::new(slots(2)).unwrap();
        assert!(super::deny_queue_headroom(capacity.snapshot(), slots(1)).is_ok());
        assert!(super::deny_queue_headroom(capacity.snapshot(), slots(2)).is_err());
        let single = PhysicalInstanceForegroundCapacity::new(slots(1)).unwrap();
        assert!(super::deny_queue_headroom(single.snapshot(), slots(1)).is_ok());
    }
}
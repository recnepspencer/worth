use worth_store_io_scheduler::foreground_reservation::ReservationViolatedWithCause;
use worth_store_tiering::ColdPlacementState;

use super::BlobPlacementMovementCounterSnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlobPlacementMovementDenial {
    StaleMovementPlan {
        counters: BlobPlacementMovementCounterSnapshot,
    },
    MissingMovementReadPlan {
        counters: BlobPlacementMovementCounterSnapshot,
    },
    LifecycleSourcePlacementBasisMismatch {
        counters: BlobPlacementMovementCounterSnapshot,
    },
    LifecycleTargetPlacementBasisMismatch {
        counters: BlobPlacementMovementCounterSnapshot,
    },
    ForegroundReservationViolated {
        violation: Box<ReservationViolatedWithCause>,
        counters: BlobPlacementMovementCounterSnapshot,
    },
    ForegroundReservationScopeMismatch {
        counters: BlobPlacementMovementCounterSnapshot,
    },
    ColdPlacementUnavailable {
        state: ColdPlacementState,
        counters: BlobPlacementMovementCounterSnapshot,
    },
}

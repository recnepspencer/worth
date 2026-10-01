use crate::placement::movement::{
    counters::BlobPlacementMovementCounterSnapshot,
    denial::BlobPlacementMovementDenial,
    types::{BlobPlacementMovementForegroundReservation, BlobPlacementMovementRequest},
    verification::{
        cold_permits_movement::require_cold_permits_movement,
        foreground_reservation_scope::require_foreground_reservation_scope,
        lifecycle_placement_basis::{
            require_source_lifecycle_placement_basis, require_target_lifecycle_placement_basis,
        },
        movement_freshness::require_current_freshness,
        read_hold_present::require_read_hold_present,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MovementEligibilityCase {
    Admit,
    Stale,
    MissingReadPlan,
    ForegroundViolated,
    ForegroundScopeMismatch,
    ColdUnavailable,
    LifecycleSourceBasisMismatch,
    LifecycleTargetBasisMismatch,
}

pub(crate) fn classify_movement_eligibility(
    request: &BlobPlacementMovementRequest,
) -> MovementEligibilityCase {
    if let Some(case) = require_current_freshness(request) {
        return case;
    }
    if let Some(case) = require_read_hold_present(request) {
        return case;
    }
    if let Some(case) = require_foreground_reservation_scope(request) {
        return case;
    }
    if let Some(case) = require_cold_permits_movement(request) {
        return case;
    }
    if let Some(case) = require_source_lifecycle_placement_basis(request) {
        return case;
    }
    if let Some(case) = require_target_lifecycle_placement_basis(request) {
        return case;
    }
    MovementEligibilityCase::Admit
}

pub(crate) fn assemble_movement_denial(
    case: MovementEligibilityCase,
    request: &BlobPlacementMovementRequest,
    counters: BlobPlacementMovementCounterSnapshot,
) -> BlobPlacementMovementDenial {
    match case {
        MovementEligibilityCase::Admit => {
            unreachable!("admit cases do not assemble denials")
        }
        MovementEligibilityCase::Stale => {
            BlobPlacementMovementDenial::StaleMovementPlan { counters }
        }
        MovementEligibilityCase::MissingReadPlan => {
            BlobPlacementMovementDenial::MissingMovementReadPlan {
                counters: counters.record_protected_denial(),
            }
        }
        MovementEligibilityCase::ForegroundViolated => {
            let BlobPlacementMovementForegroundReservation::Violated(violation) =
                request.foreground_reservation()
            else {
                unreachable!("classifier ensures violated");
            };
            BlobPlacementMovementDenial::ForegroundReservationViolated {
                violation: Box::new(violation),
                counters: counters.record_tier_move_retry().record_protected_denial(),
            }
        }
        MovementEligibilityCase::ForegroundScopeMismatch => {
            BlobPlacementMovementDenial::ForegroundReservationScopeMismatch {
                counters: counters.record_protected_denial(),
            }
        }
        MovementEligibilityCase::ColdUnavailable => {
            BlobPlacementMovementDenial::ColdPlacementUnavailable {
                state: request.cold_outcome().state(),
                counters: counters
                    .record_unavailable_cold_chunk()
                    .record_tier_move_retry()
                    .record_protected_denial(),
            }
        }
        MovementEligibilityCase::LifecycleSourceBasisMismatch => {
            BlobPlacementMovementDenial::LifecycleSourcePlacementBasisMismatch {
                counters: counters.record_protected_denial(),
            }
        }
        MovementEligibilityCase::LifecycleTargetBasisMismatch => {
            BlobPlacementMovementDenial::LifecycleTargetPlacementBasisMismatch {
                counters: counters.record_protected_denial(),
            }
        }
    }
}

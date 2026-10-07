use crate::placement::movement::{
    classification::MovementEligibilityCase, types::BlobPlacementMovementRequest,
};

pub(crate) fn require_read_hold_present(
    request: &BlobPlacementMovementRequest,
) -> Option<MovementEligibilityCase> {
    if request.read_plan().is_some() {
        None
    } else {
        Some(MovementEligibilityCase::MissingReadPlan)
    }
}

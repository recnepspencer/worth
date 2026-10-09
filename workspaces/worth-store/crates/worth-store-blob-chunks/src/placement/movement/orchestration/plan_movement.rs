use crate::placement::movement::{
    denial::BlobPlacementMovementDenial,
    transitions::admit_movement_plan::transition_admit_movement_plan,
    types::{
        AdmittedBlobPlacementMovementPlan, BlobPlacementMovementAuthority,
        BlobPlacementMovementRequest,
    },
};

impl BlobPlacementMovementAuthority {
    pub fn plan_movement(
        &self,
        request: BlobPlacementMovementRequest,
    ) -> Result<AdmittedBlobPlacementMovementPlan, BlobPlacementMovementDenial> {
        transition_admit_movement_plan(request)
    }
}

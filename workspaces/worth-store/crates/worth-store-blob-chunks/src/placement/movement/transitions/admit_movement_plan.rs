use crate::placement::movement::{
    classification::{
        assemble_movement_denial, classify_movement_eligibility, MovementEligibilityCase,
    },
    counters::BlobPlacementMovementCounterSnapshot,
    denial::BlobPlacementMovementDenial,
    receipt_construction::movement_plan::construct_movement_plan,
    types::{AdmittedBlobPlacementMovementPlan, BlobPlacementMovementRequest},
};

pub(crate) fn transition_admit_movement_plan(
    request: BlobPlacementMovementRequest,
) -> Result<AdmittedBlobPlacementMovementPlan, BlobPlacementMovementDenial> {
    let counters = BlobPlacementMovementCounterSnapshot::start(
        request.source().class(),
        request.target().class(),
    );
    let case = classify_movement_eligibility(&request);
    if case != MovementEligibilityCase::Admit {
        return Err(assemble_movement_denial(case, &request, counters));
    }
    let read_plan = request
        .read_plan()
        .expect("classifier ensures movement read plan is present");
    Ok(construct_movement_plan(request, read_plan, counters))
}

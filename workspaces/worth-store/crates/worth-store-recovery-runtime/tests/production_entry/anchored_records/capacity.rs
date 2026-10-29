//! Thirty-two anchors retain identity through a real maintenance publication.
use super::*;
use worth_store::physical_runtime::{
    PhysicalMutationIdempotencyMaterial, PhysicalMutationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest,
};
pub(super) fn rewrite(serving: &ServingPhysicalRuntime, placement: AdmittedRecordPlacementPolicy) {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([0x52; 32]))
        .unwrap();
    let result = submission
        .rewrite_selected_inline_segment(
            placement,
            PhysicalMutationRequest::platform_durable(
                key,
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            ),
        )
        .into_raw();
    let TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) = result
    else {
        panic!("maintenance must prepare");
    };
    assert!(matches!(
        prepared.execute(),
        PhysicalMutationOutcome::Completed(_)
    ));
}

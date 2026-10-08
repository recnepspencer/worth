use super::*;
use worth_query_declaration::facade::application_operation::ApplicationCandidateResourceCeiling;
use worth_query_declaration::facade::domain_computation::{
    WorthQueryCancellationSafePointFamily, WorthQueryResourceDimension,
    WorthQueryResourceLimitRequest, WorthQuerySemanticScaleAxis, WorthQuerySemanticScaleRequest,
};
#[test]
fn atomic_batch_width_does_not_enlarge_actual_candidate_reservation() {
    let envelope = worth_query_installation::facade::WorthQueryExecutionResourceEnvelope::atomic(
        WorthQuerySemanticScaleRequest::selective()
            .with(WorthQuerySemanticScaleAxis::CandidateItems, 1)
            .with(WorthQuerySemanticScaleAxis::BatchWidth, 4_096),
        WorthQueryResourceLimitRequest::selective().with(
            WorthQueryResourceDimension::CandidateRetainedRepresentationBytes,
            64,
        ),
        WorthQueryCancellationSafePointFamily::new("attempt").unwrap(),
    );
    let requirements = |creates| {
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(creates, 0, 0, 0, 0, 0),
            ApplicationCandidateResourceCeiling::representation_bytes(64),
        )
    };
    let reserve = |requested| {
        WorthQueryCandidateReservation::admit(
            requested,
            requirements(2),
            envelope.scale_ceiling(WorthQuerySemanticScaleAxis::CandidateItems),
            envelope.resource_ceiling(
                WorthQueryResourceDimension::CandidateRetainedRepresentationBytes,
            ),
            None,
        )
    };
    let mut reservation = reserve(requirements(1)).unwrap();
    assert_eq!(reservation.total_items(), 1);
    reservation.charge(CandidateItemKind::Create).unwrap();
    assert_eq!(
        reservation
            .charge(CandidateItemKind::Create)
            .unwrap_err()
            .kind(),
        WorthQueryApplicationAttemptDenialKind::CandidateReservationExceeded
    );
    let refusal = match reserve(requirements(2)) {
        Err(denial) => denial,
        Ok(_) => panic!("large declaration batch must not enlarge candidate capacity"),
    };
    assert_eq!(
        refusal.kind(),
        WorthQueryApplicationAttemptDenialKind::CandidateCapacityExceeded
    );
}

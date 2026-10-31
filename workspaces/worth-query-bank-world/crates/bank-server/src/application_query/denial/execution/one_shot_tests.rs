use super::{one_shot, BankApplicationOneShotDenialKind as Bank, QueryOneShot};
use worth_query_host::facade::runtime::ExecutionPhysicalAllocationDenialKind as Physical;

#[test]
fn one_shot_preserves_distinct_projection_inspection_denials() {
    let unavailable = one_shot(QueryOneShot::ProjectionInspectionUnavailable);
    let incomplete = one_shot(QueryOneShot::ProjectionInspectionIncomplete);
    assert_eq!(unavailable, Bank::ProjectionInspectionUnavailable);
    assert_eq!(incomplete, Bank::ProjectionInspectionIncomplete);
    assert_ne!(unavailable, incomplete);
}

#[test]
fn one_shot_preserves_the_original_source_observation_allocation_payload() {
    for (kind, requested_payload_bytes) in [
        (Physical::Allocator, Some(4096)),
        (Physical::Layout, None),
        (Physical::CapacityMismatch, Some(0)),
        (Physical::WriteBeyondReserved, Some(u64::MAX)),
        (Physical::IncompleteSeal, Some(64)),
    ] {
        assert_eq!(
            one_shot(QueryOneShot::SourceObservationAllocation {
                kind,
                requested_payload_bytes,
            }),
            Bank::SourceObservationAllocation {
                kind,
                requested_payload_bytes,
            }
        );
    }
}

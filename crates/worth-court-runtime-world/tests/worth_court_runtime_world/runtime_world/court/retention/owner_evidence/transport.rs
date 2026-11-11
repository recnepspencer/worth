//! Actual prepaid destinations and the public normalized-checkpoint path.
use super::*;
#[test]
fn every_owner_envelope_fills_its_original_prepaid_occurrence_destination() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    for tag in [1, 2, 3] {
        let journey = Journey::new(&request, tag);
        assert_eq!(journey.occurrence, journey.expected);
        journey.fixture.finish(&request);
    }
}
#[test]
fn normalized_checkpoint_refuses_a_retired_world_row_dialect() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let journey = Journey::new(&request, 1);
    let checkpoint = journey.checkpoint(&request);
    let mut selected = checkpoint.bytes().to_vec();
    assert_eq!(&selected[8..12], &3u32.to_le_bytes());
    assert_eq!(&selected[52..56], &5u32.to_le_bytes());
    selected[52..56].copy_from_slice(&4u32.to_le_bytes());
    rehash(&mut selected);
    let denial = journey.refusal(&request, Some(selected), |_| {});
    assert!(matches!(
        denial,
        WorldReadmissionDenial::Medium(WorldDurableMediumDenial::Frame(
            WorldDurableFrameDenial::UnsupportedVersion(_)
        ))
    ));
    drop(checkpoint);
    journey.fixture.finish(&request);
}

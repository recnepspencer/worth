//! A receipt serves only its paid passes, and supplies its own records.
use super::super::refreshed_rejoin::{charge_whole_map, WholeMapPasses};
use super::*;

#[test]
fn whole_map_receipt_spends_one_or_two_passes_before_refusing_without_contact() {
    let mut state = DemandRegistryState::default();
    let older = key("receipt", 1, 1);
    let newer = key("receipt", 2, 1);
    for selected in [&older, &newer] {
        state.records.insert(
            selected.clone(),
            record(occurrence(), DemandState::Admitted, 0),
        );
    }
    for (passes, expected_work) in [(WholeMapPasses::One, 2), (WholeMapPasses::Two, 4)] {
        let mut admission = record_admission();
        let mut receipt = charge_whole_map(&state.records, &mut admission, passes, 0).unwrap();
        assert_eq!(receipt.newest(&older).unwrap(), Some(newer.clone()));
        if expected_work == 4 {
            assert!(!receipt.stale_owner(&newer).unwrap());
        }
        let before = super::super::record_map::occurrence_rows_visited();
        let denial = receipt
            .newest(&older)
            .expect_err("a spent pass establishes nothing");
        assert_eq!(
            denial.kind(),
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
        );
        assert_eq!(super::super::record_map::occurrence_rows_visited(), before);
        // Two records, charged once or twice; consuming a pass adds no work.
        assert_eq!(admission.charged_work(), expected_work);
    }
}

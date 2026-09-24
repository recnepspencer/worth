#[test]
fn visual_settlement_wakes_ordinary_product_progress_without_bypassing_blockers() {
    assert!(
        super::product_turn_admitted_after_visual_readiness(true, false, false, false, false),
        "visual retirement releases the receipt and wakes ordinary product progress"
    );
    assert!(!super::product_turn_admitted_after_visual_readiness(
        false, false, false, false, false
    ));
    assert!(!super::product_turn_admitted_after_visual_readiness(
        true, true, false, false, false
    ));
    assert!(!super::product_turn_admitted_after_visual_readiness(
        true, false, true, false, false
    ));
    assert!(!super::product_turn_admitted_after_visual_readiness(
        true, false, false, true, false
    ));
    assert!(
        !super::product_turn_admitted_after_visual_readiness(true, false, false, false, true),
        "successor capture and comparison retain the receipt and cannot wake product early"
    );
}

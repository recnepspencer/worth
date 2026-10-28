//! Symbol closure identity follows a logical cell across physical promotion.
use super::*;

#[test]
fn head_symbol_identity_survives_full_promotion_without_covering_history() {
    let fixture = region_with_history();
    let head = head_demand(&fixture.cell, 1 << 24).unwrap();
    let before = head.symbol_admission_identity();
    let full = full_demand(&fixture.cell, 1 << 24).unwrap();
    assert_eq!(before, full.head_symbol_admission_identity());
    assert_ne!(before, full.symbol_admission_identity());
    assert_eq!(
        before,
        full.symbol_admission_identity().covered_head().unwrap()
    );
    assert_eq!(
        super::super::required_symbols::collect_head(&head.resident_region().partition),
        super::super::required_symbols::collect_head(full.full_view().partition()),
        "promotion preserves the entire current dictionary closure",
    );
}

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
        super::super::required_symbols::collect_head(
            &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
            &head.resident_region().partition
        ),
        super::super::required_symbols::collect_head(
            &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
            full.full_view().partition()
        ),
        "promotion preserves the entire current dictionary closure",
    );
}

#[test]
fn metadata_only_freezes_forward_coverage_without_an_intermediate_cell_chain() {
    let fixture = region_with_history();
    let original = full_demand(&fixture.cell, 1 << 24).unwrap();
    let identity = original.symbol_admission_identity();
    let first = original.into_edit().freeze();
    let second = first.into_edit().freeze();
    let inheritance = second.cell().symbol_inheritance.get().unwrap();
    assert_eq!(inheritance.predecessor(true).unwrap().0, identity);
    let cold = super::super::required_symbols::collect(
        &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
        second.full_view().partition(),
    );
    let before = super::super::required_symbols::collect(
        &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
        &full_demand(&fixture.cell, 1 << 24)
            .unwrap()
            .resident_region()
            .partition,
    );
    assert_eq!(cold, before);
}

use {worth_store_layout_indexes::access_shapes, worth_store_layout_indexes::AccessLaneClassification, worth_store_layout_indexes::FullDeclaredScanBasis, worth_store_layout_indexes::FullDeclaredScanView};

#[test]
fn foreground_full_scan_is_denied_by_the_production_shape_owner() {
    let outcome = access_shapes().full_declared_scan(
        worth_store_layout_indexes::AccessLaneClassification::Foreground,
        worth_store_layout_indexes::FullDeclaredScanBasis::DeclaredFullTraversal,
    );

    assert!(matches!(
        outcome.view(),
        worth_store_layout_indexes::FullDeclaredScanView::HiddenDenied(_)
    ));
}

#[test]
fn verifier_full_scan_is_explicitly_admitted_by_the_production_shape_owner() {
    let outcome = access_shapes().full_declared_scan(
        worth_store_layout_indexes::AccessLaneClassification::Verifier,
        worth_store_layout_indexes::FullDeclaredScanBasis::DeclaredFullTraversal,
    );

    assert!(matches!(outcome.view(), worth_store_layout_indexes::FullDeclaredScanView::Success(_)));
}

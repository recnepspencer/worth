use super::*;

#[test]
fn replacement_binding_refreshes_only_coordinate_ownership() {
    let baseline = fixture();
    let affected = baseline.binding;
    let replacement = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let row = complete(baseline.clone());
    let rebound = with(&baseline, |input| {
        input.binding = replacement;
        input.allocation_basis =
            UiMountedAllocationBasis::new(1, 2, 99, UiMountedTransformProjection::Identity);
    });
    assert!(row.same_retained_paint_meaning_after_binding_replacement(
        &complete(rebound.clone()),
        affected,
        replacement,
    ));

    let variants = [
        with(&rebound, |input| {
            input.allocation_basis =
                UiMountedAllocationBasis::new(8, 2, 99, UiMountedTransformProjection::Identity)
        }),
        with(&rebound, |input| {
            input.allocation_basis =
                UiMountedAllocationBasis::new(1, 8, 99, UiMountedTransformProjection::Identity)
        }),
        with(&rebound, |input| {
            input.allocation_basis = UiMountedAllocationBasis::new(
                1,
                2,
                99,
                UiMountedTransformProjection::Omitted(
                    crate::UiMountedOmissionReason::AllocationBoundsUnknown,
                ),
            )
        }),
        with(&rebound, |input| {
            input.bounds = canonical_box(31.0, 32.0, 160.0, 96.0)
        }),
        with(&rebound, |input| {
            input.clip_bounds = canonical_box(32.0, 32.0, 159.0, 96.0)
        }),
        with(&rebound, |input| {
            input.layout = inert_layout_with_identity("ONLINE", 8)
        }),
        with(&rebound, |input| set_text(input, Arc::from("UPDATED"))),
        with(&rebound, |input| {
            input.foregrounds = Arc::from([UiMountedTextForegroundSpan::from_runtime_mounting(
                crate::UiTextOriginalRange::from_text_mechanics(0, 6).unwrap(),
                UiMountedRgba8::new(254, 255, 255, 255),
                UiMountedTextPaintSpanIdentity::from_runtime_mounting([7; 32]),
            )])
        }),
    ];
    for variant in variants {
        assert!(!row.same_retained_paint_meaning_after_binding_replacement(
            &complete(variant),
            affected,
            replacement,
        ));
    }
}

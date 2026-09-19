use super::*;
use worth_ui_host_contract::{NormalizedPoint, UiMountedLinearGradient, UiMountedSurfaceFill};

#[test]
fn gradient_translation_preserves_physical_axis_and_half_open_bounds() {
    let ids = fixture_ids();
    let frame_id = frame_identity();
    let issuer = UiMountedNodeReceiptIssuer::mint_for(frame_id).unwrap();
    let fill = UiMountedSurfaceFill::LinearGradient(
        UiMountedLinearGradient::new(
            NormalizedPoint::new(0, 0).unwrap(),
            NormalizedPoint::new(10_000, 10_000).unwrap(),
            [[0, 0, 0, 255], [255, 255, 255, 255]]
                .map(UiMountedAppearanceColor::from_straight_srgba),
        )
        .unwrap(),
    );
    let mechanic = surface(
        issuer,
        ids.base,
        UiAppearanceAllocationBounds::new(20, 10, 200, 100).unwrap(),
        fill,
    );
    let order = UiMountedOverlayOrderMechanic::complete_from_runtime_overlay_order(
        ids.surface,
        ids.presentation,
        7,
        11,
        [],
    )
    .unwrap();
    let frame = UiMountedAppearanceFrame::from_runtime_mounting(
        frame_id,
        ids.surface,
        [UiMountedAppearanceMechanic::Surface(mechanic)],
        order,
    )
    .unwrap();
    let work = initial_work(&frame, damage(20, 10, 200, 100));
    let translated = super::super::super::super::translate_appearance_fragment_work(&work).unwrap();
    let observed = translated.successor();
    // Physical direction (200,100): (20,60) projects to .1 and (120,60)
    // to .5. Independent sRGB encodings are 89 and 188, respectively.
    for (point, rgba) in [
        ([20, 60], [89, 89, 89, 255]),
        ([120, 60], [188, 188, 188, 255]),
        ([220, 60], [0, 0, 0, 0]),
    ] {
        assert_eq!(
            observed
                .reference_surface_at(ids.base, point[0], point[1])
                .unwrap()
                .straight_srgba(),
            rgba
        );
    }
    assert!(observed.mechanics()[0].matches_mounted(&frame.mechanics()[0]));
    assert_eq!(translated.damage(), &[damage(20, 10, 200, 100)]);
}

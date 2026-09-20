use super::geometry::UiNativeAppearanceScale;
use super::mounted_mechanic_fixtures::scroll_chrome;
use super::scroll_chrome_pipeline::UiNativeScrollChromePipeline;
use super::surface_pipeline::UiNativeSurfacePipeline;
use worth_ui_host_contract::UiMountedScrollChromePart;

/// The prepared primitive scales the rectangle the runtime already snapped and
/// keeps every field a host paints from.
#[test]
fn scroll_chrome_pipeline_preserves_identity_rect_clip_colour_radii_and_opacity() {
    let mechanic = scroll_chrome(UiMountedScrollChromePart::Thumb);
    let primitive = UiNativeScrollChromePipeline::prepare(
        &mechanic,
        UiNativeAppearanceScale::qualified(1_000).unwrap(),
    )
    .unwrap();
    assert_eq!(primitive.identity(), mechanic.identity());
    assert_eq!(primitive.semantic_surface(), mechanic.semantic_surface());
    assert_eq!(primitive.paint_ordinal(), 1);
    assert_eq!(primitive.background().straight_srgba(), [90, 90, 90, 255]);
    assert_eq!(primitive.opacity(), u16::MAX);
    assert!(primitive.radii().iter().all(|corner| *corner > 0));
    let bounds = primitive.rect().pixel_bounds();
    assert_eq!([bounds.left, bounds.top], [100, 20]);
    assert_eq!([bounds.right, bounds.bottom], [106, 68]);
}

/// The track is prepared with the same pipeline and paints under the thumb.
#[test]
fn scroll_chrome_track_paints_before_the_thumb() {
    let scale = UiNativeAppearanceScale::qualified(1_000).unwrap();
    let track = UiNativeScrollChromePipeline::prepare(
        &scroll_chrome(UiMountedScrollChromePart::Track),
        scale,
    )
    .unwrap();
    let thumb = UiNativeScrollChromePipeline::prepare(
        &scroll_chrome(UiMountedScrollChromePart::Thumb),
        scale,
    )
    .unwrap();
    assert!(track.paint_ordinal() < thumb.paint_ordinal());
}

/// Chrome reuses the authored surface fill path, so its rounded rectangle is
/// rasterized by the same coverage the surfaces use rather than by a second
/// rounding implementation.
#[test]
fn scroll_chrome_rasterizes_through_the_surface_fill_path() {
    let mechanic = scroll_chrome(UiMountedScrollChromePart::Thumb);
    let primitive = UiNativeScrollChromePipeline::prepare(
        &mechanic,
        UiNativeAppearanceScale::qualified(1_000).unwrap(),
    )
    .unwrap();
    let surface = UiNativeSurfacePipeline::prepare_scroll_chrome(&primitive);
    let operation = surface.raster_operation([256, 256]).unwrap();
    assert!(operation.is_some(), "a visible thumb rasterizes a fill");
}

/// A pixel outside the clip is not covered even when the rectangle reaches it.
#[test]
fn scroll_chrome_paints_only_inside_its_region_clip() {
    let mechanic = scroll_chrome(UiMountedScrollChromePart::Thumb);
    let primitive = UiNativeScrollChromePipeline::prepare(
        &mechanic,
        UiNativeAppearanceScale::qualified(1_000).unwrap(),
    )
    .unwrap();
    assert!(primitive.paints_in_order(102, 40));
    assert!(!primitive.paints_in_order(40, 40));
}

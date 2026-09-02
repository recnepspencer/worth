use super::backdrop_pipeline::UiNativeBackdropPipeline;
use super::geometry::UiNativeAppearanceScale;
use super::mounted_mechanic_fixtures::backdrop;

#[test]
fn backdrop_pipeline_preserves_identity_extent_clip_color_and_opacity() {
    let mechanic = backdrop(4);
    let primitive = UiNativeBackdropPipeline::prepare(
        &mechanic,
        UiNativeAppearanceScale::qualified(1_250).unwrap(),
    )
    .unwrap();
    assert_eq!(primitive.identity(), mechanic.identity());
    assert_eq!(primitive.ordinal(), 4);
    assert_eq!(primitive.background().straight_srgba(), [0, 0, 0, 128]);
    assert_eq!(primitive.opacity(), u16::MAX);
    assert!(primitive.paints_in_order(0, 0));
    assert_eq!(primitive.damage_rect().left, 0);
}

use super::geometry::UiNativeAppearanceScale;
use super::mounted_mechanic_fixtures::{
    allocation, logical_length, mounted_outline, MountedOutlineFixtureInput,
};
use super::outline_pipeline::UiNativeOutlinePipeline;
use worth_ui_host_contract::{
    UiAppearanceClip, UiMountedAppearanceColor, UiMountedAppearanceOpacity,
};

#[test]
fn outline_damage_keeps_full_fringe_while_parent_clip_only_removes_pixels() {
    for (milli, expected_left, expected_right, ring_pixel, expected_coverage) in [
        (1_000, -3, 13, -2, u16::MAX),
        (1_250, -4, 17, -2, 49_151),
        (1_500, -5, 20, -3, u16::MAX),
        (2_000, -6, 26, -4, u16::MAX),
    ] {
        let primitive = UiNativeOutlinePipeline::prepare(
            &mounted_outline(MountedOutlineFixtureInput {
                allocation: allocation(0, 0, 10_000, 10_000),
                clip: UiAppearanceClip::new(-3_000, -3_000, 16_000, 16_000).unwrap(),
                radii: [logical_length(0); 4],
                line_width: logical_length(1_000),
                offset: logical_length(1_000),
                anti_alias_fringe: logical_length(1_000),
                color: UiMountedAppearanceColor::from_straight_srgba([8, 16, 32, 255]),
                opacity: UiMountedAppearanceOpacity::ONE,
            }),
            UiNativeAppearanceScale::qualified(milli).unwrap(),
        )
        .unwrap();
        assert!(primitive.extends_beyond_allocation());
        assert_eq!(primitive.damage_rect().left, expected_left);
        assert_eq!(primitive.damage_rect().right, expected_right);
        assert_eq!(
            primitive.sample(ring_pixel, 5).coverage.units(),
            expected_coverage
        );
        assert_eq!(primitive.sample(ring_pixel, 5).opacity, u16::MAX);
    }
}

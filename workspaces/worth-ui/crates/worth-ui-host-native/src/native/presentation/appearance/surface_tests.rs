use super::antialiasing::UiNativeAnalyticCoverage;
use super::geometry::UiNativeAppearanceScale;
use super::mounted_mechanic_fixtures::{
    allocation, logical_length, mounted_surface, mounted_surface_with_edges,
    MountedSurfaceFixtureInput,
};
use super::surface_pipeline::UiNativeSurfacePipeline;
use worth_ui_host_contract::{
    UiAppearanceClip, UiMountedAppearanceColor, UiMountedPresentationOpacity,
    UiMountedSurfaceBorderEdges, UiMountedSurfacePaint,
};

#[test]
fn seam_loser_omits_only_the_shared_border_edge_at_all_qualified_scales() {
    let edges = UiMountedSurfaceBorderEdges::from_runtime_mosaic(true, true, true, false);
    for scale in UiNativeAppearanceScale::qualified_set() {
        let primitive = UiNativeSurfacePipeline::prepare(
            &mounted_surface_with_edges(
                MountedSurfaceFixtureInput {
                    allocation: allocation(0, 0, 10_000, 10_000),
                    clip: UiAppearanceClip::new(0, 0, 10_000, 10_000).unwrap(),
                    radii: [logical_length(0); 4],
                    paint: UiMountedSurfacePaint::Border {
                        color: UiMountedAppearanceColor::from_straight_srgba([20, 40, 60, 255]),
                        inward_width: logical_length(1_000),
                    },
                    opacity: UiMountedPresentationOpacity::from_runtime_composition(u16::MAX),
                },
                edges,
            ),
            scale,
        )
        .unwrap();
        let size = primitive.visual_bounds().width();
        assert_eq!(
            primitive.sample(0, size / 2).border_coverage,
            UiNativeAnalyticCoverage::ZERO,
        );
        assert_eq!(
            primitive.sample(size / 2, 0).border_coverage,
            UiNativeAnalyticCoverage::ONE,
        );
    }
}

#[test]
fn rounded_surface_keeps_fill_and_inward_border_separate_at_all_qualified_scales() {
    let paint = UiMountedSurfacePaint::FillAndBorder {
        fill: UiMountedAppearanceColor::from_straight_srgba([10, 20, 30, 255]),
        border: UiMountedAppearanceColor::from_straight_srgba([200, 100, 50, 255]),
        inward_width: logical_length(1_000),
    };
    for (scale, expected_size) in UiNativeAppearanceScale::qualified_set().zip([10, 13, 15, 20]) {
        let primitive = UiNativeSurfacePipeline::prepare(
            &mounted_surface(MountedSurfaceFixtureInput {
                allocation: allocation(0, 0, 10_000, 10_000),
                clip: UiAppearanceClip::new(0, 0, 10_000, 10_000).unwrap(),
                radii: [logical_length(2_000); 4],
                paint: paint.clone(),
                opacity: UiMountedPresentationOpacity::from_runtime_composition(u16::MAX),
            }),
            scale,
        )
        .unwrap();
        let center = primitive.sample(5, 5);
        assert_eq!(center.fill_coverage, UiNativeAnalyticCoverage::ONE);
        assert_eq!(center.border_coverage, UiNativeAnalyticCoverage::ZERO);
        assert_eq!(
            primitive.sample(0, 5).border_coverage,
            UiNativeAnalyticCoverage::ONE
        );
        assert!(primitive.has_inward_border());
        assert_eq!(primitive.visual_bounds().width(), expected_size);
        assert_eq!(primitive.visual_bounds().height(), expected_size);
        assert_eq!(primitive.damage_rect().right, expected_size);
        assert_eq!(primitive.damage_rect().bottom, expected_size);
    }
}

#[test]
fn rounded_surface_paints_the_curved_border_between_allocation_edge_strips() {
    let primitive = UiNativeSurfacePipeline::prepare(
        &mounted_surface(MountedSurfaceFixtureInput {
            allocation: allocation(0, 0, 64_000, 64_000),
            clip: UiAppearanceClip::new(0, 0, 64_000, 64_000).unwrap(),
            radii: [logical_length(24_000); 4],
            paint: UiMountedSurfacePaint::FillAndBorder {
                fill: UiMountedAppearanceColor::from_straight_srgba([10, 20, 30, 255]),
                border: UiMountedAppearanceColor::from_straight_srgba([200, 100, 50, 255]),
                inward_width: logical_length(2_000),
            },
            opacity: UiMountedPresentationOpacity::from_runtime_composition(u16::MAX),
        }),
        UiNativeAppearanceScale::qualified(1_000).unwrap(),
    )
    .unwrap();

    assert!(
        primitive.sample(7, 7).border_coverage != UiNativeAnalyticCoverage::ZERO,
        "the circular corner stroke must continue beyond the rectangular edge strips"
    );
}

#[test]
fn analytic_box_reference_has_exact_inside_and_outside_limits() {
    let primitive = UiNativeSurfacePipeline::prepare(
        &mounted_surface(MountedSurfaceFixtureInput {
            allocation: allocation(0, 0, 20_000, 20_000),
            clip: UiAppearanceClip::new(0, 0, 20_000, 20_000).unwrap(),
            radii: [logical_length(0); 4],
            paint: UiMountedSurfacePaint::Fill(UiMountedAppearanceColor::from_straight_srgba([
                1, 2, 3, 255,
            ])),
            opacity: UiMountedPresentationOpacity::from_runtime_composition(u16::MAX),
        }),
        UiNativeAppearanceScale::qualified(1_500).unwrap(),
    )
    .unwrap();
    assert_eq!(
        primitive.sample(0, 0).fill_coverage,
        UiNativeAnalyticCoverage::ONE
    );
    assert_eq!(
        primitive.sample(-1, 0).fill_coverage,
        UiNativeAnalyticCoverage::ZERO
    );
    assert_eq!(
        primitive.sample(0, 0).border_coverage,
        UiNativeAnalyticCoverage::ZERO
    );
    assert_eq!(
        primitive.sample(0, 0).opacity,
        u16::MAX,
        "coverage and opacity remain separate fixed-point factors"
    );
}

#[test]
fn border_endpoints_are_total_at_all_qualified_scales() {
    let color = UiMountedAppearanceColor::from_straight_srgba([20, 40, 60, 255]);
    for scale in UiNativeAppearanceScale::qualified_set() {
        for paint in [
            UiMountedSurfacePaint::Border {
                color,
                inward_width: logical_length(0),
            },
            UiMountedSurfacePaint::FillAndBorder {
                fill: color,
                border: color,
                inward_width: logical_length(0),
            },
        ] {
            let primitive = UiNativeSurfacePipeline::prepare(
                &mounted_surface(MountedSurfaceFixtureInput {
                    allocation: allocation(0, 0, 10_000, 10_000),
                    clip: UiAppearanceClip::new(0, 0, 10_000, 10_000).unwrap(),
                    radii: [logical_length(0); 4],
                    paint,
                    opacity: UiMountedPresentationOpacity::from_runtime_composition(u16::MAX),
                }),
                scale,
            )
            .unwrap();
            assert_eq!(
                primitive.sample(5, 5).border_coverage,
                UiNativeAnalyticCoverage::ZERO
            );
        }
        for paint in [
            UiMountedSurfacePaint::Border {
                color,
                inward_width: logical_length(5_000),
            },
            UiMountedSurfacePaint::FillAndBorder {
                fill: color,
                border: color,
                inward_width: logical_length(5_000),
            },
        ] {
            let primitive = UiNativeSurfacePipeline::prepare(
                &mounted_surface(MountedSurfaceFixtureInput {
                    allocation: allocation(0, 0, 10_000, 10_000),
                    clip: UiAppearanceClip::new(0, 0, 10_000, 10_000).unwrap(),
                    radii: [logical_length(0); 4],
                    paint,
                    opacity: UiMountedPresentationOpacity::from_runtime_composition(u16::MAX),
                }),
                scale,
            )
            .unwrap();
            assert_eq!(
                primitive.sample(5, 5).border_coverage,
                UiNativeAnalyticCoverage::ONE
            );
        }
    }
}

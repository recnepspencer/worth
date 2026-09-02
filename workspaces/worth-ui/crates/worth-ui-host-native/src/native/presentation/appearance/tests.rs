use super::antialiasing::UiNativeAnalyticCoverage;
use super::command::{UiNativeAppearanceCommand, UiNativeAppearanceCommandFamily};
use super::damage::UiNativeAppearanceDamageRect;
use super::geometry::{UiNativeAppearanceScale, UiNativePhysicalRect};
use super::outline_pipeline::UiNativeOutlinePipeline;
use super::retained::UiNativeAppearanceRetained;
use super::surface_pipeline::UiNativeSurfacePipeline;
use worth_ui_host_contract::{
    UiAppearanceAllocationBounds, UiAppearanceClip, UiAppearanceLogicalLength,
    UiAppearanceNormalizedLogicalRadii, UiAppearanceOutlineGeometry, UiHostPointerIdentity,
    UiMountedAppearanceColor, UiMountedBackdropAppearanceAttribution,
    UiMountedBackdropCompletionInput, UiMountedBackdropIdentity, UiMountedBackdropMechanic,
    UiMountedBackdropScope, UiMountedFrameIdentity, UiMountedInstanceIdentity,
    UiMountedLayerProjection, UiMountedLayerReference, UiMountedNodeAppearanceAttribution,
    UiMountedNodeReceiptIssuer, UiMountedOutlineAppearanceCompletionInput,
    UiMountedOutlineAppearanceMechanic, UiMountedOverlayOrderMechanic,
    UiMountedPointerAffordanceMechanic, UiMountedPresentationAttemptIdentity,
    UiMountedSurfaceAppearanceCompletionInput, UiMountedSurfaceAppearanceMechanic,
    UiMountedSurfacePaint, UiMountedTextForegroundAppearanceCompletionInput,
    UiMountedTextForegroundAppearanceMechanic, UiMountedTextPaintSpanIdentity,
    UiOverlayParticipantIdentity, UiOverlayPlacementReceipt, UiPointerAffordanceFamily,
};

pub(super) fn length(value: i32) -> UiAppearanceLogicalLength {
    UiAppearanceLogicalLength::new(value).unwrap()
}

pub(super) fn bounds(x: i32, y: i32, width: u32, height: u32) -> UiAppearanceAllocationBounds {
    UiAppearanceAllocationBounds::new(x, y, width, height).unwrap()
}

pub(super) fn surface(
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    radius: i32,
    paint: UiMountedSurfacePaint,
) -> UiMountedSurfaceAppearanceMechanic {
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let issuer = UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
    let allocation = bounds(x, y, width, height);
    UiMountedSurfaceAppearanceMechanic::complete_from_runtime_mounting(
        UiMountedSurfaceAppearanceCompletionInput {
            issuer,
            node_receipt: issuer.receipt_for(UiMountedInstanceIdentity::mint_unbound().unwrap()),
            bounds: allocation,
            clip: UiAppearanceClip::new(x, y, width, height).unwrap(),
            layer: UiMountedLayerProjection::Layer(UiMountedLayerReference::new(0)),
            radii: UiAppearanceNormalizedLogicalRadii::normalize(allocation, [length(radius); 4]),
            paint,
            opacity: worth_ui_host_contract::UiMountedAppearanceOpacity::ONE,
            projection: UiMountedNodeAppearanceAttribution::from_runtime_mounting(issuer, 1, 1)
                .unwrap(),
        },
    )
    .unwrap()
}

fn outline_with_clip(
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    line_width: i32,
    offset: i32,
    fringe: i32,
    clip_x: i32,
    clip_y: i32,
    clip_width: u32,
    clip_height: u32,
) -> UiMountedOutlineAppearanceMechanic {
    let allocation = bounds(x, y, width, height);
    let radii = UiAppearanceNormalizedLogicalRadii::normalize(allocation, [length(0); 4]);
    let geometry = UiAppearanceOutlineGeometry::admit(
        allocation,
        radii,
        length(line_width),
        length(offset),
        length(fringe),
    )
    .unwrap();
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let issuer = UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
    UiMountedOutlineAppearanceMechanic::complete_from_runtime_mounting(
        UiMountedOutlineAppearanceCompletionInput {
            issuer,
            node_receipt: issuer.receipt_for(UiMountedInstanceIdentity::mint_unbound().unwrap()),
            clip: UiAppearanceClip::new(clip_x, clip_y, clip_width, clip_height).unwrap(),
            geometry,
            color: UiMountedAppearanceColor::from_straight_srgba([8, 16, 32, 255]),
            opacity: worth_ui_host_contract::UiMountedAppearanceOpacity::ONE,
            projection: UiMountedNodeAppearanceAttribution::from_runtime_mounting(issuer, 1, 1)
                .unwrap(),
        },
    )
    .unwrap()
}

pub(super) fn backdrop(ordinal: u32) -> UiMountedBackdropMechanic {
    let surface = worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    backdrop_on_surface(surface, ordinal)
}

fn backdrop_on_surface(
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ordinal: u32,
) -> UiMountedBackdropMechanic {
    let placement = UiOverlayPlacementReceipt::from_runtime_overlay_order(1, ordinal).unwrap();
    let identity = UiMountedBackdropIdentity::from_runtime_mounting(
        format!("test.backdrop.{ordinal}"),
        UiMountedBackdropScope::SurfaceSingleton(surface),
        u64::from(ordinal) + 1,
    )
    .unwrap();
    UiMountedBackdropMechanic::complete_from_runtime_mounting(UiMountedBackdropCompletionInput {
        identity,
        semantic_surface: surface,
        placement,
        extent: worth_ui_host_contract::UiAppearanceBackdropExtent::new(0, 0, 40, 40).unwrap(),
        clip: UiAppearanceClip::new(0, 0, 40, 40).unwrap(),
        background: UiMountedAppearanceColor::from_straight_srgba([0, 0, 0, 128]),
        opacity: worth_ui_host_contract::UiMountedAppearanceOpacity::ONE,
        attribution: UiMountedBackdropAppearanceAttribution::from_runtime_transport(
            surface,
            placement,
            u64::from(ordinal) + 10,
            1,
        )
        .unwrap(),
    })
    .unwrap()
}

pub(super) fn text_foreground(seed: u8) -> UiMountedTextForegroundAppearanceMechanic {
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let issuer = UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
    UiMountedTextForegroundAppearanceMechanic::complete_from_runtime_mounting(
        UiMountedTextForegroundAppearanceCompletionInput {
            issuer,
            paint_span: UiMountedTextPaintSpanIdentity::from_runtime_mounting([seed; 32]),
            foreground: UiMountedAppearanceColor::from_straight_srgba([255, 255, 255, 255]),
            opacity: worth_ui_host_contract::UiMountedAppearanceOpacity::ONE,
            projection: UiMountedNodeAppearanceAttribution::from_runtime_mounting(issuer, 1, 1)
                .unwrap(),
        },
    )
    .unwrap()
}

pub(super) fn pointer(family: UiPointerAffordanceFamily) -> UiMountedPointerAffordanceMechanic {
    UiMountedPointerAffordanceMechanic::complete_from_runtime_mounting(
        UiHostPointerIdentity::new(1),
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
        UiMountedInstanceIdentity::mint_unbound().unwrap(),
        family,
    )
}

#[test]
fn qualified_scales_preserve_exact_half_open_floor_and_ceil_edges() {
    let scale = UiNativeAppearanceScale::qualified(1_250).unwrap();
    let rect =
        UiNativePhysicalRect::from_allocation(bounds(-1_000, 2_000, 3_000, 4_000), scale).unwrap();
    assert_eq!(
        rect.pixel_bounds(),
        super::geometry::UiNativePhysicalPixelRect {
            left: -2,
            top: 2,
            right: 3,
            bottom: 8,
        }
    );
    assert!(UiNativeAppearanceScale::qualified(1_333).is_err());
}

#[test]
fn rounded_surface_keeps_fill_and_inward_border_separate_at_all_qualified_scales() {
    let paint = UiMountedSurfacePaint::FillAndBorder {
        fill: UiMountedAppearanceColor::from_straight_srgba([10, 20, 30, 255]),
        border: UiMountedAppearanceColor::from_straight_srgba([200, 100, 50, 255]),
        inward_width: length(1_000),
    };
    for (scale, expected_size) in UiNativeAppearanceScale::qualified_set().zip([10, 13, 15, 20]) {
        let primitive = UiNativeSurfacePipeline::prepare(
            &surface(0, 0, 10_000, 10_000, 2_000, paint.clone()),
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
fn analytic_box_reference_has_exact_inside_and_outside_limits() {
    let scale = UiNativeAppearanceScale::qualified(1_500).unwrap();
    let primitive = UiNativeSurfacePipeline::prepare(
        &surface(
            0,
            0,
            20_000,
            20_000,
            0,
            UiMountedSurfacePaint::Fill(UiMountedAppearanceColor::from_straight_srgba([
                1, 2, 3, 255,
            ])),
        ),
        scale,
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
fn outline_damage_keeps_full_fringe_while_parent_clip_only_removes_pixels() {
    for (milli, expected_left, expected_right, ring_pixel, expected_coverage) in [
        (1_000, -3, 13, -2, u16::MAX),
        (1_250, -4, 17, -2, 49_151),
        (1_500, -5, 20, -3, u16::MAX),
        (2_000, -6, 26, -4, u16::MAX),
    ] {
        let primitive = UiNativeOutlinePipeline::prepare(
            &outline_with_clip(
                0, 0, 10_000, 10_000, 1_000, 1_000, 1_000, -3_000, -3_000, 16_000, 16_000,
            ),
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

#[test]
fn retained_commands_preserve_backdrop_and_overlay_order_without_flattening() {
    let surface = worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let first = backdrop_on_surface(surface, 0);
    let second = backdrop_on_surface(surface, 1);
    let order = UiMountedOverlayOrderMechanic::complete_from_runtime_overlay_order(
        surface,
        UiMountedPresentationAttemptIdentity::mint_unbound().unwrap(),
        1,
        1,
        [
            UiOverlayParticipantIdentity::Backdrop(first.identity().clone()),
            UiOverlayParticipantIdentity::Backdrop(second.identity().clone()),
        ],
    )
    .unwrap();
    let scale = UiNativeAppearanceScale::qualified(1_000).unwrap();
    let first_key = {
        let mut retained = UiNativeAppearanceRetained::new(scale);
        let first_key = retained
            .insert(UiNativeAppearanceCommand::Backdrop(first.clone()), None)
            .unwrap();
        let order_key = retained
            .insert(
                UiNativeAppearanceCommand::OverlayOrder(order),
                Some(first_key),
            )
            .unwrap();
        let second_key = retained
            .insert(
                UiNativeAppearanceCommand::Backdrop(second.clone()),
                Some(order_key),
            )
            .unwrap();
        assert_eq!(
            retained.ordered_keys().as_ref(),
            &[first_key, order_key, second_key]
        );
        assert_eq!(
            retained.command(order_key).unwrap().family(),
            UiNativeAppearanceCommandFamily::OverlayOrder
        );
        let replay = retained
            .replay_for_damage(UiNativeAppearanceDamageRect {
                left: 0,
                top: 0,
                right: 40,
                bottom: 40,
            })
            .unwrap();
        assert_eq!(replay.as_ref(), &[first_key, second_key]);
        let counters = retained.counters();
        assert_eq!(counters.full_scan_commands, 0);
        assert!(counters.damage_queries > 0);
        second_key
    };
    assert!(first_key.value() > 0);
}

#[test]
fn retained_spatial_replay_uses_the_bounded_index_not_a_draw_list_scan() {
    let scale = UiNativeAppearanceScale::qualified(1_000).unwrap();
    let mut retained = UiNativeAppearanceRetained::new(scale);
    let mut previous = None;
    for index in 0..64 {
        let x = index * 10_000;
        let key = retained
            .insert(
                UiNativeAppearanceCommand::Surface(surface(
                    x,
                    0,
                    4_000,
                    4_000,
                    0,
                    UiMountedSurfacePaint::Fill(UiMountedAppearanceColor::from_straight_srgba([
                        1, 2, 3, 255,
                    ])),
                )),
                previous,
            )
            .unwrap();
        previous = Some(key);
    }
    let replay = retained
        .replay_for_damage(UiNativeAppearanceDamageRect {
            left: 0,
            top: 0,
            right: 4,
            bottom: 4,
        })
        .unwrap();
    assert_eq!(replay.len(), 1);
    let counters = retained.counters();
    assert_eq!(counters.full_scan_commands, 0);
    assert!(counters.damage_leaf_probes < 64);
}

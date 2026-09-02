use worth_ui_host_contract::{
    UiAppearanceAllocationBounds, UiAppearanceBackdropExtent, UiAppearanceClip,
    UiAppearanceLogicalLength, UiAppearanceNormalizedLogicalRadii, UiAppearanceOutlineGeometry,
    UiHostPointerIdentity, UiMountedAppearanceColor, UiMountedAppearanceOpacity,
    UiMountedBackdropAppearanceAttribution, UiMountedBackdropCompletionInput,
    UiMountedBackdropIdentity, UiMountedBackdropMechanic, UiMountedBackdropScope,
    UiMountedFrameIdentity, UiMountedInstanceIdentity, UiMountedLayerProjection,
    UiMountedLayerReference, UiMountedNodeAppearanceAttribution, UiMountedNodeReceiptIssuer,
    UiMountedOutlineAppearanceCompletionInput, UiMountedOutlineAppearanceMechanic,
    UiMountedPointerAffordanceMechanic, UiMountedSurfaceAppearanceCompletionInput,
    UiMountedSurfaceAppearanceMechanic, UiMountedSurfacePaint,
    UiMountedTextForegroundAppearanceCompletionInput, UiMountedTextForegroundAppearanceMechanic,
    UiMountedTextPaintSpanIdentity, UiOverlayPlacementReceipt, UiPointerAffordanceFamily,
    UiSemanticSurfaceIdentity,
};

pub(super) fn logical_length(value: i32) -> UiAppearanceLogicalLength {
    UiAppearanceLogicalLength::new(value).unwrap()
}

pub(super) fn allocation(x: i32, y: i32, width: u32, height: u32) -> UiAppearanceAllocationBounds {
    UiAppearanceAllocationBounds::new(x, y, width, height).unwrap()
}

pub(super) struct MountedSurfaceFixtureInput {
    pub(super) allocation: UiAppearanceAllocationBounds,
    pub(super) clip: UiAppearanceClip,
    pub(super) radii: [UiAppearanceLogicalLength; 4],
    pub(super) paint: UiMountedSurfacePaint,
    pub(super) opacity: UiMountedAppearanceOpacity,
}

pub(super) fn mounted_surface(
    input: MountedSurfaceFixtureInput,
) -> UiMountedSurfaceAppearanceMechanic {
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let issuer = UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
    UiMountedSurfaceAppearanceMechanic::complete_from_runtime_mounting(
        UiMountedSurfaceAppearanceCompletionInput {
            issuer,
            node_receipt: issuer.receipt_for(UiMountedInstanceIdentity::mint_unbound().unwrap()),
            bounds: input.allocation,
            clip: input.clip,
            layer: UiMountedLayerProjection::Layer(UiMountedLayerReference::new(0)),
            radii: UiAppearanceNormalizedLogicalRadii::normalize(input.allocation, input.radii),
            paint: input.paint,
            opacity: input.opacity,
            projection: UiMountedNodeAppearanceAttribution::from_runtime_mounting(issuer, 1, 1)
                .unwrap(),
        },
    )
    .unwrap()
}

pub(super) struct FilledSurfaceFixtureInput {
    pub(super) allocation: UiAppearanceAllocationBounds,
    pub(super) color: UiMountedAppearanceColor,
}

pub(super) fn filled_surface(
    input: FilledSurfaceFixtureInput,
) -> UiMountedSurfaceAppearanceMechanic {
    let x = input.allocation.x();
    let y = input.allocation.y();
    let width = input.allocation.width();
    let height = input.allocation.height();
    mounted_surface(MountedSurfaceFixtureInput {
        allocation: input.allocation,
        clip: UiAppearanceClip::new(x, y, width, height).unwrap(),
        radii: [UiAppearanceLogicalLength::ZERO; 4],
        paint: UiMountedSurfacePaint::Fill(input.color),
        opacity: UiMountedAppearanceOpacity::ONE,
    })
}

pub(super) struct MountedOutlineFixtureInput {
    pub(super) allocation: UiAppearanceAllocationBounds,
    pub(super) clip: UiAppearanceClip,
    pub(super) radii: [UiAppearanceLogicalLength; 4],
    pub(super) line_width: UiAppearanceLogicalLength,
    pub(super) offset: UiAppearanceLogicalLength,
    pub(super) anti_alias_fringe: UiAppearanceLogicalLength,
    pub(super) color: UiMountedAppearanceColor,
    pub(super) opacity: UiMountedAppearanceOpacity,
}

pub(super) fn mounted_outline(
    input: MountedOutlineFixtureInput,
) -> UiMountedOutlineAppearanceMechanic {
    let geometry = UiAppearanceOutlineGeometry::admit(
        input.allocation,
        UiAppearanceNormalizedLogicalRadii::normalize(input.allocation, input.radii),
        input.line_width,
        input.offset,
        input.anti_alias_fringe,
    )
    .unwrap();
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let issuer = UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
    UiMountedOutlineAppearanceMechanic::complete_from_runtime_mounting(
        UiMountedOutlineAppearanceCompletionInput {
            issuer,
            node_receipt: issuer.receipt_for(UiMountedInstanceIdentity::mint_unbound().unwrap()),
            clip: input.clip,
            geometry,
            color: input.color,
            opacity: input.opacity,
            projection: UiMountedNodeAppearanceAttribution::from_runtime_mounting(issuer, 1, 1)
                .unwrap(),
        },
    )
    .unwrap()
}

pub(super) struct MountedBackdropFixtureInput {
    pub(super) semantic_surface: UiSemanticSurfaceIdentity,
    pub(super) ordinal: u32,
    pub(super) extent: UiAppearanceBackdropExtent,
    pub(super) clip: UiAppearanceClip,
    pub(super) background: UiMountedAppearanceColor,
    pub(super) opacity: UiMountedAppearanceOpacity,
}

pub(super) fn mounted_backdrop(input: MountedBackdropFixtureInput) -> UiMountedBackdropMechanic {
    let placement =
        UiOverlayPlacementReceipt::from_runtime_overlay_order(1, input.ordinal).unwrap();
    let identity = UiMountedBackdropIdentity::from_runtime_mounting(
        format!("test.backdrop.{}", input.ordinal),
        UiMountedBackdropScope::SurfaceSingleton(input.semantic_surface),
        u64::from(input.ordinal) + 1,
    )
    .unwrap();
    UiMountedBackdropMechanic::complete_from_runtime_mounting(UiMountedBackdropCompletionInput {
        identity,
        semantic_surface: input.semantic_surface,
        placement,
        extent: input.extent,
        clip: input.clip,
        background: input.background,
        opacity: input.opacity,
        attribution: UiMountedBackdropAppearanceAttribution::from_runtime_transport(
            input.semantic_surface,
            placement,
            u64::from(input.ordinal) + 10,
            1,
        )
        .unwrap(),
    })
    .unwrap()
}

pub(super) fn backdrop(ordinal: u32) -> UiMountedBackdropMechanic {
    let semantic_surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    backdrop_for_surface(semantic_surface, ordinal)
}

pub(super) fn backdrop_for_surface(
    semantic_surface: UiSemanticSurfaceIdentity,
    ordinal: u32,
) -> UiMountedBackdropMechanic {
    mounted_backdrop(MountedBackdropFixtureInput {
        semantic_surface,
        ordinal,
        extent: UiAppearanceBackdropExtent::new(0, 0, 40, 40).unwrap(),
        clip: UiAppearanceClip::new(0, 0, 40, 40).unwrap(),
        background: UiMountedAppearanceColor::from_straight_srgba([0, 0, 0, 128]),
        opacity: UiMountedAppearanceOpacity::ONE,
    })
}

pub(super) struct MountedTextForegroundFixtureInput {
    pub(super) seed: u8,
    pub(super) foreground: UiMountedAppearanceColor,
    pub(super) opacity: UiMountedAppearanceOpacity,
}

pub(super) fn mounted_text_foreground(
    input: MountedTextForegroundFixtureInput,
) -> UiMountedTextForegroundAppearanceMechanic {
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let issuer = UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
    UiMountedTextForegroundAppearanceMechanic::complete_from_runtime_mounting(
        UiMountedTextForegroundAppearanceCompletionInput {
            issuer,
            node_receipt: issuer.receipt_for(UiMountedInstanceIdentity::mint_unbound().unwrap()),
            paint_span: UiMountedTextPaintSpanIdentity::from_runtime_mounting([input.seed; 32]),
            foreground: input.foreground,
            opacity: input.opacity,
            projection: UiMountedNodeAppearanceAttribution::from_runtime_mounting(issuer, 1, 1)
                .unwrap(),
        },
    )
    .unwrap()
}

pub(super) fn text_foreground(seed: u8) -> UiMountedTextForegroundAppearanceMechanic {
    mounted_text_foreground(MountedTextForegroundFixtureInput {
        seed,
        foreground: UiMountedAppearanceColor::from_straight_srgba([255, 255, 255, 255]),
        opacity: UiMountedAppearanceOpacity::ONE,
    })
}

pub(super) struct MountedPointerAffordanceFixtureInput {
    pub(super) pointer: UiHostPointerIdentity,
    pub(super) semantic_surface: UiSemanticSurfaceIdentity,
    pub(super) target: UiMountedInstanceIdentity,
    pub(super) family: UiPointerAffordanceFamily,
}

pub(super) fn mounted_pointer_affordance(
    input: MountedPointerAffordanceFixtureInput,
) -> UiMountedPointerAffordanceMechanic {
    UiMountedPointerAffordanceMechanic::complete_from_runtime_mounting(
        input.pointer,
        input.semantic_surface,
        input.target,
        input.family,
    )
}

pub(super) fn pointer(family: UiPointerAffordanceFamily) -> UiMountedPointerAffordanceMechanic {
    mounted_pointer_affordance(MountedPointerAffordanceFixtureInput {
        pointer: UiHostPointerIdentity::new(1),
        semantic_surface: UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
        target: UiMountedInstanceIdentity::mint_unbound().unwrap(),
        family,
    })
}

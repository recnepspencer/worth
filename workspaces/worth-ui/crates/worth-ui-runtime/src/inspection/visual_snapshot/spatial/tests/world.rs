pub(super) struct SpatialWorld {
    pub(super) frame: worth_ui_host_contract::UiMountedFrameIdentity,
    pub(super) surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    pub(super) binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    pub(super) instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    pub(super) receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
}

impl SpatialWorld {
    pub(super) fn new() -> Self {
        let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
        let instance = worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
        let receipt = worth_ui_host_contract::UiMountedNodeReceiptIssuer::mint_for(frame)
            .unwrap()
            .receipt_for(instance);
        Self {
            frame,
            surface: worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
            binding: worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound().unwrap(),
            instance,
            receipt,
        }
    }
}

pub(super) fn paint(
    world: &SpatialWorld,
    bounds: worth_ui_host_contract::UiMountedCanonicalBox,
    order: u32,
    alpha: u8,
) -> crate::mounting::UiMountedUnsupportedPaintBasis {
    crate::mounting::UiMountedUnsupportedPaintBasis::new(
        world.receipt,
        bounds,
        bounds,
        order,
        u64::from(alpha),
    )
}

pub(super) fn hit_test(
    world: &SpatialWorld,
    bounds: worth_ui_host_contract::UiMountedCanonicalBox,
    order: u32,
) -> worth_ui_host_contract::UiMountedHitTestMechanic {
    worth_ui_host_contract::UiMountedHitTestMechanic::complete_from_runtime_mounting(
        worth_ui_host_contract::UiMountedHitTestCompletionInput {
            frame: world.frame,
            surface: world.surface,
            binding: world.binding,
            mounted_instance: world.instance,
            node_receipt: world.receipt,
            bounds,
            clip_bounds: bounds,
            order: worth_ui_host_contract::UiMountedHitTestOrder::from_runtime_plan(order),
        },
    )
    .unwrap()
}

pub(super) fn observed_paint(
    row: crate::mounting::UiMountedUnsupportedPaintBasis,
    order: u32,
) -> worth_ui_host_contract::UiHostRealizedRegion {
    observed(
        row.node_receipt(),
        realized_geometry(row.bounds(), row.clip()),
        realized_ordering(
            order,
            worth_ui_host_contract::UiHostRealizedRegionParticipation::Paint,
        ),
    )
}

pub(super) fn observed_hit(
    row: worth_ui_host_contract::UiMountedHitTestMechanic,
    order: u32,
) -> worth_ui_host_contract::UiHostRealizedRegion {
    observed(
        row.node_receipt(),
        realized_geometry(row.bounds(), row.clip_bounds()),
        realized_ordering(
            order,
            worth_ui_host_contract::UiHostRealizedRegionParticipation::HitTest,
        ),
    )
}

pub(super) fn observed(
    receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    geometry: worth_ui_host_contract::UiHostRealizedGeometry,
    ordering: worth_ui_host_contract::UiHostRealizedOrdering,
) -> worth_ui_host_contract::UiHostRealizedRegion {
    worth_ui_host_contract::UiHostRealizedRegion::observed_by_host(receipt, geometry, ordering)
}

pub(super) fn realized_geometry(
    bounds: worth_ui_host_contract::UiMountedCanonicalBox,
    clip: worth_ui_host_contract::UiMountedCanonicalBox,
) -> worth_ui_host_contract::UiHostRealizedGeometry {
    worth_ui_host_contract::UiHostRealizedGeometry::observed_by_host(bounds, clip)
}

pub(super) fn realized_ordering(
    order: u32,
    participation: worth_ui_host_contract::UiHostRealizedRegionParticipation,
) -> worth_ui_host_contract::UiHostRealizedOrdering {
    worth_ui_host_contract::UiHostRealizedOrdering::observed_by_host(order, participation)
}

pub(super) fn bounds(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> worth_ui_host_contract::UiMountedCanonicalBox {
    worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
        worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x,
            y,
            width,
            height,
            coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace::Viewport,
        },
    )
    .unwrap()
}

pub(super) fn transform(dimensions: [u32; 2]) -> worth_ui_host_contract::UiHostCoordinateTransform {
    worth_ui_host_contract::UiHostCoordinateTransform::observed_by_host(
        worth_ui_host_contract::UiHostClientAreaObservation::observed_by_host([0, 0], dimensions),
        worth_ui_host_contract::UiHostViewportTransformObservation::observed_by_host(
            [dimensions[0] as f32, dimensions[1] as f32],
            [1.0, 1.0],
            [0.0, 0.0],
        ),
        worth_ui_host_contract::UiHostCoordinatePosture::observed_by_host(
            worth_ui_host_contract::UiHostCoordinateOrientation::TopLeftOrigin,
            worth_ui_host_contract::UiHostCoordinateRounding::FloorEdges,
        ),
    )
}

pub(super) fn point(x: i64, y: i64) -> worth_ui_inspection::UiClientPhysicalPixel {
    worth_ui_inspection::UiClientPhysicalPixel::new(x, y).unwrap()
}

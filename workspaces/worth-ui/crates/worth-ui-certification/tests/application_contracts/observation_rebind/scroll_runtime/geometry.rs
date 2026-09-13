use worth_ui::facade::app::{
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
    UiMountedLayoutRevision, UiMountedOccurrenceGeometry, UiMountedSurfaceGeometryBatch,
    WorthUiActiveApplicationSession,
};
use worth_ui_runtime::facade::mounted::UiMountedAllocationProjection;
use worth_ui_test_support::{
    WorthUiActiveSessionCertificationExt, WorthUiMountedAllocationInspectionCertificationExt,
    WorthUiMountedIdentityCertificationExt,
};

pub(super) fn complete(session: &mut WorthUiActiveApplicationSession) {
    static REVISION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let identity = session.inspect_mounted_identity();
    let surface = identity.surface_bindings()[0].semantic_surface_identity();
    let region_inputs = session.declared_region_layout_inputs(surface);
    let content_owners = region_inputs
        .iter()
        .map(|region| region.owner())
        .collect::<std::collections::BTreeSet<_>>();
    let occurrences = identity
        .mounted_instances()
        .iter()
        .enumerate()
        .map(|(index, instance)| {
            let bounds = match session
                .inspect_mounted_allocation_projection(instance.graph_node_identity())
                .unwrap()
            {
                Some(UiMountedAllocationProjection::Known { bounds, .. }) => {
                    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
                        x: bounds.x(),
                        y: bounds.y(),
                        width: bounds.width(),
                        height: bounds.height(),
                        coordinate_space: UiMountedCoordinateSpace::HostSurface,
                    })
                    .unwrap()
                }
                _ => UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
                    x: 8.0 + index as f32 * 36.0,
                    y: 12.0,
                    width: 28.0,
                    height: 20.0,
                    coordinate_space: UiMountedCoordinateSpace::HostSurface,
                })
                .unwrap(),
            };
            let bounds = if content_owners.contains(&instance.identity()) {
                // Authored list content is 320x320, independently of its
                // 160x96 mounted viewport and graph measurement allocation.
                UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
                    x: bounds.x(),
                    y: bounds.y(),
                    width: 320.0,
                    height: 320.0,
                    coordinate_space: UiMountedCoordinateSpace::HostSurface,
                })
                .unwrap()
            } else {
                bounds
            };
            UiMountedOccurrenceGeometry::surface(instance.identity(), bounds)
        })
        .collect::<Vec<_>>();
    // The recorder's real viewport is 160x96, while these authored content
    // allocations overflow it. Supply concrete region viewports, rather than
    // letting graph allocations stand in for mounted Scroll ownership.
    let viewport = UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: 0.0,
        y: 0.0,
        width: 160.0,
        height: 96.0,
        coordinate_space: UiMountedCoordinateSpace::HostSurface,
    })
    .unwrap();
    let regions = region_inputs
        .iter()
        .map(|region| {
            region.geometry(
                UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
                    x: 0.0,
                    y: 0.0,
                    width: 160.0,
                    height: 96.0,
                    coordinate_space: UiMountedCoordinateSpace::GraphNodeLocal,
                })
                .unwrap(),
            )
        })
        .collect::<Vec<_>>();
    let mut layout = session.begin_mounted_layout();
    let basis = layout.basis(surface).unwrap();
    layout
        .complete_surface_geometry(
            UiMountedSurfaceGeometryBatch::new(
                basis,
                UiMountedLayoutRevision::new(
                    REVISION.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                )
                .unwrap(),
                viewport,
                occurrences,
            )
            .with_regions(regions),
        )
        .unwrap();
}

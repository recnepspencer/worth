use crate::facade::WorthUiActiveApplicationSession;
use crate::mounting::{
    UiMountedLayoutRevision, UiMountedOccurrenceGeometry, UiMountedSurfaceGeometryBatch,
};
use worth_ui_host_contract::*;

pub(super) const VIEWPORT: [f32; 4] = [0.0, 0.0, 800.0, 600.0];
pub(super) const SECONDARY_REGION: [f32; 4] = [50.0, 60.0, 120.0, 40.0];
pub(super) const BOXES: [[f32; 4]; 5] = [
    [40.0, 50.0, 180.0, 60.0],
    [300.0, 50.0, 220.0, 70.0],
    [80.0, 200.0, 160.0, 60.0],
    [40.0, 50.0, 180.0, 60.0],
    [8.0, 12.0, 140.0, 36.0],
];

pub(super) fn canonical([x, y, width, height]: [f32; 4]) -> UiMountedCanonicalBox {
    canonical_in([x, y, width, height], UiMountedCoordinateSpace::HostSurface)
}

pub(super) fn viewport([x, y, width, height]: [f32; 4]) -> UiMountedCanonicalBox {
    canonical_in([x, y, width, height], UiMountedCoordinateSpace::Viewport)
}

fn canonical_in(
    [x, y, width, height]: [f32; 4],
    coordinate_space: UiMountedCoordinateSpace,
) -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x,
        y,
        width,
        height,
        coordinate_space,
    })
    .unwrap()
}

pub(super) fn install(
    session: &mut WorthUiActiveApplicationSession,
    surfaces: [UiSemanticSurfaceIdentity; 2],
    instances: [UiMountedInstanceIdentity; 5],
    declarations: [worth_ui_dsl::UiMosaicRegionDeclarationIdentity; 2],
) {
    install_with_child_region(session, surfaces, instances, declarations, 2, BOXES, None);
}

pub(super) fn install_disjoint_child_region(
    session: &mut WorthUiActiveApplicationSession,
    surfaces: [UiSemanticSurfaceIdentity; 2],
    instances: [UiMountedInstanceIdentity; 5],
    declarations: [worth_ui_dsl::UiMosaicRegionDeclarationIdentity; 2],
) {
    install_with_child_region(
        session,
        surfaces,
        instances,
        declarations,
        3,
        BOXES,
        Some([1_000.0, 1_000.0, 20.0, 20.0]),
    );
}

pub(super) fn install_seam_pair(
    session: &mut WorthUiActiveApplicationSession,
    surfaces: [UiSemanticSurfaceIdentity; 2],
    instances: [UiMountedInstanceIdentity; 5],
    declarations: [worth_ui_dsl::UiMosaicRegionDeclarationIdentity; 2],
) {
    install_with_child_region(
        session,
        surfaces,
        instances,
        declarations,
        4,
        [
            [0.0, 0.0, 100.0, 100.0],
            [100.0, 0.0, 100.0, 100.0],
            [300.0, 200.0, 100.0, 100.0],
            BOXES[3],
            [10.0, 10.0, 40.0, 40.0],
        ],
        None,
    );
}

fn install_with_child_region(
    session: &mut WorthUiActiveApplicationSession,
    surfaces: [UiSemanticSurfaceIdentity; 2],
    instances: [UiMountedInstanceIdentity; 5],
    _declarations: [worth_ui_dsl::UiMosaicRegionDeclarationIdentity; 2],
    revision: u64,
    boxes: [[f32; 4]; 5],
    child_region: Option<[f32; 4]>,
) {
    for (index, surface) in surfaces.into_iter().enumerate() {
        let surface_declaration = session
            .application
            .authored_overlay_material()
            .overlay_declaration_bindings()
            .surface_named(if index == 0 {
                "workspace.surface.overlay"
            } else {
                "workspace.surface.secondary"
            })
            .unwrap();
        let rows = instances
            .iter()
            .enumerate()
            .filter(|(mount, _)| (*mount == 3) == (index == 1))
            .map(|(mount, instance)| {
                if mount == 4 {
                    UiMountedOccurrenceGeometry::parent_relative(
                        *instance,
                        instances[0],
                        canonical_in(boxes[mount], UiMountedCoordinateSpace::GraphNodeLocal),
                    )
                } else {
                    UiMountedOccurrenceGeometry::surface(*instance, canonical(boxes[mount]))
                }
            })
            .collect::<Vec<_>>();
        let regions = instances
            .iter()
            .enumerate()
            .filter(|(mount, _)| (*mount == 3) == (index == 1))
            .flat_map(|(mount, instance)| {
                let mounted = session
                    .mounted
                    .current_mounted_identity_basis(*instance)
                    .unwrap();
                let bindings = session
                    .application
                    .mounted_region_declarations(surface_declaration, mounted.graph_node_identity())
                    .0;
                let bounds = if mount == 4 {
                    child_region.unwrap_or([0.0, 0.0, boxes[mount][2], boxes[mount][3]])
                } else if mount == 3 {
                    [10.0, 10.0, 120.0, 40.0]
                } else {
                    [0.0, 0.0, boxes[mount][2], boxes[mount][3]]
                };
                bindings
                    .into_iter()
                    .map(move |binding| {
                        crate::mounting::UiMountedMosaicRegionGeometry::new(
                            *instance,
                            binding.declaration(),
                            binding.executed_region(),
                            canonical_in(bounds, UiMountedCoordinateSpace::GraphNodeLocal),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let mut layout = session.begin_mounted_layout();
        let basis = layout.basis(surface).unwrap();
        let mut batch = UiMountedSurfaceGeometryBatch::new(
            basis,
            UiMountedLayoutRevision::new(revision).unwrap(),
            canonical(VIEWPORT),
            rows,
        );
        batch = batch.with_regions(regions);
        layout.complete_surface_geometry(batch).unwrap();
    }
}

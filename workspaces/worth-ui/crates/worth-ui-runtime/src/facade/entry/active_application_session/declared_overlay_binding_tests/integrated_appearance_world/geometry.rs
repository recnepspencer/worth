use crate::facade::WorthUiActiveApplicationSession;
use crate::mounting::{
    UiMountedLayoutRevision, UiMountedOccurrenceGeometry, UiMountedSurfaceGeometryBatch,
};
use worth_ui_host_contract::*;

#[path = "geometry/scrollable.rs"]
pub(super) mod scrollable;

pub(super) const VIEWPORT: [f32; 4] = [0.0, 0.0, 800.0, 600.0];
pub(super) const SECONDARY_REGION: [f32; 4] = [50.0, 60.0, 120.0, 40.0];
pub(super) const BOXES: [[f32; 4]; 5] = [
    [40.0, 50.0, 180.0, 60.0],
    [300.0, 50.0, 220.0, 70.0],
    [80.0, 200.0, 160.0, 60.0],
    [40.0, 50.0, 180.0, 60.0],
    [8.0, 12.0, 140.0, 36.0],
];
pub(super) const MOVED_TARGET_BOX: [f32; 4] = [560.0, 420.0, 180.0, 60.0];

/// Rows that differ from the defaults: one region filling its owner, and every
/// component but the child laid out against the surface.
#[derive(Clone, Copy, Default)]
struct RegionOverrides {
    /// The child occurrence's region, in its owner-local space.
    child: Option<[f32; 4]>,
    /// The first component's region, in its own local space.
    primary: Option<[f32; 4]>,
    /// One more mount laid out relative to the first component rather than the
    /// surface, so it is that component's content and travels with its scroll
    /// offset. The child occurrence is by default; this names a second.
    nested: Option<usize>,
    /// A region nested in the first component's: its owner is laid out as
    /// that component's content, and one more mount as its own.
    inner: Option<InnerRegion>,
    /// Lay the child occurrence out against the surface instead of inside the
    /// first component, so that component's region stops carrying it while it
    /// stays mounted.
    detach_child: bool,
}

/// A mount that owns a region while it travels as the first component's
/// content, and the mount laid out as that region's content.
#[derive(Clone, Copy)]
struct InnerRegion {
    owner: usize,
    /// The owner's region, in its own local space.
    region: [f32; 4],
    content: usize,
}

impl RegionOverrides {
    /// The mount `mount` is laid out relative to, if not the surface.
    fn parent(&self, mount: usize) -> Option<usize> {
        match self.inner {
            Some(inner) if inner.content == mount => Some(inner.owner),
            Some(inner) if inner.owner == mount => Some(0),
            _ if (mount == 4 && !self.detach_child) || Some(mount) == self.nested => Some(0),
            _ => None,
        }
    }
}

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
    _declarations: [worth_ui_dsl::UiMosaicRegionDeclarationIdentity; 2],
) {
    install_with_child_region(
        session,
        surfaces,
        instances,
        2,
        BOXES,
        RegionOverrides::default(),
        None,
        VIEWPORT,
    );
}

pub(super) fn install_without_target(
    session: &mut WorthUiActiveApplicationSession,
    surfaces: [UiSemanticSurfaceIdentity; 2],
    instances: [UiMountedInstanceIdentity; 5],
    _declarations: [worth_ui_dsl::UiMosaicRegionDeclarationIdentity; 2],
) {
    install_with_child_region(
        session,
        surfaces,
        instances,
        10,
        BOXES,
        RegionOverrides::default(),
        Some(1),
        VIEWPORT,
    );
}

pub(super) fn install_successor(
    session: &mut WorthUiActiveApplicationSession,
    surfaces: [UiSemanticSurfaceIdentity; 2],
    instances: [UiMountedInstanceIdentity; 5],
    _declarations: [worth_ui_dsl::UiMosaicRegionDeclarationIdentity; 2],
) {
    install_with_child_region(
        session,
        surfaces,
        instances,
        11,
        BOXES,
        RegionOverrides::default(),
        None,
        VIEWPORT,
    );
}

pub(super) fn install_moved_target(
    session: &mut WorthUiActiveApplicationSession,
    surfaces: [UiSemanticSurfaceIdentity; 2],
    instances: [UiMountedInstanceIdentity; 5],
    _declarations: [worth_ui_dsl::UiMosaicRegionDeclarationIdentity; 2],
) {
    let mut boxes = BOXES;
    boxes[1] = MOVED_TARGET_BOX;
    install_with_child_region(
        session,
        surfaces,
        instances,
        12,
        boxes,
        RegionOverrides::default(),
        None,
        VIEWPORT,
    );
}

/// Lays the surfaces out again with the Portal owner at `owner` inside
/// `viewport`, as a resize or a moved anchor would.
pub(super) fn install_owner_in_viewport(
    session: &mut WorthUiActiveApplicationSession,
    surfaces: [UiSemanticSurfaceIdentity; 2],
    instances: [UiMountedInstanceIdentity; 5],
    revision: u64,
    owner: [f32; 4],
    viewport: [f32; 4],
) {
    let mut boxes = BOXES;
    boxes[0] = owner;
    install_with_child_region(
        session,
        surfaces,
        instances,
        revision,
        boxes,
        RegionOverrides::default(),
        None,
        viewport,
    );
}

/// Lays the surfaces out again with the Portal child at `child` inside
/// `viewport`, as a resize that lays the Portal's content out anew would.
pub(super) fn install_child_in_viewport(
    session: &mut WorthUiActiveApplicationSession,
    surfaces: [UiSemanticSurfaceIdentity; 2],
    instances: [UiMountedInstanceIdentity; 5],
    revision: u64,
    child: [f32; 4],
    viewport: [f32; 4],
) {
    let mut boxes = BOXES;
    boxes[4] = child;
    install_with_child_region(
        session,
        surfaces,
        instances,
        revision,
        boxes,
        RegionOverrides::default(),
        None,
        viewport,
    );
}

pub(super) fn install_restored_target(
    session: &mut WorthUiActiveApplicationSession,
    surfaces: [UiSemanticSurfaceIdentity; 2],
    instances: [UiMountedInstanceIdentity; 5],
    _declarations: [worth_ui_dsl::UiMosaicRegionDeclarationIdentity; 2],
) {
    install_with_child_region(
        session,
        surfaces,
        instances,
        13,
        BOXES,
        RegionOverrides::default(),
        None,
        VIEWPORT,
    );
}

pub(super) fn install_disjoint_child_region(
    session: &mut WorthUiActiveApplicationSession,
    surfaces: [UiSemanticSurfaceIdentity; 2],
    instances: [UiMountedInstanceIdentity; 5],
    _declarations: [worth_ui_dsl::UiMosaicRegionDeclarationIdentity; 2],
) {
    install_with_child_region(
        session,
        surfaces,
        instances,
        3,
        BOXES,
        RegionOverrides {
            child: Some([1_000.0, 1_000.0, 20.0, 20.0]),
            primary: None,
            nested: None,
            inner: None,
            detach_child: false,
        },
        None,
        VIEWPORT,
    );
}

pub(super) fn install_seam_pair(
    session: &mut WorthUiActiveApplicationSession,
    surfaces: [UiSemanticSurfaceIdentity; 2],
    instances: [UiMountedInstanceIdentity; 5],
    _declarations: [worth_ui_dsl::UiMosaicRegionDeclarationIdentity; 2],
) {
    install_with_child_region(
        session,
        surfaces,
        instances,
        4,
        [
            [0.0, 0.0, 100.0, 100.0],
            [100.0, 0.0, 100.0, 100.0],
            [300.0, 200.0, 100.0, 100.0],
            BOXES[3],
            [10.0, 10.0, 40.0, 40.0],
        ],
        RegionOverrides::default(),
        None,
        VIEWPORT,
    );
}

fn install_with_child_region(
    session: &mut WorthUiActiveApplicationSession,
    surfaces: [UiSemanticSurfaceIdentity; 2],
    instances: [UiMountedInstanceIdentity; 5],
    revision: u64,
    boxes: [[f32; 4]; 5],
    overrides: RegionOverrides,
    excluded: Option<usize>,
    viewport: [f32; 4],
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
            .filter(|(mount, _)| Some(*mount) != excluded && (*mount == 3) == (index == 1))
            .map(|(mount, instance)| {
                if let Some(parent) = overrides.parent(mount) {
                    UiMountedOccurrenceGeometry::parent_relative(
                        *instance,
                        instances[parent],
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
            .filter(|(mount, _)| Some(*mount) != excluded && (*mount == 3) == (index == 1))
            .flat_map(|(mount, instance)| {
                let mounted = session
                    .mounted
                    .current_mounted_identity_basis(*instance)
                    .unwrap();
                let bindings = session
                    .application
                    .mounted_region_declarations(surface_declaration, mounted.graph_node_identity())
                    .0;
                let filling = [0.0, 0.0, boxes[mount][2], boxes[mount][3]];
                let bounds = match mount {
                    _ if overrides.inner.is_some_and(|inner| inner.owner == mount) => {
                        overrides.inner.map_or(filling, |inner| inner.region)
                    }
                    4 => overrides.child.unwrap_or(filling),
                    3 => [10.0, 10.0, 120.0, 40.0],
                    0 => overrides.primary.unwrap_or(filling),
                    _ => filling,
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
            canonical(viewport),
            rows,
        );
        batch = batch.with_regions(regions);
        layout.complete_surface_geometry(batch).unwrap();
    }
}

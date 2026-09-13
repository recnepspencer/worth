use worth_ui::facade::app::{
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
    UiMountedLayoutRevision, UiMountedOccurrenceGeometry, UiMountedSurfaceGeometryBatch,
    WorthUiActiveApplicationSession,
};
use worth_ui_host_contract::{UiMountedInstanceIdentity, UiSemanticSurfaceIdentity};

pub(super) struct MountedNode {
    pub(super) authored_index: usize,
    pub(super) instance: UiMountedInstanceIdentity,
    pub(super) surface: UiSemanticSurfaceIdentity,
}

pub(super) fn install(
    session: &mut WorthUiActiveApplicationSession,
    surfaces: &[UiSemanticSurfaceIdentity; super::SURFACE_COUNT],
    nodes: &[MountedNode],
) {
    let mut layout = session.begin_mounted_layout();
    for surface in surfaces {
        let basis = layout.basis(*surface).unwrap();
        let occurrences = nodes
            .iter()
            .filter(|node| node.surface == *surface)
            .map(|node| {
                UiMountedOccurrenceGeometry::surface(
                    node.instance,
                    canonical_box(bounds(node.authored_index)),
                )
            })
            .collect::<Vec<_>>();
        layout
            .complete_surface_geometry(UiMountedSurfaceGeometryBatch::new(
                basis,
                UiMountedLayoutRevision::new(1).unwrap(),
                canonical_box([0.0, 0.0, 1_280.0, 720.0]),
                occurrences,
            ))
            .expect("AP10 exact authored positions cover every live occurrence");
    }
}

pub(super) fn surface_index(index: usize) -> usize {
    if index < super::STYLED_COUNT {
        index / (super::STYLED_COUNT / super::SURFACE_COUNT)
    } else {
        (index - super::STYLED_COUNT)
            / ((super::NODE_COUNT - super::STYLED_COUNT) / super::SURFACE_COUNT)
    }
}

pub(super) fn bounds(index: usize) -> [f32; 4] {
    let position = if index < super::STYLED_COUNT {
        index % (super::STYLED_COUNT / super::SURFACE_COUNT)
    } else {
        super::STYLED_COUNT / super::SURFACE_COUNT
            + (index - super::STYLED_COUNT)
                % ((super::NODE_COUNT - super::STYLED_COUNT) / super::SURFACE_COUNT)
    };
    [
        (position % 32) as f32 * 40.0,
        (position / 32) as f32 * 22.0,
        38.0,
        20.0,
    ]
}

fn canonical_box([x, y, width, height]: [f32; 4]) -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x,
        y,
        width,
        height,
        coordinate_space: UiMountedCoordinateSpace::HostSurface,
    })
    .unwrap()
}

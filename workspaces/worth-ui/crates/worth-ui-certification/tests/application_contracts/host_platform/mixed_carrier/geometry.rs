use worth_ui::facade::app::{
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
    UiMountedLayoutRevision, UiMountedOccurrenceGeometry, UiMountedSurfaceGeometryBatch,
    WorthUiActiveApplicationSession,
};
use worth_ui_test_support::WorthUiMountedIdentityCertificationExt;

use super::MountedMixedRows;

pub(super) fn install(
    session: &mut WorthUiActiveApplicationSession,
    mounted: &MountedMixedRows,
    revision: u64,
) {
    let identity = session.inspect_mounted_identity();
    let live = identity
        .mounted_instances()
        .iter()
        .map(|instance| instance.identity())
        .collect::<BTreeSet<_>>();
    let occurrences = mounted
        .rows
        .iter()
        .enumerate()
        .filter(|(_, row)| live.contains(&row.instance))
        .map(|(slot, row)| {
            let column = (slot % 64) as f32;
            let line = (slot / 64) as f32;
            UiMountedOccurrenceGeometry::surface(
                row.instance,
                canonical_box([column * 20.0, line * 11.0, 18.0, 9.0]),
            )
        })
        .collect::<Vec<_>>();
    let mut layout = session.begin_mounted_layout();
    let basis = layout
        .basis(mounted.surface)
        .expect("mixed carrier surface remains bound during stable layout");
    layout
        .complete_surface_geometry(UiMountedSurfaceGeometryBatch::new(
            basis,
            UiMountedLayoutRevision::new(revision)
                .expect("mixed carrier layout revision is nonzero"),
            canonical_box([0.0, 0.0, 1_280.0, 720.0]),
            occurrences,
        ))
        .expect("mixed carrier layout covers each live stable row slot");
}

fn canonical_box([x, y, width, height]: [f32; 4]) -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x,
        y,
        width,
        height,
        coordinate_space: UiMountedCoordinateSpace::HostSurface,
    })
    .expect("mixed carrier geometry is finite and nonnegative")
}
use std::collections::BTreeSet;

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

use worth_ui::facade::app::{
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
    UiMountedLayoutRevision, UiMountedOccurrenceGeometry, UiMountedSurfaceGeometryBatch,
    UiNativeMountedComponentLayoutInput, WorthUiActiveApplicationSession,
};
use worth_ui_runtime::facade::mounted::UiMountedAllocationProjection;
use worth_ui_test_support::{
    WorthUiMountedAllocationInspectionCertificationExt, WorthUiMountedIdentityCertificationExt,
};

static NEXT_LAYOUT_REVISION: AtomicU64 = AtomicU64::new(1);
pub(crate) fn install_current_occurrence_geometry(session: &mut WorthUiActiveApplicationSession) {
    let identity = session.inspect_mounted_identity();
    let mut surfaces = identity
        .mounted_instances()
        .iter()
        .map(|instance| instance.basis().semantic_surface_identity())
        .collect::<Vec<_>>();
    surfaces.sort_unstable();
    surfaces.dedup();
    let occurrence_counts =
        identity
            .mounted_instances()
            .iter()
            .fold(BTreeMap::new(), |mut counts, instance| {
                *counts
                    .entry((
                        instance.basis().semantic_surface_identity(),
                        instance.graph_node_identity(),
                    ))
                    .or_insert(0usize) += 1;
                counts
            });
    for surface in surfaces {
        let mut instances = identity
            .mounted_instances()
            .iter()
            .filter(|instance| instance.basis().semantic_surface_identity() == surface)
            .map(|instance| instance.identity())
            .collect::<Vec<_>>();
        instances.sort_unstable();
        let occurrences = instances
            .into_iter()
            .enumerate()
            .map(|(index, instance)| {
                let graph_node = identity
                    .mounted_instances()
                    .iter()
                    .find(|candidate| candidate.identity() == instance)
                    .expect("current occurrence remains in the identity view")
                    .graph_node_identity();
                let bounds = if occurrence_counts[&(surface, graph_node)] > 1 {
                    repeated_occurrence_bounds(index)
                } else {
                    match session.inspect_mounted_allocation_projection(graph_node) {
                        Ok(Some(UiMountedAllocationProjection::Known { bounds, .. })) => {
                            canonical_box([bounds.x(), bounds.y(), bounds.width(), bounds.height()])
                        }
                        _ => occurrence_bounds(index),
                    }
                };
                UiMountedOccurrenceGeometry::surface(instance, bounds)
            })
            .collect::<Vec<_>>();
        let revision = NEXT_LAYOUT_REVISION.fetch_add(1, Ordering::Relaxed);
        let mut layout = session.begin_mounted_layout();
        let basis = layout
            .basis(surface)
            .expect("mounted certification surface remains bound during layout");
        layout
            .complete_surface_geometry(UiMountedSurfaceGeometryBatch::new(
                basis,
                UiMountedLayoutRevision::new(revision)
                    .expect("certification layout revisions remain nonzero"),
                viewport_bounds(),
                occurrences,
            ))
            .expect("certification layout covers every mounted occurrence");
    }
}

pub(crate) fn install_native_occurrence_geometry(
    shell: &mut worth_ui::facade::app::WorthUiNativeApplicationShell,
) {
    let inputs = shell.native_component_layout_inputs();
    // Scripted native allocation observations and input drains use 800 by 600.
    let viewport = canonical_box([0.0, 0.0, 800.0, 600.0]);
    let allocated = inputs
        .iter()
        .filter(|input| input.allocation().is_some())
        .cloned()
        .collect::<Vec<_>>();
    let mut resolved =
        UiNativeMountedComponentLayoutInput::resolve_occurrences(viewport, &allocated)
            .expect("certification components declare placeable allocations")
            .into_vec()
            .into_iter();
    // Occurrences keep the mounted order; a component without an allocation
    // contract takes a distinct grid slot.
    let occurrences = inputs
        .iter()
        .enumerate()
        .map(|(index, input)| {
            if input.allocation().is_some() {
                return resolved
                    .next()
                    .expect("one resolved occurrence per allocated component");
            }
            let bounds = occurrence_bounds(index);
            let bounds = [bounds.x(), bounds.y(), bounds.width(), bounds.height()];
            match input.portal_parent() {
                Some(parent) => UiMountedOccurrenceGeometry::parent_relative(
                    input.instance(),
                    parent,
                    local_box(bounds),
                ),
                None => {
                    UiMountedOccurrenceGeometry::surface(input.instance(), canonical_box(bounds))
                }
            }
        })
        .collect::<Vec<_>>();
    let regions = shell
        .native_region_layout_inputs()
        .iter()
        .map(|region| region.geometry(local_box([0.0, 0.0, 28.0, 20.0])))
        .collect::<Vec<_>>();
    let basis = shell
        .native_layout_basis()
        .expect("native certification surface remains available during layout");
    let revision = shell
        .next_native_layout_revision()
        .expect("native certification layout revision remains available");
    shell
        .complete_native_layout(
            UiMountedSurfaceGeometryBatch::new(basis, revision, viewport, occurrences)
                .with_regions(regions),
        )
        .expect("native certification layout covers every mounted occurrence");
}

fn occurrence_bounds(index: usize) -> UiMountedCanonicalBox {
    canonical_box([
        8.0 + (index % 32) as f32 * 36.0,
        12.0 + (index / 32) as f32 * 28.0,
        28.0,
        20.0,
    ])
}

fn repeated_occurrence_bounds(index: usize) -> UiMountedCanonicalBox {
    canonical_box([
        (index % 64) as f32 * 20.0,
        (index / 64) as f32 * 11.0,
        18.0,
        9.0,
    ])
}

fn viewport_bounds() -> UiMountedCanonicalBox {
    canonical_box([0.0, 0.0, 1_280.0, 720.0])
}

fn local_box([x, y, width, height]: [f32; 4]) -> UiMountedCanonicalBox {
    box_in_space(
        [x, y, width, height],
        UiMountedCoordinateSpace::GraphNodeLocal,
    )
}

fn canonical_box([x, y, width, height]: [f32; 4]) -> UiMountedCanonicalBox {
    box_in_space([x, y, width, height], UiMountedCoordinateSpace::HostSurface)
}

fn box_in_space(
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
    .expect("certification geometry is finite host-surface geometry")
}

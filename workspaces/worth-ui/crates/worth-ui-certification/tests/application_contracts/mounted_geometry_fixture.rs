use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};

use worth_ui::facade::app::{
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
    UiMountedLayoutRevision, UiMountedOccurrenceGeometry, UiMountedSurfaceGeometryBatch,
    WorthUiActiveApplicationSession,
};
use worth_ui::facade::declaration::{
    ComponentAllocationMeasurementContract, ComponentViewportAxisPlacement,
};
use worth_ui_runtime::facade::mounted::UiMountedAllocationProjection;
use worth_ui_test_support::{
    WorthUiMountedAllocationInspectionCertificationExt, WorthUiMountedIdentityCertificationExt,
};

static NEXT_LAYOUT_REVISION: AtomicU64 = AtomicU64::new(1);
static INSTALLED_TOPOLOGY: LazyLock<Mutex<BTreeMap<u64, String>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));

pub(crate) fn install_current_occurrence_geometry(session: &mut WorthUiActiveApplicationSession) {
    let identity = session.inspect_mounted_identity();
    let bound_surfaces = identity
        .surface_bindings()
        .iter()
        .map(|binding| {
            (
                binding.semantic_surface_identity(),
                binding.host_surface_identity(),
            )
        })
        .collect::<Vec<_>>();
    let topology = format!("{:?}|{bound_surfaces:?}", identity.mounted_instances());
    let session_identity = session.session_identity().as_u64();
    if INSTALLED_TOPOLOGY
        .lock()
        .expect("certification geometry topology cache remains available")
        .get(&session_identity)
        == Some(&topology)
    {
        return;
    }
    let mut surfaces = identity
        .mounted_instances()
        .iter()
        .map(|instance| instance.basis().semantic_surface_identity())
        .collect::<Vec<_>>();
    surfaces.sort_unstable();
    surfaces.dedup();

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
                let bounds = match session.inspect_mounted_allocation_projection(graph_node) {
                    Ok(Some(UiMountedAllocationProjection::Known { bounds, .. })) => {
                        canonical_box([bounds.x(), bounds.y(), bounds.width(), bounds.height()])
                    }
                    _ => occurrence_bounds(index),
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
    INSTALLED_TOPOLOGY
        .lock()
        .expect("certification geometry topology cache remains available")
        .insert(session_identity, topology);
}

pub(crate) fn install_native_occurrence_geometry(
    shell: &mut worth_ui::facade::app::WorthUiNativeApplicationShell,
) {
    let inputs = shell.native_component_layout_inputs();
    let viewport = viewport_bounds();
    let mut surface_bounds = BTreeMap::new();
    for (index, input) in inputs.iter().enumerate() {
        let coordinate_space = if input.portal_parent().is_some() {
            UiMountedCoordinateSpace::GraphNodeLocal
        } else {
            UiMountedCoordinateSpace::HostSurface
        };
        let bounds = input
            .allocation()
            .map(|contract| resolve_allocation(contract, viewport, coordinate_space))
            .unwrap_or_else(|| {
                let bounds = occurrence_bounds(index);
                box_in_space(
                    [bounds.x(), bounds.y(), bounds.width(), bounds.height()],
                    coordinate_space,
                )
            });
        surface_bounds.insert(input.instance(), bounds);
    }
    let occurrences = inputs
        .iter()
        .map(|input| match input.portal_parent() {
            Some(parent) => UiMountedOccurrenceGeometry::parent_relative(
                input.instance(),
                parent,
                surface_bounds[&input.instance()],
            ),
            None => UiMountedOccurrenceGeometry::surface(
                input.instance(),
                surface_bounds[&input.instance()],
            ),
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

fn resolve_allocation(
    contract: ComponentAllocationMeasurementContract,
    viewport: UiMountedCanonicalBox,
    coordinate_space: UiMountedCoordinateSpace,
) -> UiMountedCanonicalBox {
    let (x, y, width, height) = match contract {
        ComponentAllocationMeasurementContract::FillViewport => {
            (0.0, 0.0, viewport.width(), viewport.height())
        }
        ComponentAllocationMeasurementContract::ViewportInset(inset) => {
            let horizontal = f32::from(inset.horizontal_logical_points());
            let vertical = f32::from(inset.vertical_logical_points());
            (
                horizontal,
                vertical,
                (viewport.width() - 2.0 * horizontal).max(0.0),
                (viewport.height() - 2.0 * vertical).max(0.0),
            )
        }
        ComponentAllocationMeasurementContract::ViewportRegion(region) => {
            let horizontal = resolve_axis(region.horizontal(), viewport.width());
            let vertical = resolve_axis(region.vertical(), viewport.height());
            (horizontal.0, vertical.0, horizontal.1, vertical.1)
        }
        ComponentAllocationMeasurementContract::FixedLogicalSize { width, height } => {
            (0.0, 0.0, f32::from(width), f32::from(height))
        }
    };
    box_in_space([x, y, width, height], coordinate_space)
}

fn resolve_axis(axis: ComponentViewportAxisPlacement, available: f32) -> (f32, f32) {
    match axis {
        ComponentViewportAxisPlacement::FixedFromStart {
            start_logical_points,
            extent_logical_points,
        } => (
            f32::from(start_logical_points),
            f32::from(extent_logical_points),
        ),
        ComponentViewportAxisPlacement::StretchBetween {
            start_logical_points,
            end_logical_points,
        } => {
            let start = f32::from(start_logical_points);
            (
                start,
                (available - start - f32::from(end_logical_points)).max(0.0),
            )
        }
        ComponentViewportAxisPlacement::FixedFromEnd {
            end_logical_points,
            extent_logical_points,
        } => {
            let extent = f32::from(extent_logical_points);
            (
                (available - f32::from(end_logical_points) - extent).max(0.0),
                extent,
            )
        }
    }
}

fn occurrence_bounds(index: usize) -> UiMountedCanonicalBox {
    canonical_box([
        8.0 + (index % 32) as f32 * 36.0,
        12.0 + (index / 32) as f32 * 28.0,
        28.0,
        20.0,
    ])
}

fn viewport_bounds() -> UiMountedCanonicalBox {
    canonical_box([0.0, 0.0, 1_280.0, 720.0])
}

fn local_box([x, y, width, height]: [f32; 4]) -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x,
        y,
        width,
        height,
        coordinate_space: UiMountedCoordinateSpace::GraphNodeLocal,
    })
    .expect("certification geometry is finite graph-local geometry")
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

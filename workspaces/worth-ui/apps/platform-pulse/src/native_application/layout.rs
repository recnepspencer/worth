use worth_ui::facade::app::{
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
    UiMountedOccurrenceGeometry, UiMountedSurfaceGeometryBatch, WorthUiNativeApplicationShell,
};
use worth_ui::facade::declaration::{
    ComponentAllocationMeasurementContract, ComponentViewportAxisPlacement,
};
use worth_ui_platform_pulse::product_world::PlatformPulseMosaicRegion;

pub(super) fn publish_native_layout(
    shell: &mut WorthUiNativeApplicationShell,
) -> Result<bool, String> {
    let Some(viewport) = shell.native_layout_viewport() else {
        return Ok(false);
    };
    let mut occurrences = Vec::new();
    for component in shell.native_component_layout_inputs() {
        let contract = component.allocation().ok_or_else(|| {
            format!(
                "native-layout-missing-allocation:{}",
                component.authored_semantic_identity()
            )
        })?;
        let coordinate_space = if component.portal_parent().is_some() {
            UiMountedCoordinateSpace::GraphNodeLocal
        } else {
            UiMountedCoordinateSpace::HostSurface
        };
        let bounds = resolve_allocation(contract, viewport, coordinate_space)?;
        occurrences.push(match component.portal_parent() {
            Some(parent) => {
                UiMountedOccurrenceGeometry::parent_relative(component.instance(), parent, bounds)
            }
            None => UiMountedOccurrenceGeometry::surface(component.instance(), bounds),
        });
    }
    let occurrence_index = occurrences
        .iter()
        .copied()
        .map(|occurrence| (occurrence.instance(), occurrence))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut host_bounds = std::collections::BTreeMap::new();
    let regions = shell
        .native_region_layout_inputs()
        .iter()
        .map(|region| {
            let owner = resolve_host_bounds(
                region.owner(),
                &occurrence_index,
                &mut host_bounds,
                occurrence_index.len(),
            )?;
            region_bounds(region.region_kind(), viewport, owner)
                .map(|bounds| region.geometry(bounds))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let basis = shell
        .native_layout_basis()
        .map_err(|denial| format!("native-layout-basis:{denial:?}"))?;
    let revision = shell
        .next_native_layout_revision()
        .map_err(|denial| format!("native-layout-revision:{denial:?}"))?;
    shell
        .complete_native_layout(
            UiMountedSurfaceGeometryBatch::new(basis, revision, viewport, occurrences)
                .with_regions(regions),
        )
        .map_err(|denial| format!("native-layout-completion:{denial:?}"))?;
    Ok(true)
}

fn resolve_allocation(
    contract: ComponentAllocationMeasurementContract,
    viewport: UiMountedCanonicalBox,
    coordinate_space: UiMountedCoordinateSpace,
) -> Result<UiMountedCanonicalBox, String> {
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
    canonical_box(x, y, width, height, coordinate_space)
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

fn region_bounds(
    region_kind: &str,
    viewport: UiMountedCanonicalBox,
    owner: UiMountedCanonicalBox,
) -> Result<UiMountedCanonicalBox, String> {
    let width = viewport.width();
    let height = viewport.height();
    let (x, y, region_width, region_height) = match region_kind {
        kind if kind == PlatformPulseMosaicRegion::Viewport.id() => (0.0, 0.0, width, height),
        kind if kind == PlatformPulseMosaicRegion::Masthead.id() => {
            (24.0, 24.0, (width - 48.0).max(0.0), 56.0)
        }
        kind if kind == PlatformPulseMosaicRegion::EvidenceRail.id() => {
            (24.0, 104.0, 216.0, (height - 176.0).max(0.0))
        }
        kind if kind == PlatformPulseMosaicRegion::ServiceStage.id() => (
            264.0,
            104.0,
            (width - 288.0).max(0.0),
            (height - 176.0).max(0.0),
        ),
        kind if kind == PlatformPulseMosaicRegion::StatusBand.id() => (
            24.0,
            (height - 48.0).max(0.0),
            (width - 48.0).max(0.0),
            24.0,
        ),
        _ => return Err(format!("native-layout-unknown-mosaic-region:{region_kind}")),
    };
    canonical_box(
        x - owner.x(),
        y - owner.y(),
        region_width,
        region_height,
        UiMountedCoordinateSpace::GraphNodeLocal,
    )
}

fn resolve_host_bounds(
    instance: worth_ui::facade::app::UiMountedInstanceIdentity,
    occurrence_index: &std::collections::BTreeMap<
        worth_ui::facade::app::UiMountedInstanceIdentity,
        UiMountedOccurrenceGeometry,
    >,
    resolved: &mut std::collections::BTreeMap<
        worth_ui::facade::app::UiMountedInstanceIdentity,
        UiMountedCanonicalBox,
    >,
    remaining_depth: usize,
) -> Result<UiMountedCanonicalBox, String> {
    if let Some(bounds) = resolved.get(&instance) {
        return Ok(*bounds);
    }
    if remaining_depth == 0 {
        return Err("native-layout-occurrence-cycle".into());
    }
    let occurrence = occurrence_index
        .get(&instance)
        .ok_or_else(|| "native-layout-region-owner-missing".to_owned())?;
    let bounds = match occurrence.parent() {
        None => occurrence.bounds(),
        Some(parent) => {
            let parent =
                resolve_host_bounds(parent, occurrence_index, resolved, remaining_depth - 1)?;
            let local = occurrence.bounds();
            canonical_box(
                parent.x() + local.x(),
                parent.y() + local.y(),
                local.width(),
                local.height(),
                UiMountedCoordinateSpace::HostSurface,
            )?
        }
    };
    resolved.insert(instance, bounds);
    Ok(bounds)
}

fn canonical_box(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    coordinate_space: UiMountedCoordinateSpace,
) -> Result<UiMountedCanonicalBox, String> {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x,
        y,
        width,
        height,
        coordinate_space,
    })
    .map_err(|denial| format!("native-layout-geometry:{denial:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_ui::facade::declaration::ComponentViewportRegion;

    #[test]
    fn resized_surface_and_portal_local_contracts_keep_their_coordinate_owners() {
        let viewport = canonical_box(
            0.0,
            0.0,
            1_120.0,
            700.0,
            UiMountedCoordinateSpace::HostSurface,
        )
        .unwrap();
        let service =
            ComponentAllocationMeasurementContract::viewport_region(ComponentViewportRegion::new(
                ComponentViewportAxisPlacement::stretch_between(264, 24),
                ComponentViewportAxisPlacement::stretch_between(104, 72),
            ));
        let service =
            resolve_allocation(service, viewport, UiMountedCoordinateSpace::HostSurface).unwrap();
        assert_eq!(
            [service.x(), service.y(), service.width(), service.height()],
            [264.0, 104.0, 832.0, 524.0]
        );

        let portal =
            ComponentAllocationMeasurementContract::viewport_region(ComponentViewportRegion::new(
                ComponentViewportAxisPlacement::fixed_from_start(24, 104).unwrap(),
                ComponentViewportAxisPlacement::fixed_from_start(248, 40).unwrap(),
            ));
        let portal =
            resolve_allocation(portal, viewport, UiMountedCoordinateSpace::GraphNodeLocal).unwrap();
        assert_eq!(
            portal.coordinate_space(),
            UiMountedCoordinateSpace::GraphNodeLocal
        );
        assert_eq!(
            [portal.x(), portal.y(), portal.width(), portal.height()],
            [24.0, 248.0, 104.0, 40.0]
        );
        let service_region = region_bounds(
            PlatformPulseMosaicRegion::ServiceStage.id(),
            viewport,
            service,
        )
        .unwrap();
        assert_eq!(
            [
                service_region.x(),
                service_region.y(),
                service_region.width(),
                service_region.height()
            ],
            [0.0, 0.0, 832.0, 524.0]
        );
        assert_eq!(service.x() + service_region.x(), 264.0);
        assert_eq!(service.y() + service_region.y(), 104.0);
    }
}

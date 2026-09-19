use crate::capability::{ComponentAllocationMeasurementContract, ComponentViewportAxisPlacement};
use crate::facade::WorthUiNativeApplicationShell;
use crate::mounting::{UiMountedOccurrenceGeometry, UiMountedSurfaceGeometryBatch};
use worth_ui_host_contract::{
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
};

/// Completes the exact occurrence geometry for the declarative frame-program
/// driver. Applications with a live runtime remain responsible for their own
/// layout; this owner covers the bounded program path used without one.
pub(super) fn complete_program_layout(shell: &mut WorthUiNativeApplicationShell) -> Result<(), ()> {
    if !shell.native_program_layout_required() {
        return Ok(());
    }
    let viewport = shell.native_layout_viewport().ok_or(())?;
    if !shell.native_region_layout_inputs().is_empty() {
        return Err(());
    }
    let occurrences = shell
        .native_component_layout_inputs()
        .iter()
        .map(|component| {
            let contract = component.allocation().ok_or(())?;
            let coordinate_space = if component.portal_parent().is_some() {
                UiMountedCoordinateSpace::GraphNodeLocal
            } else {
                UiMountedCoordinateSpace::HostSurface
            };
            let bounds = resolve_allocation(contract, viewport, coordinate_space)?;
            Ok(match component.portal_parent() {
                Some(parent) => UiMountedOccurrenceGeometry::parent_relative(
                    component.instance(),
                    parent,
                    bounds,
                ),
                None => UiMountedOccurrenceGeometry::surface(component.instance(), bounds),
            })
        })
        .collect::<Result<Vec<_>, ()>>()?;
    let basis = shell.native_layout_basis().map_err(|_| ())?;
    let revision = shell.next_native_layout_revision().map_err(|_| ())?;
    shell
        .complete_native_layout(UiMountedSurfaceGeometryBatch::new(
            basis,
            revision,
            viewport,
            occurrences,
        ))
        .map_err(|_| ())?;
    Ok(())
}

fn resolve_allocation(
    contract: ComponentAllocationMeasurementContract,
    viewport: UiMountedCanonicalBox,
    coordinate_space: UiMountedCoordinateSpace,
) -> Result<UiMountedCanonicalBox, ()> {
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
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x,
        y,
        width,
        height,
        coordinate_space,
    })
    .map_err(|_| ())
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

#[cfg(test)]
mod tests {
    use super::complete_program_layout;
    use crate::certification_support::ScriptedPresentationHost;
    use crate::runtime::tests::active_application_session_test_support::source_backed_component_app_with_host_and_viewport_allocation;

    #[test]
    fn same_binding_viewport_resize_recompletes_program_owned_geometry() {
        let host = ScriptedPresentationHost::native_display();
        let mut shell = source_backed_component_app_with_host_and_viewport_allocation(host)
            .launch_native_surface()
            .expect("native viewport shell should launch");
        shell.observe_native_viewport_readiness([800, 600], 1_000, false);
        assert!(shell.native_program_layout_required());
        complete_program_layout(&mut shell).expect("initial program layout should complete");
        assert!(!shell.native_program_layout_required());

        shell.observe_native_viewport_readiness([960, 720], 1_000, false);
        assert!(
            shell.native_program_layout_required(),
            "same-binding viewport change invalidates program-owned geometry"
        );
        complete_program_layout(&mut shell).expect("resized program layout should complete");
        assert!(!shell.native_program_layout_required());
    }
}

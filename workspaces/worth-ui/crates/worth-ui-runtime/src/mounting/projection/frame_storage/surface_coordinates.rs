//! Project mounted logical surface geometry into the client viewport.
use super::UiMountedProjectionDenial;

impl super::UiMountedProjectionFrame {
    pub(in crate::mounting) fn scroll_sample_viewport_bounds(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        bounds: worth_ui_host_contract::UiMountedCanonicalBox,
    ) -> Result<worth_ui_host_contract::UiMountedCanonicalBox, UiMountedProjectionDenial> {
        let surface = self
            .semantic
            .surface_for(surface)
            .ok_or(UiMountedProjectionDenial::CoordinateBasisMismatch)?;
        viewport_bounds(bounds, surface.coordinate_posture)
    }
}

pub(in crate::mounting::projection) fn viewport_allocation(
    allocation: worth_ui_host_contract::UiMountedAllocationProjection,
    posture: crate::mounting::UiSurfaceBindingCoordinatePosture,
) -> Result<worth_ui_host_contract::UiMountedAllocationProjection, UiMountedProjectionDenial> {
    use worth_ui_host_contract::UiMountedAllocationProjection as Allocation;
    Ok(match allocation {
        Allocation::Known { bounds, basis } => Allocation::Known {
            bounds: viewport_bounds(bounds, posture)?,
            basis,
        },
        Allocation::PortalAnchorObservation { bounds, basis } => {
            Allocation::PortalAnchorObservation {
                bounds: viewport_bounds(bounds, posture)?,
                basis,
            }
        }
        Allocation::Omitted(reason) => Allocation::Omitted(reason),
    })
}

/// The one surface-to-viewport crossing for committed boxes: a box in a
/// logical-point surface's layout is the same box in the client viewport.
pub(in crate::mounting) fn viewport_bounds(
    bounds: worth_ui_host_contract::UiMountedCanonicalBox,
    posture: crate::mounting::UiSurfaceBindingCoordinatePosture,
) -> Result<worth_ui_host_contract::UiMountedCanonicalBox, UiMountedProjectionDenial> {
    use worth_ui_host_contract::{
        UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
    };
    if posture != crate::mounting::UiSurfaceBindingCoordinatePosture::LogicalPoints {
        return Err(UiMountedProjectionDenial::CoordinateBasisMismatch);
    }
    match bounds.coordinate_space() {
        UiMountedCoordinateSpace::Viewport => Ok(bounds),
        UiMountedCoordinateSpace::HostSurface => {
            // Mounted surface layout and host viewport input share the client
            // origin and logical-point units. A layout viewport is a coverage
            // rectangle, not an additional origin. Native input already removes DPI.
            UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
                x: bounds.x(),
                y: bounds.y(),
                width: bounds.width(),
                height: bounds.height(),
                coordinate_space: UiMountedCoordinateSpace::Viewport,
            })
            .map_err(|denial| match denial {
                worth_ui_host_contract::UiMountedGeometryDenial::NonFinite => {
                    UiMountedProjectionDenial::NonFiniteGeometry
                }
                worth_ui_host_contract::UiMountedGeometryDenial::NegativeExtent => {
                    UiMountedProjectionDenial::NegativeExtent
                }
            })
        }
        _ => Err(UiMountedProjectionDenial::CoordinateBasisMismatch),
    }
}

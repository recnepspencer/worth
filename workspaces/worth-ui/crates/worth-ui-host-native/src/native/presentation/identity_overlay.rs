use worth_ui_host_contract::{
    UiHostSurfacePresentationDenial, UiMountedDiagnosticProjection,
    UiMountedIdentityOverlayMechanic, UiMountedPresentationDelta, UiMountedProjectionView,
};

#[derive(Clone, Copy, PartialEq)]
pub(super) enum UiNativeRetainedIdentityOverlay {
    Absent,
    Retained {
        target: worth_ui_host_contract::UiMountedInstanceIdentity,
        mechanic: UiMountedIdentityOverlayMechanic,
    },
}

impl UiNativeRetainedIdentityOverlay {
    const fn target(self) -> Option<worth_ui_host_contract::UiMountedInstanceIdentity> {
        match self {
            Self::Retained { target, .. } => Some(target),
            Self::Absent => None,
        }
    }

    const fn mechanic(self) -> Option<UiMountedIdentityOverlayMechanic> {
        match self {
            Self::Retained { mechanic, .. } => Some(mechanic),
            Self::Absent => None,
        }
    }

    pub(super) fn prepare(
        projection: &UiMountedProjectionView,
    ) -> Result<Self, UiHostSurfacePresentationDenial> {
        let mechanics = projection
            .nodes()
            .iter()
            .filter_map(|node| match node.diagnostic() {
                UiMountedDiagnosticProjection::IdentityOverlay(mechanic) => Some((node, mechanic)),
                _ => None,
            })
            .collect::<Vec<_>>();
        if mechanics.len() > 1 {
            return Err(malformed());
        }
        let Some((node, mechanic)) = mechanics.first() else {
            return Ok(Self::Absent);
        };
        validate(
            projection.frame(),
            projection.surface(),
            projection.binding(),
            node.mounted_instance(),
            *mechanic,
        )?;
        Ok(Self::Retained {
            target: node.mounted_instance(),
            mechanic: *mechanic,
        })
    }

    pub(super) fn apply_delta(
        &mut self,
        delta: &UiMountedPresentationDelta,
    ) -> Result<bool, UiHostSurfacePresentationDenial> {
        let predecessor = *self;
        let touched = |target| {
            delta
                .nodes()
                .iter()
                .any(|change| change.mounted_instance() == target)
        };
        let retained = self.target().filter(|target| !touched(*target));
        let mut replacement = None;
        for change in delta.nodes() {
            let worth_ui_host_contract::UiMountedPresentationNodeChange::Upsert(state) = change
            else {
                continue;
            };
            let UiMountedDiagnosticProjection::IdentityOverlay(mechanic) = state.diagnostic()
            else {
                continue;
            };
            if replacement.replace((*state, mechanic)).is_some() {
                return Err(malformed());
            }
        }
        if retained.is_some() && replacement.is_some() {
            return Err(malformed());
        }
        let Some((state, mechanic)) = replacement else {
            if retained.is_none() {
                *self = Self::Absent;
            }
            return Ok(*self != predecessor);
        };
        validate(
            delta.affinity().successor(),
            delta.affinity().surface(),
            delta.affinity().binding(),
            state.mounted_instance(),
            mechanic,
        )?;
        *self = Self::Retained {
            target: state.mounted_instance(),
            mechanic,
        };
        Ok(*self != predecessor)
    }

    pub(super) const fn is_active(self) -> bool {
        matches!(self, Self::Retained { .. })
    }

    pub(super) fn raster_operations(
        self,
        basis: super::raster::UiNativeRasterBasis,
    ) -> Result<Vec<super::UiNativeRasterOperation>, UiHostSurfacePresentationDenial> {
        let Self::Retained { mechanic, .. } = self else {
            return Ok(Vec::new());
        };
        validate_coordinate_basis(mechanic, basis)?;
        let target = mechanic.target_region();
        let width = u32::from(mechanic.border_width());
        let left = target.left();
        let top = target.top();
        let right = target.right();
        let bottom = target.bottom();
        let strips = [
            [left, top, right, top.saturating_add(width).min(bottom)],
            [left, bottom.saturating_sub(width).max(top), right, bottom],
            [left, top, left.saturating_add(width).min(right), bottom],
            [right.saturating_sub(width).max(left), top, right, bottom],
        ];
        strips
            .into_iter()
            .map(|bounds| {
                let rect = super::raster::raster_physical_bounds(bounds, basis.extent())
                    .ok_or_else(malformed)?;
                Ok(super::UiNativeRasterOperation::FilledRect {
                    rect,
                    source_rgba8: mechanic.color().channels(),
                })
            })
            .collect()
    }

    pub(super) fn transition_damage(
        predecessor: Self,
        successor: Self,
    ) -> Result<Vec<worth_ui_host_contract::UiMountedLogicalDamage>, UiHostSurfacePresentationDenial>
    {
        if predecessor == successor {
            return Ok(Vec::new());
        }
        [predecessor.mechanic(), successor.mechanic()]
            .into_iter()
            .flatten()
            .map(logical_damage)
            .collect()
    }
}

fn logical_damage(
    mechanic: UiMountedIdentityOverlayMechanic,
) -> Result<worth_ui_host_contract::UiMountedLogicalDamage, UiHostSurfacePresentationDenial> {
    let target = mechanic.target_region();
    let scale = mechanic.coordinate_basis().scale();
    let left = target.left() as f32 / scale[0];
    let top = target.top() as f32 / scale[1];
    let right = target.right() as f32 / scale[0];
    let bottom = target.bottom() as f32 / scale[1];
    worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
        worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
            coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace::Viewport,
        },
    )
    .map(worth_ui_host_contract::UiMountedLogicalDamage::from_runtime_mounting)
    .map_err(|_| malformed())
}

fn validate(
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    mechanic: UiMountedIdentityOverlayMechanic,
) -> Result<(), UiHostSurfacePresentationDenial> {
    if mechanic.successor_frame() != frame
        || mechanic.surface() != surface
        || mechanic.binding() != binding
        || mechanic.target_receipt().mounted_instance() != mounted_instance
    {
        return Err(malformed());
    }
    Ok(())
}

fn validate_coordinate_basis(
    mechanic: UiMountedIdentityOverlayMechanic,
    basis: super::raster::UiNativeRasterBasis,
) -> Result<(), UiHostSurfacePresentationDenial> {
    let coordinates = mechanic.coordinate_basis();
    let extent = basis.extent();
    let scale = coordinates.scale();
    let viewport = coordinates.viewport_logical_dimensions();
    let target = mechanic.target_region();
    if coordinates.client_physical_dimensions() != extent
        || coordinates.orientation()
            != worth_ui_host_contract::UiHostCoordinateOrientation::TopLeftOrigin
        || coordinates.rounding()
            != worth_ui_host_contract::UiHostCoordinateRounding::PixelCenterNearest
        || coordinates.translation() != [0.0, 0.0]
        || scale != [basis.scale_factor(), basis.scale_factor()]
        || viewport[0].mul_add(scale[0], 0.0).round() as u32 != extent[0]
        || viewport[1].mul_add(scale[1], 0.0).round() as u32 != extent[1]
        || target.right() > extent[0]
        || target.bottom() > extent[1]
    {
        return Err(malformed());
    }
    Ok(())
}

const fn malformed() -> UiHostSurfacePresentationDenial {
    UiHostSurfacePresentationDenial::MalformedProjection
}

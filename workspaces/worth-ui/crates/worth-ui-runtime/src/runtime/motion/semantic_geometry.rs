#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiMotionSemanticGeometry {
    components: [u32; 4],
    coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiMotionGeometryDenial {
    NonFinite,
    NegativeExtent,
}

impl UiMotionSemanticGeometry {
    pub(crate) fn from_committed_components(
        components: [f32; 4],
        coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace,
    ) -> Result<Self, UiMotionGeometryDenial> {
        if components.iter().any(|component| !component.is_finite()) {
            return Err(UiMotionGeometryDenial::NonFinite);
        }
        if components[2] < 0.0 || components[3] < 0.0 {
            return Err(UiMotionGeometryDenial::NegativeExtent);
        }
        Ok(Self {
            components: components.map(f32::to_bits),
            coordinate_space,
        })
    }

    pub(crate) fn from_committed_box(
        bounds: worth_ui_host_contract::UiMountedCanonicalBox,
    ) -> Self {
        Self {
            components: [bounds.x(), bounds.y(), bounds.width(), bounds.height()].map(f32::to_bits),
            coordinate_space: bounds.coordinate_space(),
        }
    }

    pub(crate) fn components(self) -> [f32; 4] {
        self.components.map(f32::from_bits)
    }

    pub(crate) const fn coordinate_space(self) -> worth_ui_host_contract::UiMountedCoordinateSpace {
        self.coordinate_space
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_is_branded_only_after_finite_non_negative_validation() {
        assert_eq!(
            UiMotionSemanticGeometry::from_committed_components(
                [0.0, 0.0, -1.0, 1.0],
                worth_ui_host_contract::UiMountedCoordinateSpace::HostSurface,
            ),
            Err(UiMotionGeometryDenial::NegativeExtent)
        );
        assert_eq!(
            UiMotionSemanticGeometry::from_committed_components(
                [f32::NAN, 0.0, 1.0, 1.0],
                worth_ui_host_contract::UiMountedCoordinateSpace::HostSurface,
            ),
            Err(UiMotionGeometryDenial::NonFinite)
        );
        assert_eq!(
            UiMotionSemanticGeometry::from_committed_components(
                [1.0, 2.0, 3.0, 4.0],
                worth_ui_host_contract::UiMountedCoordinateSpace::Viewport,
            )
            .unwrap()
            .components(),
            [1.0, 2.0, 3.0, 4.0]
        );
        assert_eq!(
            UiMotionSemanticGeometry::from_committed_components(
                [1.0, 2.0, 3.0, 4.0],
                worth_ui_host_contract::UiMountedCoordinateSpace::Viewport,
            )
            .unwrap()
            .coordinate_space(),
            worth_ui_host_contract::UiMountedCoordinateSpace::Viewport
        );
    }
}

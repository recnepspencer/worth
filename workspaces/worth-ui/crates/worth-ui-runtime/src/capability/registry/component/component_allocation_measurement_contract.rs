#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentAllocationMeasurementContract {
    FillViewport,
    ViewportInset(super::ComponentViewportInset),
    ViewportRegion(super::ComponentViewportRegion),
    FixedLogicalSize {
        width: u16,
        height: u16,
    },
    /// A region of the cell that the component's layout container assigns
    /// it. The cell, not the viewport, is the region's reference box.
    LayoutCell(super::ComponentViewportRegion),
}

impl ComponentAllocationMeasurementContract {
    pub const fn fill_viewport() -> Self {
        Self::FillViewport
    }

    pub const fn viewport_inset(inset: super::ComponentViewportInset) -> Self {
        Self::ViewportInset(inset)
    }

    pub const fn viewport_region(region: super::ComponentViewportRegion) -> Self {
        Self::ViewportRegion(region)
    }

    pub fn fixed_logical_size(width: u16, height: u16) -> Option<Self> {
        (width != 0 && height != 0).then_some(Self::FixedLogicalSize { width, height })
    }

    pub const fn layout_cell(region: super::ComponentViewportRegion) -> Self {
        Self::LayoutCell(region)
    }

    /// The whole cell that the component's layout container assigns it.
    pub const fn fill_layout_cell() -> Self {
        Self::LayoutCell(super::ComponentViewportRegion::new(
            super::ComponentViewportAxisPlacement::stretch_between(0, 0),
            super::ComponentViewportAxisPlacement::stretch_between(0, 0),
        ))
    }

    pub(crate) fn digest_basis(self) -> String {
        match self {
            Self::FillViewport => "fill-viewport".to_owned(),
            Self::ViewportInset(inset) => inset.digest_basis(),
            Self::ViewportRegion(region) => region.digest_basis(),
            Self::FixedLogicalSize { width, height } => {
                format!("fixed-logical-size:{width}:{height}")
            }
            Self::LayoutCell(region) => format!("layout-cell:{}", region.digest_basis()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ComponentAllocationMeasurementContract;
    use crate::capability::{
        ComponentViewportAxisPlacement, ComponentViewportInset, ComponentViewportRegion,
    };

    #[test]
    fn viewport_inset_digest_preserves_both_axes_and_differs_from_fill() {
        let fill = ComponentAllocationMeasurementContract::fill_viewport();
        let inset = ComponentAllocationMeasurementContract::viewport_inset(
            ComponentViewportInset::symmetric(48, 24),
        );
        let changed = ComponentAllocationMeasurementContract::viewport_inset(
            ComponentViewportInset::symmetric(48, 23),
        );

        assert_ne!(fill.digest_basis(), inset.digest_basis());
        assert_ne!(inset.digest_basis(), changed.digest_basis());
        assert_eq!(inset.digest_basis(), "viewport-inset:48:24");
        let region =
            ComponentAllocationMeasurementContract::viewport_region(ComponentViewportRegion::new(
                ComponentViewportAxisPlacement::fixed_from_start(24, 216).unwrap(),
                ComponentViewportAxisPlacement::stretch_between(104, 72),
            ));
        assert_eq!(
            region.digest_basis(),
            "viewport-region:fixed-from-start:24:216:stretch-between:104:72"
        );
        assert!(ComponentAllocationMeasurementContract::fixed_logical_size(0, 24).is_none());
        assert_eq!(
            ComponentAllocationMeasurementContract::fixed_logical_size(160, 24)
                .unwrap()
                .digest_basis(),
            "fixed-logical-size:160:24"
        );
        assert_eq!(
            ComponentAllocationMeasurementContract::fill_layout_cell().digest_basis(),
            "layout-cell:viewport-region:stretch-between:0:0:stretch-between:0:0"
        );
    }
}

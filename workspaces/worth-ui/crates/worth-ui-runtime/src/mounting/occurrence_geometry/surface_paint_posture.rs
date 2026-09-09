#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiMountedCanonicalBorderOmission {
    side: worth_ui_host_contract::UiMountedSurfaceBorderSide,
    start: f32,
    end: f32,
}

impl UiMountedCanonicalBorderOmission {
    pub(crate) const fn new(
        side: worth_ui_host_contract::UiMountedSurfaceBorderSide,
        start: f32,
        end: f32,
    ) -> Self {
        Self { side, start, end }
    }

    pub(crate) const fn side(self) -> worth_ui_host_contract::UiMountedSurfaceBorderSide {
        self.side
    }

    pub(crate) const fn start(self) -> f32 {
        self.start
    }

    pub(crate) const fn end(self) -> f32 {
        self.end
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiMountedSurfacePaintPosture {
    border_edges: worth_ui_host_contract::UiMountedSurfaceBorderEdges,
    border_omissions: Box<[UiMountedCanonicalBorderOmission]>,
    exterior_corners: [bool; 4],
}

impl UiMountedSurfacePaintPosture {
    pub(crate) fn ordinary() -> Self {
        Self {
            border_edges: worth_ui_host_contract::UiMountedSurfaceBorderEdges::ALL,
            border_omissions: Box::new([]),
            exterior_corners: [true; 4],
        }
    }

    pub(crate) fn mosaic(
        border_edges: worth_ui_host_contract::UiMountedSurfaceBorderEdges,
        border_omissions: Box<[UiMountedCanonicalBorderOmission]>,
        exterior_corners: [bool; 4],
    ) -> Self {
        Self {
            border_edges,
            border_omissions,
            exterior_corners,
        }
    }

    pub(crate) const fn border_edges(&self) -> worth_ui_host_contract::UiMountedSurfaceBorderEdges {
        self.border_edges
    }

    pub(crate) fn border_omissions(&self) -> &[UiMountedCanonicalBorderOmission] {
        &self.border_omissions
    }

    pub(crate) const fn exterior_corners(&self) -> [bool; 4] {
        self.exterior_corners
    }
}

impl Default for UiMountedSurfacePaintPosture {
    fn default() -> Self {
        Self::ordinary()
    }
}

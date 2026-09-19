use worth_ui_host_contract::{
    UiMountedAppearanceMechanic, UiMountedCanonicalBox, UiMountedCanonicalBoxInput,
    UiMountedCoordinateSpace, UiMountedPresentationOpacity, UiMountedSurfacePaint,
    UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
    UI_APPEARANCE_LOGICAL_SUBPIXELS_PER_POINT,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedRetainedAppearanceVisualMechanic {
    semantic_surface: UiSemanticSurfaceIdentity,
    mechanic: UiMountedAppearanceMechanic,
    semantic_digest: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiMountedAppearancePaintBasis {
    binding: UiSurfaceBindingGeneration,
    node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    bounds: UiMountedCanonicalBox,
    clip: UiMountedCanonicalBox,
    semantic_order: u32,
    alpha: Option<u8>,
    source_digest: u64,
}

impl UiMountedRetainedAppearanceVisualMechanic {
    pub(crate) const fn new(
        semantic_surface: UiSemanticSurfaceIdentity,
        mechanic: UiMountedAppearanceMechanic,
        semantic_digest: u64,
    ) -> Self {
        Self {
            semantic_surface,
            mechanic,
            semantic_digest,
        }
    }

    pub(crate) fn into_paint_basis(
        self,
        binding: UiSurfaceBindingGeneration,
    ) -> Option<UiMountedAppearancePaintBasis> {
        let surface = match &self.mechanic {
            UiMountedAppearanceMechanic::Surface(surface) => surface,
            UiMountedAppearanceMechanic::PortalSurface(portal) => portal.surface(),
            UiMountedAppearanceMechanic::Outline(_)
            | UiMountedAppearanceMechanic::TextForeground(_)
            | UiMountedAppearanceMechanic::Pointer(_)
            | UiMountedAppearanceMechanic::Backdrop(_) => return None,
        };
        let (paint_alpha, uniform_fill) = match surface.paint() {
            UiMountedSurfacePaint::Fill(fill) => (
                fill.colors()
                    .map(|color| color.straight_srgba()[3])
                    .into_iter()
                    .max()
                    .unwrap(),
                matches!(fill, worth_ui_host_contract::UiMountedSurfaceFill::Solid(_)),
            ),
            UiMountedSurfacePaint::FillAndBorder { fill, border, .. } => (
                fill.colors()
                    .map(|color| color.straight_srgba()[3])
                    .into_iter()
                    .max()
                    .unwrap()
                    .max(border.straight_srgba()[3]),
                false,
            ),
            UiMountedSurfacePaint::Border { color, .. } => (color.straight_srgba()[3], false),
        };
        let alpha = composed_alpha(paint_alpha, surface.opacity());
        // The existing inspection index can prove uniform rectangular coverage.
        // Rounded corners and borders need shape-aware adjudication; retaining
        // their bounds as unsupported must not turn transparent pixels opaque.
        let rectangular_coverage = uniform_fill
            && surface.radii().corners() == [0; 4]
            && surface.portal_group().is_none()
            && matches!(self.mechanic, UiMountedAppearanceMechanic::Surface(_));
        (alpha != 0).then_some(UiMountedAppearancePaintBasis {
            binding,
            node_receipt: surface.node_receipt(),
            bounds: canonical_visual(surface.visual_bounds())?,
            clip: canonical_clip(surface.clip())?,
            semantic_order: surface.surface_paint_order(),
            alpha: rectangular_coverage.then_some(alpha),
            source_digest: self.semantic_digest,
        })
    }

    pub(crate) const fn semantic_surface(&self) -> UiSemanticSurfaceIdentity {
        self.semantic_surface
    }
}

impl UiMountedAppearancePaintBasis {
    pub(crate) const fn binding(self) -> UiSurfaceBindingGeneration {
        self.binding
    }

    pub(crate) const fn node_receipt(self) -> worth_ui_host_contract::UiMountedNodeReceiptIdentity {
        self.node_receipt
    }

    pub(crate) const fn bounds(self) -> UiMountedCanonicalBox {
        self.bounds
    }

    pub(crate) const fn clip(self) -> UiMountedCanonicalBox {
        self.clip
    }

    pub(crate) const fn semantic_order(self) -> u32 {
        self.semantic_order
    }

    pub(crate) const fn alpha(self) -> Option<u8> {
        self.alpha
    }

    pub(crate) const fn source_digest(self) -> u64 {
        self.source_digest
    }

    pub(crate) fn reattribute(
        mut self,
        receipts: &super::UiMountedNodeReceiptBasis,
    ) -> Option<Self> {
        self.node_receipt = receipts.receipt_for(self.node_receipt.mounted_instance())?;
        Some(self)
    }
}

fn canonical_visual(
    bounds: worth_ui_host_contract::UiAppearanceVisualBounds,
) -> Option<UiMountedCanonicalBox> {
    canonical(bounds.x(), bounds.y(), bounds.width(), bounds.height())
}

fn canonical_clip(clip: worth_ui_host_contract::UiAppearanceClip) -> Option<UiMountedCanonicalBox> {
    canonical(clip.x(), clip.y(), clip.width(), clip.height())
}

fn canonical(x: i32, y: i32, width: u32, height: u32) -> Option<UiMountedCanonicalBox> {
    let units = UI_APPEARANCE_LOGICAL_SUBPIXELS_PER_POINT as f32;
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: x as f32 / units,
        y: y as f32 / units,
        width: width as f32 / units,
        height: height as f32 / units,
        coordinate_space: UiMountedCoordinateSpace::Viewport,
    })
    .ok()
}

fn composed_alpha(color_alpha: u8, opacity: UiMountedPresentationOpacity) -> u8 {
    let product = u32::from(color_alpha) * u32::from(opacity.units());
    ((product + u32::from(u16::MAX) / 2) / u32::from(u16::MAX)) as u8
}

#[cfg(test)]
#[path = "visual_region_appearance_tests.rs"]
mod tests;

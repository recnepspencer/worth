#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiMountedSurfacePaint {
    Fill(super::UiMountedAppearanceColor),
    Border {
        color: super::UiMountedAppearanceColor,
        inward_width: super::UiAppearanceLogicalLength,
    },
    FillAndBorder {
        fill: super::UiMountedAppearanceColor,
        border: super::UiMountedAppearanceColor,
        inward_width: super::UiAppearanceLogicalLength,
    },
}

/// Exact allocation edges on which an inward surface border may paint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiMountedSurfaceBorderEdges(u8);

impl UiMountedSurfaceBorderEdges {
    const TOP: u8 = 1;
    const RIGHT: u8 = 2;
    const BOTTOM: u8 = 4;
    const LEFT: u8 = 8;

    pub const ALL: Self = Self(Self::TOP | Self::RIGHT | Self::BOTTOM | Self::LEFT);

    #[doc(hidden)]
    pub const fn from_runtime_mosaic(top: bool, right: bool, bottom: bool, left: bool) -> Self {
        Self(
            (if top { Self::TOP } else { 0 })
                | (if right { Self::RIGHT } else { 0 })
                | (if bottom { Self::BOTTOM } else { 0 })
                | (if left { Self::LEFT } else { 0 }),
        )
    }

    pub const fn top(self) -> bool {
        self.0 & Self::TOP != 0
    }
    pub const fn right(self) -> bool {
        self.0 & Self::RIGHT != 0
    }
    pub const fn bottom(self) -> bool {
        self.0 & Self::BOTTOM != 0
    }
    pub const fn left(self) -> bool {
        self.0 & Self::LEFT != 0
    }
    pub const fn any(self) -> bool {
        self.0 != 0
    }
}

impl Default for UiMountedSurfaceBorderEdges {
    fn default() -> Self {
        Self::ALL
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiMountedSurfaceAppearanceMechanic {
    node_receipt: crate::UiMountedNodeReceiptIdentity,
    bounds: super::UiAppearanceAllocationBounds,
    clip: super::UiAppearanceClip,
    surface_paint_order: u32,
    visual_bounds: super::UiAppearanceVisualBounds,
    radii: super::UiAppearanceNormalizedLogicalRadii,
    border_edges: UiMountedSurfaceBorderEdges,
    border_omissions: Box<[super::UiMountedSurfaceBorderOmission]>,
    paint: UiMountedSurfacePaint,
    opacity: crate::UiMountedPresentationOpacity,
    projection: super::UiMountedNodeAppearanceAttribution,
}

#[doc(hidden)]
pub struct UiMountedSurfaceAppearanceCompletionInput {
    pub issuer: crate::UiMountedNodeReceiptIssuer,
    pub node_receipt: crate::UiMountedNodeReceiptIdentity,
    pub bounds: super::UiAppearanceAllocationBounds,
    pub clip: super::UiAppearanceClip,
    pub surface_paint_order: u32,
    pub radii: super::UiAppearanceNormalizedLogicalRadii,
    pub border_edges: UiMountedSurfaceBorderEdges,
    pub border_omissions: Box<[super::UiMountedSurfaceBorderOmission]>,
    pub paint: UiMountedSurfacePaint,
    pub opacity: crate::UiMountedPresentationOpacity,
    pub projection: super::UiMountedNodeAppearanceAttribution,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedSurfaceAppearanceCompletionDenial {
    NodeReceiptFrameMismatch,
    ProjectionIssuerMismatch,
    RadiiAllocationMismatch,
    BorderWidthExceedsHalfMinimumDimension,
    BorderOmissionOutOfBounds,
}

impl UiMountedSurfaceAppearanceMechanic {
    #[doc(hidden)]
    pub fn reattribute_for_runtime_mounting(
        &self,
        issuer: crate::UiMountedNodeReceiptIssuer,
        node_receipt: crate::UiMountedNodeReceiptIdentity,
    ) -> Option<Self> {
        (node_receipt.frame() == issuer.frame_identity()
            && node_receipt.mounted_instance() == self.node_receipt.mounted_instance())
        .then(|| Self {
            node_receipt,
            bounds: self.bounds,
            clip: self.clip,
            surface_paint_order: self.surface_paint_order,
            visual_bounds: self.visual_bounds,
            radii: self.radii,
            border_edges: self.border_edges,
            border_omissions: self.border_omissions.clone(),
            paint: self.paint.clone(),
            opacity: self.opacity,
            projection: super::UiMountedNodeAppearanceAttribution::from_runtime_mounting(
                issuer,
                self.projection.identity(),
                self.projection.revision(),
            )
            .expect("accepted appearance attribution stays nonzero"),
        })
    }

    #[doc(hidden)]
    pub fn complete_from_runtime_mounting(
        input: UiMountedSurfaceAppearanceCompletionInput,
    ) -> Result<Self, UiMountedSurfaceAppearanceCompletionDenial> {
        if input.node_receipt.frame() != input.issuer.frame_identity() {
            return Err(UiMountedSurfaceAppearanceCompletionDenial::NodeReceiptFrameMismatch);
        }
        if !input.projection.matches_issuer(input.issuer) {
            return Err(UiMountedSurfaceAppearanceCompletionDenial::ProjectionIssuerMismatch);
        }
        if !input.radii.matches_allocation(input.bounds) {
            return Err(UiMountedSurfaceAppearanceCompletionDenial::RadiiAllocationMismatch);
        }
        let inward_width = match &input.paint {
            UiMountedSurfacePaint::Fill(_) => super::UiAppearanceLogicalLength::ZERO,
            UiMountedSurfacePaint::Border { inward_width, .. }
            | UiMountedSurfacePaint::FillAndBorder { inward_width, .. } => *inward_width,
        };
        if inward_width.subpixels() > input.bounds.width().min(input.bounds.height()) / 2 {
            return Err(
                UiMountedSurfaceAppearanceCompletionDenial::BorderWidthExceedsHalfMinimumDimension,
            );
        }
        if input.border_omissions.iter().any(|omission| {
            let extent = match omission.side() {
                super::UiMountedSurfaceBorderSide::Top
                | super::UiMountedSurfaceBorderSide::Bottom => input.bounds.width(),
                super::UiMountedSurfaceBorderSide::Right
                | super::UiMountedSurfaceBorderSide::Left => input.bounds.height(),
            };
            omission.end() > extent
        }) {
            return Err(UiMountedSurfaceAppearanceCompletionDenial::BorderOmissionOutOfBounds);
        }
        Ok(Self {
            node_receipt: input.node_receipt,
            bounds: input.bounds,
            clip: input.clip,
            surface_paint_order: input.surface_paint_order,
            visual_bounds: super::UiAppearanceVisualBounds::from_surface_allocation(input.bounds),
            radii: input.radii,
            border_edges: input.border_edges,
            border_omissions: input.border_omissions,
            paint: input.paint,
            opacity: input.opacity,
            projection: input.projection,
        })
    }

    pub const fn node_receipt(&self) -> crate::UiMountedNodeReceiptIdentity {
        self.node_receipt
    }
    pub const fn bounds(&self) -> super::UiAppearanceAllocationBounds {
        self.bounds
    }
    pub const fn clip(&self) -> super::UiAppearanceClip {
        self.clip
    }
    pub const fn surface_paint_order(&self) -> u32 {
        self.surface_paint_order
    }
    pub const fn visual_bounds(&self) -> super::UiAppearanceVisualBounds {
        self.visual_bounds
    }
    pub const fn radii(&self) -> super::UiAppearanceNormalizedLogicalRadii {
        self.radii
    }
    pub const fn border_edges(&self) -> UiMountedSurfaceBorderEdges {
        self.border_edges
    }
    pub fn border_omissions(&self) -> &[super::UiMountedSurfaceBorderOmission] {
        &self.border_omissions
    }
    pub const fn paint(&self) -> &UiMountedSurfacePaint {
        &self.paint
    }
    pub const fn opacity(&self) -> crate::UiMountedPresentationOpacity {
        self.opacity
    }
    pub const fn projection(&self) -> super::UiMountedNodeAppearanceAttribution {
        self.projection
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn length(value: i32) -> super::super::UiAppearanceLogicalLength {
        super::super::UiAppearanceLogicalLength::new(value).unwrap()
    }

    fn input(
        width: u32,
        height: u32,
        inward_width: i32,
    ) -> UiMountedSurfaceAppearanceCompletionInput {
        let frame = crate::UiMountedFrameIdentity::mint_unbound().unwrap();
        let issuer = crate::UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
        let instance = crate::UiMountedInstanceIdentity::mint_unbound().unwrap();
        let bounds = super::super::UiAppearanceAllocationBounds::new(0, 0, width, height).unwrap();
        UiMountedSurfaceAppearanceCompletionInput {
            issuer,
            node_receipt: issuer.receipt_for(instance),
            bounds,
            clip: super::super::UiAppearanceClip::new(0, 0, width, height).unwrap(),
            surface_paint_order: 0,
            radii: super::super::UiAppearanceNormalizedLogicalRadii::normalize(
                bounds,
                [super::super::UiAppearanceLogicalLength::ZERO; 4],
            ),
            border_edges: UiMountedSurfaceBorderEdges::ALL,
            border_omissions: Box::new([]),
            paint: UiMountedSurfacePaint::Border {
                color: super::super::UiMountedAppearanceColor::from_straight_srgba([0; 4]),
                inward_width: length(inward_width),
            },
            opacity: crate::UiMountedPresentationOpacity::from_runtime_composition(u16::MAX),
            projection: super::super::UiMountedNodeAppearanceAttribution::from_runtime_mounting(
                issuer, 1, 1,
            )
            .unwrap(),
        }
    }

    #[test]
    fn surface_completion_denies_border_wider_than_half_minimum_dimension() {
        assert!(
            UiMountedSurfaceAppearanceMechanic::complete_from_runtime_mounting(input(5, 9, 2))
                .is_ok()
        );
        assert_eq!(
            UiMountedSurfaceAppearanceMechanic::complete_from_runtime_mounting(input(5, 9, 3)),
            Err(UiMountedSurfaceAppearanceCompletionDenial::BorderWidthExceedsHalfMinimumDimension)
        );
    }

    #[test]
    fn surface_visual_bounds_are_derived_from_allocation() {
        let mechanic =
            UiMountedSurfaceAppearanceMechanic::complete_from_runtime_mounting(input(5, 9, 0))
                .unwrap();
        assert_eq!(
            mechanic.visual_bounds(),
            super::super::UiAppearanceVisualBounds::from_surface_allocation(mechanic.bounds())
        );
    }

    #[test]
    fn surface_completion_denies_radii_normalized_for_another_allocation() {
        let mut mismatched = input(5, 9, 0);
        let other = super::super::UiAppearanceAllocationBounds::new(0, 0, 50, 90).unwrap();
        mismatched.radii = super::super::UiAppearanceNormalizedLogicalRadii::normalize(
            other,
            [super::super::UiAppearanceLogicalLength::ZERO; 4],
        );
        assert_eq!(
            UiMountedSurfaceAppearanceMechanic::complete_from_runtime_mounting(mismatched),
            Err(UiMountedSurfaceAppearanceCompletionDenial::RadiiAllocationMismatch)
        );
    }

    #[test]
    fn surface_completion_denies_border_omission_beyond_its_side() {
        let mut surface = input(5, 9, 0);
        surface.border_omissions = Box::new([
            super::super::UiMountedSurfaceBorderOmission::from_runtime_mosaic(
                super::super::UiMountedSurfaceBorderSide::Top,
                4,
                6,
            )
            .unwrap(),
        ]);

        assert_eq!(
            UiMountedSurfaceAppearanceMechanic::complete_from_runtime_mounting(surface),
            Err(UiMountedSurfaceAppearanceCompletionDenial::BorderOmissionOutOfBounds)
        );
    }

    #[test]
    fn surface_completion_denies_outline_radii_with_an_offset_expanded_basis() {
        let mut surface = input(100, 100, 0);
        let surface_radii = super::super::UiAppearanceNormalizedLogicalRadii::normalize(
            surface.bounds,
            [length(50); 4],
        );
        let outline = super::super::UiAppearanceOutlineGeometry::admit(
            surface.bounds,
            surface_radii,
            length(1),
            length(1),
            super::super::UiAppearanceLogicalLength::ZERO,
        )
        .unwrap();
        surface.radii = outline.radii();

        assert_eq!(surface.radii.corners(), [51; 4]);
        assert_eq!(
            UiMountedSurfaceAppearanceMechanic::complete_from_runtime_mounting(surface),
            Err(UiMountedSurfaceAppearanceCompletionDenial::RadiiAllocationMismatch)
        );
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiMountedOutlineAppearanceMechanic {
    node_receipt: crate::UiMountedNodeReceiptIdentity,
    clip: super::UiAppearanceClip,
    surface_paint_order: u32,
    geometry: super::UiAppearanceOutlineGeometry,
    color: super::UiMountedAppearanceColor,
    opacity: crate::UiMountedPresentationOpacity,
    projection: super::UiMountedNodeAppearanceAttribution,
}

#[doc(hidden)]
pub struct UiMountedOutlineAppearanceCompletionInput {
    pub issuer: crate::UiMountedNodeReceiptIssuer,
    pub node_receipt: crate::UiMountedNodeReceiptIdentity,
    pub clip: super::UiAppearanceClip,
    pub surface_paint_order: u32,
    pub geometry: super::UiAppearanceOutlineGeometry,
    pub color: super::UiMountedAppearanceColor,
    pub opacity: crate::UiMountedPresentationOpacity,
    pub projection: super::UiMountedNodeAppearanceAttribution,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedOutlineAppearanceCompletionDenial {
    NodeReceiptFrameMismatch,
    ProjectionIssuerMismatch,
}

impl UiMountedOutlineAppearanceMechanic {
    #[doc(hidden)]
    pub fn complete_from_runtime_mounting(
        input: UiMountedOutlineAppearanceCompletionInput,
    ) -> Result<Self, UiMountedOutlineAppearanceCompletionDenial> {
        if input.node_receipt.frame() != input.issuer.frame_identity() {
            return Err(UiMountedOutlineAppearanceCompletionDenial::NodeReceiptFrameMismatch);
        }
        if !input.projection.matches_issuer(input.issuer) {
            return Err(UiMountedOutlineAppearanceCompletionDenial::ProjectionIssuerMismatch);
        }
        Ok(Self {
            node_receipt: input.node_receipt,
            clip: input.clip,
            surface_paint_order: input.surface_paint_order,
            geometry: input.geometry,
            color: input.color,
            opacity: input.opacity,
            projection: input.projection,
        })
    }
    pub const fn node_receipt(&self) -> crate::UiMountedNodeReceiptIdentity {
        self.node_receipt
    }
    pub const fn clip(&self) -> super::UiAppearanceClip {
        self.clip
    }
    pub const fn surface_paint_order(&self) -> u32 {
        self.surface_paint_order
    }
    pub const fn allocation(&self) -> super::UiAppearanceAllocationBounds {
        self.geometry.allocation()
    }
    pub const fn visual_bounds(&self) -> super::UiAppearanceVisualBounds {
        self.geometry.visual_bounds()
    }
    pub const fn color(&self) -> super::UiMountedAppearanceColor {
        self.color
    }
    pub const fn width(&self) -> super::UiAppearanceLogicalLength {
        self.geometry.width()
    }
    pub const fn offset(&self) -> super::UiAppearanceLogicalLength {
        self.geometry.offset()
    }
    pub const fn anti_alias_fringe(&self) -> super::UiAppearanceLogicalLength {
        self.geometry.anti_alias_fringe()
    }
    pub const fn radii(&self) -> super::UiAppearanceNormalizedLogicalRadii {
        self.geometry.radii()
    }
    pub const fn opacity(&self) -> crate::UiMountedPresentationOpacity {
        self.opacity
    }
    pub const fn projection(&self) -> super::UiMountedNodeAppearanceAttribution {
        self.projection
    }
    pub const fn participates_in_hit_testing(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outline_completion_preserves_admitted_geometry() {
        let frame = crate::UiMountedFrameIdentity::mint_unbound().unwrap();
        let issuer = crate::UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
        let allocation = super::super::UiAppearanceAllocationBounds::new(5, 6, 20, 30).unwrap();
        let radii = super::super::UiAppearanceNormalizedLogicalRadii::normalize(
            allocation,
            [super::super::UiAppearanceLogicalLength::ZERO; 4],
        );
        let length = |value| super::super::UiAppearanceLogicalLength::new(value).unwrap();
        let geometry = super::super::UiAppearanceOutlineGeometry::admit(
            allocation,
            radii,
            length(2),
            length(1),
            length(1),
        )
        .unwrap();
        let mechanic = UiMountedOutlineAppearanceMechanic::complete_from_runtime_mounting(
            UiMountedOutlineAppearanceCompletionInput {
                surface_paint_order: 0,
                issuer,
                node_receipt: issuer
                    .receipt_for(crate::UiMountedInstanceIdentity::mint_unbound().unwrap()),
                clip: super::super::UiAppearanceClip::new(0, 0, 100, 100).unwrap(),
                geometry,
                color: super::super::UiMountedAppearanceColor::from_straight_srgba([1, 2, 3, 4]),
                opacity: crate::UiMountedPresentationOpacity::from_runtime_composition(u16::MAX),
                projection:
                    super::super::UiMountedNodeAppearanceAttribution::from_runtime_mounting(
                        issuer, 1, 1,
                    )
                    .unwrap(),
            },
        )
        .unwrap();
        assert_eq!(mechanic.allocation(), allocation);
        assert_eq!(mechanic.visual_bounds(), geometry.visual_bounds());
        assert!(!mechanic.participates_in_hit_testing());
    }
}

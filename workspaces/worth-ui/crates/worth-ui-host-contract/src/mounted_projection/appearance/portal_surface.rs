#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiMountedPortalSurfaceAppearanceMechanic {
    portal_instance: crate::UiMountedInstanceIdentity,
    surface: super::UiMountedSurfaceAppearanceMechanic,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedPortalSurfaceAppearanceCompletionDenial {
    PortalInstanceMismatch,
}

impl UiMountedPortalSurfaceAppearanceMechanic {
    #[doc(hidden)]
    pub fn reattribute_for_runtime_mounting(
        &self,
        issuer: crate::UiMountedNodeReceiptIssuer,
        node_receipt: crate::UiMountedNodeReceiptIdentity,
    ) -> Option<Self> {
        Some(Self {
            portal_instance: self.portal_instance,
            surface: self
                .surface
                .reattribute_for_runtime_mounting(issuer, node_receipt)?,
        })
    }

    #[doc(hidden)]
    pub fn complete_from_runtime_mounting(
        portal_instance: crate::UiMountedInstanceIdentity,
        surface: super::UiMountedSurfaceAppearanceMechanic,
    ) -> Result<Self, UiMountedPortalSurfaceAppearanceCompletionDenial> {
        if surface.node_receipt().mounted_instance() != portal_instance {
            return Err(UiMountedPortalSurfaceAppearanceCompletionDenial::PortalInstanceMismatch);
        }
        Ok(Self {
            portal_instance,
            surface,
        })
    }
    pub const fn portal_instance(&self) -> crate::UiMountedInstanceIdentity {
        self.portal_instance
    }
    pub const fn surface(&self) -> &super::UiMountedSurfaceAppearanceMechanic {
        &self.surface
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portal_surface_completion_requires_the_surface_mounted_instance() {
        let frame = crate::UiMountedFrameIdentity::mint_unbound().unwrap();
        let issuer = crate::UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
        let surface_instance = crate::UiMountedInstanceIdentity::mint_unbound().unwrap();
        let bounds = super::super::UiAppearanceAllocationBounds::new(0, 0, 8, 8).unwrap();
        let surface =
            super::super::UiMountedSurfaceAppearanceMechanic::complete_from_runtime_mounting(
                super::super::UiMountedSurfaceAppearanceCompletionInput {
                    issuer,
                    node_receipt: issuer.receipt_for(surface_instance),
                    bounds,
                    clip: super::super::UiAppearanceClip::new(0, 0, 8, 8).unwrap(),
                    surface_paint_order: 0,
                    radii: super::super::UiAppearanceNormalizedLogicalRadii::normalize(
                        bounds,
                        [super::super::UiAppearanceLogicalLength::ZERO; 4],
                    ),
                    border_edges: super::super::UiMountedSurfaceBorderEdges::ALL,
                    border_omissions: Box::new([]),
                    paint: super::super::UiMountedSurfacePaint::Fill(
                        super::super::UiMountedAppearanceColor::from_straight_srgba([0; 4]),
                    ),
                    opacity: crate::UiMountedPresentationOpacity::from_runtime_composition(
                        u16::MAX,
                    ),
                    projection:
                        super::super::UiMountedNodeAppearanceAttribution::from_runtime_mounting(
                            issuer, 1, 1,
                        )
                        .unwrap(),
                },
            )
            .unwrap();
        assert_eq!(
            UiMountedPortalSurfaceAppearanceMechanic::complete_from_runtime_mounting(
                crate::UiMountedInstanceIdentity::mint_unbound().unwrap(),
                surface,
            ),
            Err(UiMountedPortalSurfaceAppearanceCompletionDenial::PortalInstanceMismatch)
        );
    }
}

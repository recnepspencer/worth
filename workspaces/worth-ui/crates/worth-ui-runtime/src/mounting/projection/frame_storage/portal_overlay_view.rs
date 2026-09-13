use worth_ui_host_contract::UiMountedPortalOverlayMechanic;

use super::{UiMountedProjectionDenial, UiMountedProjectionFrame, UiMountedProjectionSurface};

pub(super) struct UiMountedPortalOverlayViewRows {
    pub(super) rows: Vec<UiMountedPortalOverlayMechanic>,
}

impl UiMountedProjectionFrame {
    pub(super) fn portal_overlay_view_rows(
        &self,
        surface: UiMountedProjectionSurface,
    ) -> Result<UiMountedPortalOverlayViewRows, UiMountedProjectionDenial> {
        let mut rows = Vec::new();
        for input in self.portal_overlays.iter().copied() {
            if input.surface() != surface.surface {
                continue;
            }
            let owner = self
                .semantic
                .node(input.owner())
                .ok_or(UiMountedProjectionDenial::PortalOverlayOwnerMissing)?;
            if owner.receipt.semantic_surface() != surface.surface {
                return Err(UiMountedProjectionDenial::PortalOverlayOwnerMissing);
            }
            let receipt = self
                .receipt_basis
                .receipt_for(input.owner())
                .ok_or(UiMountedProjectionDenial::PortalOverlayOwnerMissing)?;
            rows.push(
                input
                    .mechanic_for(self.frame, surface.binding, receipt)
                    .map_err(UiMountedProjectionDenial::PortalOverlayCompletion)?,
            );
        }
        Ok(UiMountedPortalOverlayViewRows { rows })
    }

    pub(super) fn portal_overlay_visual_rows(&self) -> Vec<UiMountedPortalOverlayMechanic> {
        self.semantic
            .surfaces
            .iter()
            .map(|(_, surface)| *surface)
            .flat_map(|surface| {
                self.portal_overlay_view_rows(surface)
                    .expect("prepared Portal overlays retain exact mounted owners")
                    .rows
            })
            .collect()
    }
}

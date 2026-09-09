use super::portal_child_view::UiMountedPortalChildPresentation;
use super::{UiMountedProjectionDenial, UiMountedProjectionFrame};

impl UiMountedProjectionFrame {
    pub(super) fn complete_presented_hits(&mut self) -> Result<(), UiMountedProjectionDenial> {
        for instance in self.changed_instances.clone().iter().copied() {
            let row = self.presented_hit_row(instance)?;
            self.hit_index_work
                .merge(self.mechanics.presented_hits.replace_base(instance, row));
        }
        Ok(())
    }

    pub(super) fn reconstruct_presented_hits(&mut self) -> Result<(), UiMountedProjectionDenial> {
        self.mechanics.presented_hits = Default::default();
        let instances = self.semantic.mounted_instances().collect::<Vec<_>>();
        self.hit_index_work.reconstructed_rows += instances.len();
        for instance in instances {
            let row = self.presented_hit_row(instance)?;
            self.hit_index_work
                .merge(self.mechanics.presented_hits.replace_base(instance, row));
        }
        Ok(())
    }

    fn presented_hit_row(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Result<Option<crate::mounting::UiPresentedHitTestRow>, UiMountedProjectionDenial> {
        let Some(node) = self.semantic.node(instance) else {
            return Ok(None);
        };
        let Some(surface) = self.semantic.surface_for(node.receipt.semantic_surface()) else {
            return Ok(None);
        };
        let Some(row) = self.mechanics.hit_test_for_instance(
            instance,
            surface.surface,
            surface.binding,
            self.frame,
            &self.receipt_basis,
        )?
        else {
            return Ok(None);
        };
        let (row, portal) =
            match self.portal_child_presentation(instance, surface.surface, surface.binding)? {
                UiMountedPortalChildPresentation::Ordinary => (row, None),
                UiMountedPortalChildPresentation::Suppressed => return Ok(None),
                UiMountedPortalChildPresentation::Presented(portal) => {
                    let Some(row) = row
                        .presented_within_portal(portal)
                        .map_err(UiMountedProjectionDenial::HitTestCompletion)?
                    else {
                        return Ok(None);
                    };
                    (row, Some(portal))
                }
            };
        Ok(Some(crate::mounting::UiPresentedHitTestRow::from_mounted(
            crate::mounting::UiMountedHitTestPresentation::completed(
                row,
                portal,
                self.portal_overlays
                    .iter()
                    .any(|portal| portal.owner() == instance),
            ),
        )))
    }
}

use crate::runtime::portal::UiPortalStackOrdinal;
use worth_ui_host_contract::{
    UiMountedInstanceIdentity, UiMountedPortalInputShielding,
    UiMountedPortalOverlayLifecyclePosture, UiSurfaceBindingGeneration,
};

#[cfg(test)]
mod tests;

/// What a modal on one binding admits: its input floor, and every Visible
/// Portal at or above it. Two frames that agree on it shield the same rows.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::mounting) struct UiModalInputAdmission {
    floor: UiPortalStackOrdinal,
    portals: std::collections::BTreeSet<u64>,
}

impl super::UiMountedVisualRegionBasis {
    /// What the modal on `binding` admits, or `None` when no modal shields
    /// input there and every row admits it.
    pub(in crate::mounting) fn modal_input_admission(
        &self,
        binding: UiSurfaceBindingGeneration,
    ) -> Option<UiModalInputAdmission> {
        let floor = self.modal_input_floor(binding, &mut Default::default())?;
        let portals = self
            .portal_overlays
            .iter()
            .filter(|portal| {
                portal.binding() == binding
                    && portal.lifecycle() == UiMountedPortalOverlayLifecyclePosture::Visible
                    && self
                        .portal_input_order
                        .get(&portal.portal_identity())
                        .is_some_and(|ordinal| *ordinal >= floor)
            })
            .map(|portal| portal.portal_identity())
            .collect();
        Some(UiModalInputAdmission { floor, portals })
    }

    pub(in crate::mounting) fn modal_input_floor(
        &self,
        binding: UiSurfaceBindingGeneration,
        work: &mut crate::mounting::UiHitTestSpatialWork,
    ) -> Option<UiPortalStackOrdinal> {
        self.portal_overlays
            .iter()
            .filter_map(|portal| {
                work.portal_rows_considered += 1;
                (portal.binding() == binding
                    && portal.lifecycle() == UiMountedPortalOverlayLifecyclePosture::Visible
                    && portal.shielding() == UiMountedPortalInputShielding::ModalSurface)
                    .then(|| {
                        *self
                            .portal_input_order
                            .get(&portal.portal_identity())
                            .expect("mounted Portal carries its owner-issued input order")
                    })
            })
            .max()
    }

    pub(in crate::mounting) fn admits_modal_input(
        &self,
        instance: UiMountedInstanceIdentity,
        floor: Option<UiPortalStackOrdinal>,
    ) -> bool {
        let Some(floor) = floor else {
            return true;
        };
        // Membership and order come from accepted Portal projection, never
        // surface paint rank, opacity, or the location of the pointer.
        self.portal_children
            .get(&instance)
            .and_then(|member| member.as_ref())
            .is_some_and(|(portal, _)| {
                portal.lifecycle() == UiMountedPortalOverlayLifecyclePosture::Visible
                    && self
                        .portal_input_order
                        .get(&portal.portal_identity())
                        .is_some_and(|ordinal| *ordinal >= floor)
            })
    }
}

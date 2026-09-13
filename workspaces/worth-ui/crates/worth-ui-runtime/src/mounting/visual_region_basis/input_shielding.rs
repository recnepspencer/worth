use worth_ui_host_contract::{
    UiMountedInstanceIdentity, UiMountedPortalInputShielding,
    UiMountedPortalOverlayLifecyclePosture, UiSurfaceBindingGeneration,
};

impl super::UiMountedVisualRegionBasis {
    pub(in crate::mounting) fn modal_input_floor(
        &self,
        binding: UiSurfaceBindingGeneration,
        work: &mut crate::mounting::UiHitTestSpatialWork,
    ) -> Option<crate::runtime::portal::UiPortalStackOrdinal> {
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
        floor: Option<crate::runtime::portal::UiPortalStackOrdinal>,
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

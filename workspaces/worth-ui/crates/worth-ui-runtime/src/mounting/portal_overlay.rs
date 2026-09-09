#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedPortalOverlayProjectionInput {
    portal_identity: u64,
    owner: worth_ui_host_contract::UiMountedInstanceIdentity,
    placement: crate::runtime::portal::UiPreparedPortalPlacement,
    lifecycle: crate::runtime::portal::UiPortalLifecyclePosture,
}

impl UiMountedPortalOverlayProjectionInput {
    pub(crate) const fn new(
        portal_identity: u64,
        owner: worth_ui_host_contract::UiMountedInstanceIdentity,
        placement: crate::runtime::portal::UiPreparedPortalPlacement,
        lifecycle: crate::runtime::portal::UiPortalLifecyclePosture,
    ) -> Self {
        Self {
            portal_identity,
            owner,
            placement,
            lifecycle,
        }
    }

    pub(crate) const fn owner(self) -> worth_ui_host_contract::UiMountedInstanceIdentity {
        self.owner
    }

    pub(crate) fn same_mounted_projection_meaning(self, other: Self) -> bool {
        let left_presentation = self.placement.presentation();
        let right_presentation = other.placement.presentation();
        self.portal_identity == other.portal_identity
            && self.owner == other.owner
            && left_presentation.host_surface() == right_presentation.host_surface()
            && left_presentation.binding() == right_presentation.binding()
            && self.placement.anchor() == other.placement.anchor()
            && self.placement.clip_bounds() == other.placement.clip_bounds()
            && self.placement.bounds() == other.placement.bounds()
            && self.placement.layer() == other.placement.layer()
            && self.placement.shielding() == other.placement.shielding()
            && self.lifecycle == other.lifecycle
    }

    pub(crate) const fn lifecycle(self) -> crate::runtime::portal::UiPortalLifecyclePosture {
        self.lifecycle
    }

    pub(crate) fn mechanic_for(
        self,
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
        owner_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    ) -> Result<
        worth_ui_host_contract::UiMountedPortalOverlayMechanic,
        worth_ui_host_contract::UiMountedPortalOverlayCompletionDenial,
    > {
        let bounds = self.placement.bounds().mounted_box();
        let layer = self.placement.layer();
        worth_ui_host_contract::UiMountedPortalOverlayMechanic::complete_from_runtime_mounting(
            worth_ui_host_contract::UiMountedPortalOverlayCompletionInput {
                frame,
                surface,
                binding,
                owner: self.owner,
                owner_receipt,
                portal_identity: self.portal_identity,
                anchor_presentation: self.placement.presentation(),
                anchor_bounds: self.placement.anchor(),
                bounds,
                clip_bounds: self.placement.clip_bounds(),
                color: worth_ui_host_contract::UiMountedRgba8::new(0, 0, 0, 0),
                layer_semantic_order: u32::MAX - 4_096 + u32::from(layer.depth()),
                layer_depth: layer.depth(),
                lifecycle: match self.lifecycle {
                    crate::runtime::portal::UiPortalLifecyclePosture::Closing => {
                        worth_ui_host_contract::UiMountedPortalOverlayLifecyclePosture::Closing
                    }
                    crate::runtime::portal::UiPortalLifecyclePosture::Open
                    | crate::runtime::portal::UiPortalLifecyclePosture::Visible => {
                        worth_ui_host_contract::UiMountedPortalOverlayLifecyclePosture::Visible
                    }
                    crate::runtime::portal::UiPortalLifecyclePosture::Closed => {
                        unreachable!("closed portals do not enter mounted projection")
                    }
                },
                shielding: match self.placement.shielding() {
                    crate::runtime::portal::UiPortalInputShielding::ContentBounds => {
                        worth_ui_host_contract::UiMountedPortalInputShielding::ContentBounds
                    }
                    crate::runtime::portal::UiPortalInputShielding::ModalSurface => {
                        worth_ui_host_contract::UiMountedPortalInputShielding::ModalSurface
                    }
                },
            },
        )
    }
}

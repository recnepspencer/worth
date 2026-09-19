use worth_ui_host_contract::{
    UiHostObservationPresentationBasis, UiHostPointerIdentity, UiHostSurfacePosition,
    UiMountedInstanceIdentity, UiSemanticSurfaceIdentity,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiPointerPresenceTargetTransition {
    pub(super) generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    pub(super) pointer: UiHostPointerIdentity,
    pub(super) previous_surface: Option<UiSemanticSurfaceIdentity>,
    pub(super) current_surface: Option<UiSemanticSurfaceIdentity>,
    pub(super) previous: Option<UiMountedInstanceIdentity>,
    pub(super) current: Option<UiMountedInstanceIdentity>,
    pub(super) previous_node_receipt: Option<worth_ui_host_contract::UiMountedNodeReceiptIdentity>,
    pub(super) current_node_receipt: Option<worth_ui_host_contract::UiMountedNodeReceiptIdentity>,
    pub(super) owner_revision: u64,
    pub(super) position: UiHostSurfacePosition,
    pub(super) presentation: UiHostObservationPresentationBasis,
}

impl UiPointerPresenceTargetTransition {
    pub const fn generation(&self) -> &crate::runtime::WorthUiActiveApplicationGenerationIdentity {
        &self.generation
    }
    pub const fn pointer(&self) -> UiHostPointerIdentity {
        self.pointer
    }
    pub const fn previous_surface(&self) -> Option<UiSemanticSurfaceIdentity> {
        self.previous_surface
    }
    pub const fn current_surface(&self) -> Option<UiSemanticSurfaceIdentity> {
        self.current_surface
    }
    pub const fn previous(&self) -> Option<UiMountedInstanceIdentity> {
        self.previous
    }
    pub const fn current(&self) -> Option<UiMountedInstanceIdentity> {
        self.current
    }
    pub const fn previous_node_receipt(
        &self,
    ) -> Option<worth_ui_host_contract::UiMountedNodeReceiptIdentity> {
        self.previous_node_receipt
    }
    pub const fn current_node_receipt(
        &self,
    ) -> Option<worth_ui_host_contract::UiMountedNodeReceiptIdentity> {
        self.current_node_receipt
    }
    pub const fn owner_revision(&self) -> u64 {
        self.owner_revision
    }
    pub const fn position(&self) -> UiHostSurfacePosition {
        self.position
    }
    pub const fn presentation(&self) -> UiHostObservationPresentationBasis {
        self.presentation
    }
}

use worth_ui_host_contract::{
    UiHostObservationPresentationBasis, UiHostPointerIdentity, UiHostSurfacePosition,
    UiMountedInstanceIdentity, UiSemanticSurfaceIdentity,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiPointerPresenceTargetTransition {
    pub(super) generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    pub(super) pointer: UiHostPointerIdentity,
    /// The surface the pointer was on, or `None` for a pointer first seen now.
    pub(super) previous_surface: Option<UiSemanticSurfaceIdentity>,
    pub(super) current_surface: UiSemanticSurfaceIdentity,
    /// What the pointer was over; an instance and its node receipt come
    /// together from one presented target, never one without the other.
    pub(super) previous_target:
        Option<crate::runtime::interaction::UiPresentedInteractionTargetView>,
    pub(super) current_target:
        Option<crate::runtime::interaction::UiPresentedInteractionTargetView>,
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
    pub const fn current_surface(&self) -> UiSemanticSurfaceIdentity {
        self.current_surface
    }
    pub const fn previous_target(
        &self,
    ) -> Option<crate::runtime::interaction::UiPresentedInteractionTargetView> {
        self.previous_target
    }
    pub const fn current_target(
        &self,
    ) -> Option<crate::runtime::interaction::UiPresentedInteractionTargetView> {
        self.current_target
    }
    pub fn previous(&self) -> Option<UiMountedInstanceIdentity> {
        self.previous_target.map(|target| target.mounted_instance())
    }
    pub fn current(&self) -> Option<UiMountedInstanceIdentity> {
        self.current_target.map(|target| target.mounted_instance())
    }
    pub fn previous_node_receipt(
        &self,
    ) -> Option<worth_ui_host_contract::UiMountedNodeReceiptIdentity> {
        self.previous_target.map(|target| target.node_receipt())
    }
    pub fn current_node_receipt(
        &self,
    ) -> Option<worth_ui_host_contract::UiMountedNodeReceiptIdentity> {
        self.current_target.map(|target| target.node_receipt())
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

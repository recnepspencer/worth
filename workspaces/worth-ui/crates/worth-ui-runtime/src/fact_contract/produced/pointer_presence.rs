#[derive(Debug, Eq, PartialEq)]
pub struct UiPointerPresenceTargetChangedFact {
    generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    pointer: worth_ui_host_contract::UiHostPointerIdentity,
    previous_surface: Option<worth_ui_host_contract::UiSemanticSurfaceIdentity>,
    current_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    /// An instance and its node receipt always travel together.
    previous: Option<crate::runtime::interaction::UiPresentedInteractionTargetView>,
    current: Option<crate::runtime::interaction::UiPresentedInteractionTargetView>,
    owner_revision: u64,
    position: worth_ui_host_contract::UiHostSurfacePosition,
    presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
}

impl UiPointerPresenceTargetChangedFact {
    pub(crate) fn from_owner_transition(
        transition: crate::runtime::interaction::UiPointerPresenceTargetTransition,
    ) -> Self {
        Self {
            generation: transition.generation().clone(),
            pointer: transition.pointer(),
            previous_surface: transition.previous_surface(),
            current_surface: transition.current_surface(),
            previous: transition.previous_target(),
            current: transition.current_target(),
            owner_revision: transition.owner_revision(),
            position: transition.position(),
            presentation: transition.presentation(),
        }
    }

    pub const fn generation(&self) -> &crate::runtime::WorthUiActiveApplicationGenerationIdentity {
        &self.generation
    }

    pub const fn pointer(&self) -> worth_ui_host_contract::UiHostPointerIdentity {
        self.pointer
    }
    pub const fn previous_surface(
        &self,
    ) -> Option<worth_ui_host_contract::UiSemanticSurfaceIdentity> {
        self.previous_surface
    }
    pub const fn current_surface(&self) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.current_surface
    }
    pub fn previous(&self) -> Option<worth_ui_host_contract::UiMountedInstanceIdentity> {
        self.previous.map(|target| target.mounted_instance())
    }
    pub fn current(&self) -> Option<worth_ui_host_contract::UiMountedInstanceIdentity> {
        self.current.map(|target| target.mounted_instance())
    }
    pub fn previous_node_receipt(
        &self,
    ) -> Option<worth_ui_host_contract::UiMountedNodeReceiptIdentity> {
        self.previous.map(|target| target.node_receipt())
    }
    pub fn current_node_receipt(
        &self,
    ) -> Option<worth_ui_host_contract::UiMountedNodeReceiptIdentity> {
        self.current.map(|target| target.node_receipt())
    }
    pub const fn owner_revision(&self) -> u64 {
        self.owner_revision
    }
    pub const fn position(&self) -> worth_ui_host_contract::UiHostSurfacePosition {
        self.position
    }
    pub const fn presentation(&self) -> worth_ui_host_contract::UiHostObservationPresentationBasis {
        self.presentation
    }
}

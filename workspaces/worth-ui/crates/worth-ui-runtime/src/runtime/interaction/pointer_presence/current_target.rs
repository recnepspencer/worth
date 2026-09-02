use worth_ui_host_contract::{
    UiHostObservationPresentationBasis, UiHostObservationSequence, UiHostPointerIdentity,
    UiMountedInstanceIdentity, UiSemanticSurfaceIdentity,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiPointerPresenceClass {
    Outside,
    Hovered,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiPointerPresenceAppearancePosture {
    pub(super) pointer: UiHostPointerIdentity,
    pub(super) kind: super::UiPrimaryPointerKind,
    pub(super) presentation: UiHostObservationPresentationBasis,
    pub(super) target: Option<UiMountedInstanceIdentity>,
    pub(super) node_receipt: Option<worth_ui_host_contract::UiMountedNodeReceiptIdentity>,
    pub(super) class: UiPointerPresenceClass,
    pub(super) owner_revision: u64,
    pub(super) observation_sequence: UiHostObservationSequence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiPointerPresenceAppearanceOwnerSnapshot {
    pub(super) owner_revision: u64,
    pub(super) primary_by_surface: Box<[(UiSemanticSurfaceIdentity, UiHostPointerIdentity)]>,
    pub(super) postures: Box<[UiPointerPresenceAppearancePosture]>,
}

#[allow(
    dead_code,
    reason = "Gate 0 seals pointer-presence snapshots before role resolution"
)]
impl UiPointerPresenceAppearanceOwnerSnapshot {
    pub(crate) const fn owner_revision(&self) -> u64 {
        self.owner_revision
    }
    pub(crate) fn postures(&self) -> &[UiPointerPresenceAppearancePosture] {
        &self.postures
    }
    pub(crate) fn primary_pointer(
        &self,
        surface: UiSemanticSurfaceIdentity,
    ) -> Option<UiHostPointerIdentity> {
        self.primary_by_surface
            .iter()
            .find_map(|(candidate, pointer)| (*candidate == surface).then_some(*pointer))
    }
}

#[allow(
    dead_code,
    reason = "Gate 0 seals pointer posture before role resolution"
)]
impl UiPointerPresenceAppearancePosture {
    pub(crate) const fn pointer(self) -> UiHostPointerIdentity {
        self.pointer
    }
    pub(crate) const fn kind(self) -> super::UiPrimaryPointerKind {
        self.kind
    }
    pub(crate) const fn presentation(self) -> UiHostObservationPresentationBasis {
        self.presentation
    }
    pub(crate) const fn target(self) -> Option<UiMountedInstanceIdentity> {
        self.target
    }
    pub(crate) const fn node_receipt(
        self,
    ) -> Option<worth_ui_host_contract::UiMountedNodeReceiptIdentity> {
        self.node_receipt
    }
    pub(crate) const fn class(self) -> UiPointerPresenceClass {
        self.class
    }
    pub(crate) const fn owner_revision(self) -> u64 {
        self.owner_revision
    }
    pub(crate) const fn observation_sequence(self) -> UiHostObservationSequence {
        self.observation_sequence
    }
}

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
    pub(super) target: Option<crate::runtime::interaction::UiPresentedInteractionTargetView>,
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
    pub(crate) fn primary_postures(
        &self,
    ) -> impl Iterator<
        Item = (
            UiSemanticSurfaceIdentity,
            UiPointerPresenceAppearancePosture,
        ),
    > + '_ {
        self.primary_by_surface
            .iter()
            .filter_map(|(surface, pointer)| {
                self.postures
                    .binary_search_by_key(pointer, |posture| posture.pointer())
                    .ok()
                    .map(|index| (*surface, self.postures[index]))
            })
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
    pub(crate) fn appearance_dependency_eq(self, other: Self) -> bool {
        self.pointer == other.pointer
            && self.kind == other.kind
            && self.presentation.host_surface() == other.presentation.host_surface()
            && self.presentation.binding() == other.presentation.binding()
            && self.target() == other.target()
            && self.class == other.class
    }

    pub(crate) const fn pointer(self) -> UiHostPointerIdentity {
        self.pointer
    }
    pub(crate) const fn kind(self) -> super::UiPrimaryPointerKind {
        self.kind
    }
    pub(crate) const fn presentation(self) -> UiHostObservationPresentationBasis {
        self.presentation
    }
    pub(crate) fn target(self) -> Option<UiMountedInstanceIdentity> {
        self.target.map(|target| target.mounted_instance())
    }
    pub(crate) const fn presented_target(
        self,
    ) -> Option<crate::runtime::interaction::UiPresentedInteractionTargetView> {
        self.target
    }
    pub(crate) fn node_receipt(
        self,
    ) -> Option<worth_ui_host_contract::UiMountedNodeReceiptIdentity> {
        self.target.map(|target| target.node_receipt())
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

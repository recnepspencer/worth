use worth_ui_dsl::{UiAppearanceAxisClass, UiAppearanceStateAxis};
use worth_ui_host_contract::{
    UiHostObservationSequence, UiHostPointerIdentity, UiMountedInstanceIdentity,
    UiMountedNodeReceiptIdentity,
};

use super::{UiAppearanceCoherentBasis, UiAppearanceOwnerSnapshot, UiAppearanceStateAdapterDenial};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiHoverAppearanceState {
    class: UiAppearanceAxisClass,
    source_class: crate::runtime::interaction::UiPointerPresenceClass,
    owner_revision: u64,
    pointer: Option<UiHostPointerIdentity>,
    kind: Option<crate::runtime::interaction::UiPrimaryPointerKind>,
    target: Option<UiMountedInstanceIdentity>,
    node_receipt: Option<UiMountedNodeReceiptIdentity>,
    observation_sequence: Option<UiHostObservationSequence>,
}

pub(crate) fn adapt(
    snapshot: &UiAppearanceOwnerSnapshot,
    basis: &UiAppearanceCoherentBasis,
) -> Result<UiHoverAppearanceState, UiAppearanceStateAdapterDenial> {
    let owner = snapshot
        .pointer_presence()
        .ok_or(UiAppearanceStateAdapterDenial::MissingOwner(
            UiAppearanceStateAxis::Hover,
        ))?;
    let owner_revision = owner.owner_revision();
    let Some(pointer) = owner.primary_pointer(basis.surface()) else {
        return Ok(UiHoverAppearanceState {
            class: UiAppearanceAxisClass::HoverOutside,
            source_class: crate::runtime::interaction::UiPointerPresenceClass::Outside,
            owner_revision,
            pointer: None,
            kind: None,
            target: None,
            node_receipt: None,
            observation_sequence: None,
        });
    };
    let posture = owner
        .postures()
        .iter()
        .find(|posture| posture.pointer() == pointer)
        .copied()
        .ok_or(UiAppearanceStateAdapterDenial::MissingSource(
            UiAppearanceStateAxis::Hover,
        ))?;
    let on_target = basis
        .presentation()
        .is_some_and(|presentation| posture.presentation() == presentation)
        && posture.target() == Some(basis.mounted_instance())
        && posture.node_receipt() == Some(basis.owner_node_receipt());
    Ok(UiHoverAppearanceState {
        class: if on_target {
            UiAppearanceAxisClass::Hovered
        } else {
            UiAppearanceAxisClass::HoverOutside
        },
        source_class: posture.class(),
        owner_revision,
        pointer: Some(pointer),
        kind: Some(posture.kind()),
        target: posture.target(),
        node_receipt: posture.node_receipt(),
        observation_sequence: Some(posture.observation_sequence()),
    })
}

impl UiHoverAppearanceState {
    pub(crate) const fn class(&self) -> UiAppearanceAxisClass {
        self.class
    }

    pub(crate) const fn source_class(&self) -> crate::runtime::interaction::UiPointerPresenceClass {
        self.source_class
    }

    pub(crate) const fn owner_revision(&self) -> u64 {
        self.owner_revision
    }

    pub(crate) const fn pointer(&self) -> Option<UiHostPointerIdentity> {
        self.pointer
    }

    pub(crate) const fn kind(&self) -> Option<crate::runtime::interaction::UiPrimaryPointerKind> {
        self.kind
    }

    pub(crate) const fn target(&self) -> Option<UiMountedInstanceIdentity> {
        self.target
    }

    pub(crate) const fn node_receipt(&self) -> Option<UiMountedNodeReceiptIdentity> {
        self.node_receipt
    }

    pub(crate) const fn observation_sequence(&self) -> Option<UiHostObservationSequence> {
        self.observation_sequence
    }
}

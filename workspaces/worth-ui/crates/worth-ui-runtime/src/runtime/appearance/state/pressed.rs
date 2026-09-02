use worth_ui_dsl::{UiAppearanceAxisClass, UiAppearanceStateAxis};
use worth_ui_host_contract::{
    UiHostObservationPresentationBasis, UiHostObservationSequence, UiHostPointerIdentity,
    UiMountedInstanceIdentity, UiMountedNodeReceiptIdentity,
};

use super::{UiAppearanceCoherentBasis, UiAppearanceOwnerSnapshot, UiAppearanceStateAdapterDenial};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiPressedAppearanceState {
    class: UiAppearanceAxisClass,
    source_class: Option<crate::runtime::interaction::gesture::UiPressedAppearanceClass>,
    owner_revision: u64,
    pointer: Option<UiHostPointerIdentity>,
    presentation: Option<UiHostObservationPresentationBasis>,
    target: UiMountedInstanceIdentity,
    node_receipt: UiMountedNodeReceiptIdentity,
    press_sequence: Option<UiHostObservationSequence>,
}

pub(crate) fn adapt(
    snapshot: &UiAppearanceOwnerSnapshot,
    basis: &UiAppearanceCoherentBasis,
) -> Result<UiPressedAppearanceState, UiAppearanceStateAdapterDenial> {
    let owner = snapshot
        .pressed()
        .ok_or(UiAppearanceStateAdapterDenial::MissingOwner(
            UiAppearanceStateAxis::Pressed,
        ))?;
    let postures = owner
        .postures()
        .iter()
        .filter(|posture| {
            posture.target() == basis.mounted_instance()
                && posture.node_receipt() == basis.node_receipt()
        })
        .collect::<Vec<_>>();
    if postures.len() > 1 {
        return Err(UiAppearanceStateAdapterDenial::AmbiguousSource(
            UiAppearanceStateAxis::Pressed,
        ));
    }
    Ok(match postures.first().copied() {
        Some(posture) => UiPressedAppearanceState {
            class: map_class(posture.class()),
            source_class: Some(posture.class()),
            owner_revision: posture.owner_revision(),
            pointer: Some(posture.pointer()),
            presentation: Some(posture.presentation()),
            target: basis.mounted_instance(),
            node_receipt: basis.node_receipt(),
            press_sequence: Some(posture.press_sequence()),
        },
        None => UiPressedAppearanceState {
            class: UiAppearanceAxisClass::PressedIdle,
            source_class: None,
            owner_revision: owner.owner_revision(),
            pointer: None,
            presentation: None,
            target: basis.mounted_instance(),
            node_receipt: basis.node_receipt(),
            press_sequence: None,
        },
    })
}

fn map_class(
    class: crate::runtime::interaction::gesture::UiPressedAppearanceClass,
) -> UiAppearanceAxisClass {
    match class {
        crate::runtime::interaction::gesture::UiPressedAppearanceClass::ArmedInside => {
            UiAppearanceAxisClass::PressedArmedInside
        }
        crate::runtime::interaction::gesture::UiPressedAppearanceClass::CapturedOutside => {
            UiAppearanceAxisClass::PressedCapturedOutside
        }
    }
}

impl UiPressedAppearanceState {
    pub(crate) const fn class(&self) -> UiAppearanceAxisClass {
        self.class
    }

    pub(crate) const fn source_class(
        &self,
    ) -> Option<crate::runtime::interaction::gesture::UiPressedAppearanceClass> {
        self.source_class
    }

    pub(crate) const fn owner_revision(&self) -> u64 {
        self.owner_revision
    }

    pub(crate) const fn pointer(&self) -> Option<UiHostPointerIdentity> {
        self.pointer
    }

    pub(crate) const fn presentation(&self) -> Option<UiHostObservationPresentationBasis> {
        self.presentation
    }

    pub(crate) const fn target(&self) -> UiMountedInstanceIdentity {
        self.target
    }

    pub(crate) const fn node_receipt(&self) -> UiMountedNodeReceiptIdentity {
        self.node_receipt
    }

    pub(crate) const fn press_sequence(&self) -> Option<UiHostObservationSequence> {
        self.press_sequence
    }
}

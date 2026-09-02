use worth_ui_dsl::{UiAppearanceAxisClass, UiAppearanceStateAxis};
use worth_ui_host_contract::UiMountedInstanceIdentity;

use super::{UiAppearanceCoherentBasis, UiAppearanceOwnerSnapshot, UiAppearanceStateAdapterDenial};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiFocusAppearanceState {
    class: UiAppearanceAxisClass,
    source_class: crate::runtime::focus::UiFocusAppearanceClass,
    owner_revision: u64,
    target: Option<UiMountedInstanceIdentity>,
}

pub(crate) fn adapt(
    snapshot: &UiAppearanceOwnerSnapshot,
    basis: &UiAppearanceCoherentBasis,
) -> Result<UiFocusAppearanceState, UiAppearanceStateAdapterDenial> {
    let posture = snapshot
        .focus()
        .copied()
        .ok_or(UiAppearanceStateAdapterDenial::MissingOwner(
            UiAppearanceStateAxis::Focus,
        ))?;
    let target = posture.target();
    let current = target.is_some_and(|target| {
        target.graph_node() == basis.graph_node()
            && target.mounted_instance() == basis.mounted_instance()
            && target.incarnation() == basis.incarnation()
    });
    Ok(UiFocusAppearanceState {
        class: if current {
            map_class(posture.class())
        } else {
            UiAppearanceAxisClass::FocusUnfocused
        },
        source_class: posture.class(),
        owner_revision: posture.owner_revision(),
        target: current.then(|| basis.mounted_instance()),
    })
}

fn map_class(class: crate::runtime::focus::UiFocusAppearanceClass) -> UiAppearanceAxisClass {
    match class {
        crate::runtime::focus::UiFocusAppearanceClass::Unfocused => {
            UiAppearanceAxisClass::FocusUnfocused
        }
        crate::runtime::focus::UiFocusAppearanceClass::Focused => {
            UiAppearanceAxisClass::FocusFocused
        }
        crate::runtime::focus::UiFocusAppearanceClass::FocusVisible => {
            UiAppearanceAxisClass::FocusVisible
        }
        crate::runtime::focus::UiFocusAppearanceClass::FocusedWindowInactive => {
            UiAppearanceAxisClass::FocusedWindowInactive
        }
    }
}

impl UiFocusAppearanceState {
    pub(crate) const fn class(&self) -> UiAppearanceAxisClass {
        self.class
    }

    pub(crate) const fn source_class(&self) -> crate::runtime::focus::UiFocusAppearanceClass {
        self.source_class
    }

    pub(crate) const fn owner_revision(&self) -> u64 {
        self.owner_revision
    }

    pub(crate) const fn target(&self) -> Option<UiMountedInstanceIdentity> {
        self.target
    }
}

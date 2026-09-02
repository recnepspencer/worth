use worth_ui_dsl::{UiAppearanceAxisClass, UiAppearanceStateAxis};

use super::{UiAppearanceCoherentBasis, UiAppearanceOwnerSnapshot, UiAppearanceStateAdapterDenial};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiSelectionAppearanceState {
    class: UiAppearanceAxisClass,
    source_class: crate::runtime::selection::UiSelectionAppearanceClass,
    source_bits: (bool, bool, bool),
    owner_revision: u64,
    owner: crate::runtime::selection::UiSelectionOwnerIdentity,
    incarnation: crate::runtime::selection::UiSelectionOwnerIncarnation,
    key: crate::runtime::selection::UiSelectionStableKey,
}

pub(crate) fn adapt(
    snapshot: &UiAppearanceOwnerSnapshot,
    basis: &UiAppearanceCoherentBasis,
) -> Result<UiSelectionAppearanceState, UiAppearanceStateAdapterDenial> {
    let selector = basis
        .selection()
        .ok_or(UiAppearanceStateAdapterDenial::MissingSource(
            UiAppearanceStateAxis::Selection,
        ))?;
    let owner = selector.owner();
    if owner.semantic_surface() != basis.surface() || owner.graph_node() != basis.graph_node() {
        return Err(UiAppearanceStateAdapterDenial::ForeignSource(
            UiAppearanceStateAxis::Selection,
        ));
    }
    let owner_snapshot =
        snapshot
            .selection()
            .ok_or(UiAppearanceStateAdapterDenial::MissingOwner(
                UiAppearanceStateAxis::Selection,
            ))?;
    let posture = owner_snapshot
        .posture_for(selector.owner(), selector.key(), selector.incarnation())
        .map_err(|denial| match denial {
            crate::runtime::selection::UiSelectionAppearancePostureDenial::UnknownOwner => {
                UiAppearanceStateAdapterDenial::MissingSource(UiAppearanceStateAxis::Selection)
            }
            crate::runtime::selection::UiSelectionAppearancePostureDenial::StaleOwnerIncarnation => {
                UiAppearanceStateAdapterDenial::StaleSource(UiAppearanceStateAxis::Selection)
            }
            crate::runtime::selection::UiSelectionAppearancePostureDenial::ForeignItemKeyFamily => {
                UiAppearanceStateAdapterDenial::ForeignSource(UiAppearanceStateAxis::Selection)
            }
            crate::runtime::selection::UiSelectionAppearancePostureDenial::AmbiguousMountedOwner => {
                UiAppearanceStateAdapterDenial::AmbiguousSource(UiAppearanceStateAxis::Selection)
            }
        })?;
    Ok(UiSelectionAppearanceState {
        class: map_class(posture.class()),
        source_class: posture.class(),
        source_bits: posture.source_bits(),
        owner_revision: posture.owner_revision(),
        owner: posture.owner(),
        incarnation: posture.incarnation(),
        key: posture.key(),
    })
}

fn map_class(
    class: crate::runtime::selection::UiSelectionAppearanceClass,
) -> UiAppearanceAxisClass {
    match class {
        crate::runtime::selection::UiSelectionAppearanceClass::Unselected => {
            UiAppearanceAxisClass::SelectionUnselected
        }
        crate::runtime::selection::UiSelectionAppearanceClass::Selected => {
            UiAppearanceAxisClass::SelectionSelected
        }
        crate::runtime::selection::UiSelectionAppearanceClass::Anchor => {
            UiAppearanceAxisClass::SelectionAnchor
        }
        crate::runtime::selection::UiSelectionAppearanceClass::Cursor => {
            UiAppearanceAxisClass::SelectionCursor
        }
        crate::runtime::selection::UiSelectionAppearanceClass::SelectedAnchorCursor => {
            UiAppearanceAxisClass::SelectedAnchorCursor
        }
    }
}

impl UiSelectionAppearanceState {
    pub(crate) const fn class(&self) -> UiAppearanceAxisClass {
        self.class
    }

    pub(crate) const fn source_class(
        &self,
    ) -> crate::runtime::selection::UiSelectionAppearanceClass {
        self.source_class
    }

    pub(crate) const fn source_bits(&self) -> (bool, bool, bool) {
        self.source_bits
    }

    pub(crate) const fn owner_revision(&self) -> u64 {
        self.owner_revision
    }

    pub(crate) const fn owner(&self) -> crate::runtime::selection::UiSelectionOwnerIdentity {
        self.owner
    }

    pub(crate) const fn incarnation(
        &self,
    ) -> crate::runtime::selection::UiSelectionOwnerIncarnation {
        self.incarnation
    }

    pub(crate) const fn key(&self) -> crate::runtime::selection::UiSelectionStableKey {
        self.key
    }
}

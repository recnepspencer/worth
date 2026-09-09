#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiSelectionAppearanceClass {
    Unselected,
    Selected,
    Anchor,
    Cursor,
    SelectedAnchorCursor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiSelectionAppearancePosture {
    class: UiSelectionAppearanceClass,
    selected: bool,
    anchor: bool,
    cursor: bool,
    owner_revision: u64,
    owner: super::UiSelectionOwnerIdentity,
    incarnation: super::UiSelectionOwnerIncarnation,
    key: super::UiSelectionStableKey,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiSelectionAppearancePostureDenial {
    UnknownOwner,
    StaleOwnerIncarnation,
    ForeignItemKeyFamily,
    AmbiguousMountedOwner,
}

#[cfg(test)]
mod change_tests;
mod changes;
#[cfg(test)]
mod scale_tests;
mod snapshot;
#[cfg(test)]
mod tests;
pub(crate) use changes::UiSelectionAppearanceChange;
pub(crate) use snapshot::UiSelectionAppearanceOwnerSnapshot;

impl super::UiSelectionRuntimeState {
    pub(crate) fn appearance_posture(
        &self,
        owner: super::UiSelectionOwnerIdentity,
        key: super::UiSelectionStableKey,
        incarnation: super::UiSelectionOwnerIncarnation,
    ) -> Result<UiSelectionAppearancePosture, UiSelectionAppearancePostureDenial> {
        let record = self
            .owners
            .get(&owner)
            .ok_or(UiSelectionAppearancePostureDenial::UnknownOwner)?;
        if record.incarnation != incarnation {
            return Err(UiSelectionAppearancePostureDenial::StaleOwnerIncarnation);
        }
        if key.family() != owner.key_family() {
            return Err(UiSelectionAppearancePostureDenial::ForeignItemKeyFamily);
        }
        Ok(posture(owner, key, record, record.revision))
    }

    pub(crate) fn appearance_posture_for_mounted(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        graph_node: crate::graph::UiGraphNodeIdentity,
        key: super::UiSelectionStableKey,
        incarnation: super::UiSelectionOwnerIncarnation,
    ) -> Result<UiSelectionAppearancePosture, UiSelectionAppearancePostureDenial> {
        let owners = self
            .mounted_owners
            .get(&(surface, graph_node, incarnation))
            .ok_or(UiSelectionAppearancePostureDenial::UnknownOwner)?;
        if owners.len() != 1 {
            return Err(UiSelectionAppearancePostureDenial::AmbiguousMountedOwner);
        }
        self.appearance_posture(
            *owners.iter().next().expect("one exact owner"),
            key,
            incarnation,
        )
    }

    pub(crate) fn appearance_owner_snapshot(&self) -> UiSelectionAppearanceOwnerSnapshot {
        UiSelectionAppearanceOwnerSnapshot {
            owner_revision: self.revision,
            owners: self.owners.clone(),
            mounted_owners: self.mounted_owners.clone(),
        }
    }
}

fn posture(
    owner: super::UiSelectionOwnerIdentity,
    key: super::UiSelectionStableKey,
    record: &super::state::UiSelectionOwnerRecord,
    owner_revision: u64,
) -> UiSelectionAppearancePosture {
    let selected = record.selected.contains(&key);
    let anchor = record.anchor == Some(key);
    let cursor = record.cursor == Some(key);
    let class = match (selected, anchor, cursor) {
        (_, true, true) => UiSelectionAppearanceClass::SelectedAnchorCursor,
        (_, true, false) => UiSelectionAppearanceClass::Anchor,
        (_, false, true) => UiSelectionAppearanceClass::Cursor,
        (true, false, false) => UiSelectionAppearanceClass::Selected,
        (false, false, false) => UiSelectionAppearanceClass::Unselected,
    };
    UiSelectionAppearancePosture {
        class,
        selected,
        anchor,
        cursor,
        owner_revision,
        owner,
        incarnation: record.incarnation,
        key,
    }
}

impl UiSelectionAppearancePosture {
    pub(crate) const fn class(self) -> UiSelectionAppearanceClass {
        self.class
    }
    pub(crate) const fn source_bits(self) -> (bool, bool, bool) {
        (self.selected, self.anchor, self.cursor)
    }
    pub(crate) const fn owner_revision(self) -> u64 {
        self.owner_revision
    }
    pub(crate) const fn owner(self) -> super::UiSelectionOwnerIdentity {
        self.owner
    }
    pub(crate) const fn incarnation(self) -> super::UiSelectionOwnerIncarnation {
        self.incarnation
    }
    pub(crate) const fn key(self) -> super::UiSelectionStableKey {
        self.key
    }
}

use std::collections::BTreeSet;

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiSelectionAppearanceOwnerSnapshot {
    owner_revision: u64,
    owners: Box<
        [(
            super::UiSelectionOwnerIdentity,
            super::UiSelectionOwnerIncarnation,
        )],
    >,
    postures: Box<[UiSelectionAppearancePosture]>,
}

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
        Ok(posture(owner, key, record, self.revision))
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
        self.appearance_posture(*owners.first().expect("one exact owner"), key, incarnation)
    }

    pub(crate) fn appearance_owner_snapshot(&self) -> UiSelectionAppearanceOwnerSnapshot {
        let postures = self
            .owners
            .iter()
            .flat_map(|(owner, record)| {
                appearance_keys(record)
                    .into_iter()
                    .map(|key| posture(*owner, key, record, self.revision))
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let owners = self
            .owners
            .iter()
            .map(|(owner, record)| (*owner, record.incarnation))
            .collect();
        UiSelectionAppearanceOwnerSnapshot {
            owner_revision: self.revision,
            owners,
            postures,
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

impl UiSelectionAppearanceOwnerSnapshot {
    pub(crate) const fn owner_revision(&self) -> u64 {
        self.owner_revision
    }
    pub(crate) fn postures(&self) -> &[UiSelectionAppearancePosture] {
        &self.postures
    }

    pub(crate) fn posture_for(
        &self,
        owner: super::UiSelectionOwnerIdentity,
        key: super::UiSelectionStableKey,
        incarnation: super::UiSelectionOwnerIncarnation,
    ) -> Result<UiSelectionAppearancePosture, UiSelectionAppearancePostureDenial> {
        let owner_incarnation = self
            .owners
            .iter()
            .find_map(|(candidate, owner_incarnation)| {
                (*candidate == owner).then_some(*owner_incarnation)
            })
            .ok_or(UiSelectionAppearancePostureDenial::UnknownOwner)?;
        if owner_incarnation != incarnation {
            return Err(UiSelectionAppearancePostureDenial::StaleOwnerIncarnation);
        }
        if key.family() != owner.key_family() {
            return Err(UiSelectionAppearancePostureDenial::ForeignItemKeyFamily);
        }
        Ok(self
            .postures
            .iter()
            .find(|posture| {
                posture.owner() == owner
                    && posture.key() == key
                    && posture.incarnation() == incarnation
            })
            .copied()
            .unwrap_or(UiSelectionAppearancePosture {
                class: UiSelectionAppearanceClass::Unselected,
                selected: false,
                anchor: false,
                cursor: false,
                owner_revision: self.owner_revision,
                owner,
                incarnation,
                key,
            }))
    }
}

fn appearance_keys(
    record: &super::state::UiSelectionOwnerRecord,
) -> Vec<super::UiSelectionStableKey> {
    let mut keys = record.catalog.to_vec();
    let mut retained = BTreeSet::new();
    retained.extend(record.selected.iter().copied());
    if let Some(key) = record.anchor {
        retained.insert(key);
    }
    if let Some(key) = record.cursor {
        retained.insert(key);
    }
    keys.extend(
        retained
            .into_iter()
            .filter(|key| !record.catalog_positions.contains_key(key)),
    );
    keys
}

#[cfg(test)]
mod tests {
    use super::super::state_test_fixture::{
        incarnation, item_key_family, key, owner, registration,
    };
    use super::super::{
        UiSelectionAppearanceClass, UiSelectionAppearancePostureDenial, UiSelectionPolicy,
        UiSelectionRuntimeState,
    };

    #[test]
    fn appearance_selection_export_is_keyed_and_rejects_reincarnation() {
        let owner = owner();
        let selected = key(1);
        let absent = key(2);
        let mut state = UiSelectionRuntimeState::new_session_restore_candidate();
        state
            .synchronize(registration(
                owner,
                UiSelectionPolicy::Single,
                vec![selected, absent],
                super::super::UiSelectionCatalogPosture::Complete,
            ))
            .unwrap();
        state
            .apply(
                owner,
                incarnation(),
                super::super::UiSelectionRequest::SelectSingle(selected),
            )
            .unwrap();
        let snapshot = state.appearance_owner_snapshot();

        let selected_posture = snapshot
            .posture_for(owner, selected, incarnation())
            .unwrap();
        assert_eq!(
            selected_posture.class(),
            UiSelectionAppearanceClass::SelectedAnchorCursor
        );
        assert_eq!(selected_posture.source_bits(), (true, true, true));
        let absent_posture = snapshot.posture_for(owner, absent, incarnation()).unwrap();
        assert_eq!(
            absent_posture.class(),
            UiSelectionAppearanceClass::Unselected
        );
        assert_eq!(absent_posture.source_bits(), (false, false, false));
        assert_eq!(
            snapshot.posture_for(
                owner,
                selected,
                super::super::UiSelectionOwnerIncarnation::new(8).unwrap(),
            ),
            Err(UiSelectionAppearancePostureDenial::StaleOwnerIncarnation)
        );
        let foreign = super::super::UiSelectionOwnerIdentity::new(
            worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
            crate::graph::UiGraphNodeIdentity::new(72),
            item_key_family(),
        );
        assert_eq!(
            snapshot.posture_for(foreign, selected, incarnation()),
            Err(UiSelectionAppearancePostureDenial::UnknownOwner)
        );
        let foreign_key =
            super::super::UiSelectionStableKey::new(crate::runtime::UiApplicationItemKey::new(
                crate::runtime::UiApplicationItemKeyFamily::new(
                    core::num::NonZeroU64::new(99).unwrap(),
                ),
                core::num::NonZeroU64::new(1).unwrap(),
            ));
        assert_eq!(
            snapshot.posture_for(owner, foreign_key, incarnation()),
            Err(UiSelectionAppearancePostureDenial::ForeignItemKeyFamily)
        );
    }

    #[test]
    fn appearance_selection_export_retains_partial_catalog_posture_keys() {
        let owner = owner();
        let retained = key(1);
        let replacement = key(2);
        let mut state = UiSelectionRuntimeState::new_session_restore_candidate();
        state
            .synchronize(registration(
                owner,
                UiSelectionPolicy::Single,
                vec![retained],
                super::super::UiSelectionCatalogPosture::Complete,
            ))
            .unwrap();
        state
            .apply(
                owner,
                incarnation(),
                super::super::UiSelectionRequest::SelectSingle(retained),
            )
            .unwrap();
        state
            .synchronize(registration(
                owner,
                UiSelectionPolicy::Single,
                vec![replacement],
                super::super::UiSelectionCatalogPosture::Partial,
            ))
            .unwrap();

        let posture = state
            .appearance_owner_snapshot()
            .posture_for(owner, retained, incarnation())
            .unwrap();
        assert_eq!(
            posture.class(),
            UiSelectionAppearanceClass::SelectedAnchorCursor
        );
        assert_eq!(posture.source_bits(), (true, true, true));
    }
}

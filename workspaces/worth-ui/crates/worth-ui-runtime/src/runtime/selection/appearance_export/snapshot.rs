use super::super::{UiSelectionOwnerIdentity, UiSelectionOwnerIncarnation, UiSelectionStableKey};
use super::{UiSelectionAppearancePosture, UiSelectionAppearancePostureDenial};
use crate::runtime::persistent_index::{UiPersistentOrdMap, UiPersistentOrdSet};

#[derive(Clone, Debug)]
pub(crate) struct UiSelectionAppearanceOwnerSnapshot {
    pub(super) owner_revision: u64,
    pub(super) owners:
        UiPersistentOrdMap<UiSelectionOwnerIdentity, super::super::state::UiSelectionOwnerRecord>,
    pub(super) mounted_owners: UiPersistentOrdMap<
        (
            worth_ui_host_contract::UiSemanticSurfaceIdentity,
            crate::graph::UiGraphNodeIdentity,
            UiSelectionOwnerIncarnation,
        ),
        UiPersistentOrdSet<UiSelectionOwnerIdentity>,
    >,
}

impl PartialEq for UiSelectionAppearanceOwnerSnapshot {
    fn eq(&self, other: &Self) -> bool {
        self.owner_revision == other.owner_revision
            && self.owners.root_is_shared_with(&other.owners)
            && self
                .mounted_owners
                .root_is_shared_with(&other.mounted_owners)
    }
}
impl Eq for UiSelectionAppearanceOwnerSnapshot {}

impl UiSelectionAppearanceOwnerSnapshot {
    pub(crate) const fn owner_revision(&self) -> u64 {
        self.owner_revision
    }

    pub(crate) fn posture_for(
        &self,
        owner: UiSelectionOwnerIdentity,
        key: UiSelectionStableKey,
        incarnation: UiSelectionOwnerIncarnation,
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
        if self
            .mounted_owners
            .get(&(owner.semantic_surface(), owner.graph_node(), incarnation))
            .is_some_and(|owners| owners.len() != 1)
        {
            return Err(UiSelectionAppearancePostureDenial::AmbiguousMountedOwner);
        }
        Ok(super::posture(owner, key, record, record.revision))
    }

    // Explicit test-only enumeration for legacy direct-vector fixtures. Ordinary
    // lookup and snapshot construction never materialize the collection catalog.
    #[cfg(test)]
    pub(crate) fn postures(&self) -> Vec<UiSelectionAppearancePosture> {
        self.owners
            .iter()
            .flat_map(|(owner, record)| {
                let mut keys = record
                    .catalog
                    .iter()
                    .copied()
                    .collect::<std::collections::BTreeSet<_>>();
                keys.extend(record.selected.iter().copied());
                keys.extend(record.anchor);
                keys.extend(record.cursor);
                keys.into_iter()
                    .map(|key| super::posture(*owner, key, record, record.revision))
            })
            .collect()
    }
}

use super::super::{UiSelectionOwnerIdentity, UiSelectionOwnerIncarnation, UiSelectionStableKey};
use super::UiSelectionAppearanceOwnerSnapshot;
use std::collections::{BTreeMap, BTreeSet};

/// Exact key changes are distinct from owner membership/incarnation changes,
/// which invalidate all mounted bindings of that owner, not its whole catalog.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum UiSelectionAppearanceChange {
    Keys {
        owner: UiSelectionOwnerIdentity,
        incarnation: UiSelectionOwnerIncarnation,
        keys: Box<[UiSelectionStableKey]>,
    },
    Owner {
        owner: UiSelectionOwnerIdentity,
        previous: Option<UiSelectionOwnerIncarnation>,
        current: Option<UiSelectionOwnerIncarnation>,
    },
}

impl UiSelectionAppearanceOwnerSnapshot {
    pub(crate) fn changes_since(&self, previous: &Self) -> Vec<UiSelectionAppearanceChange> {
        let mut changes = BTreeMap::new();
        for owner in self.owners.changed_keys_with_work(&previous.owners).0 {
            match (previous.owners.get(&owner), self.owners.get(&owner)) {
                (Some(old), Some(new)) if old.incarnation == new.incarnation => {
                    let mut keys = new
                        .selected
                        .changed_keys_with_work(&old.selected)
                        .0
                        .into_iter()
                        .collect::<BTreeSet<_>>();
                    if old.anchor != new.anchor {
                        keys.extend(old.anchor);
                        keys.extend(new.anchor);
                    }
                    if old.cursor != new.cursor {
                        keys.extend(old.cursor);
                        keys.extend(new.cursor);
                    }
                    if !keys.is_empty() {
                        changes.insert(
                            owner,
                            UiSelectionAppearanceChange::Keys {
                                owner,
                                incarnation: new.incarnation,
                                keys: keys.into_iter().collect(),
                            },
                        );
                    }
                }
                _ => {
                    changes.insert(owner, self.owner_change(previous, owner));
                }
            }
        }
        // A mounted owner gaining/losing an eligible family changes ambiguity
        // for the surviving family too. Only that indexed bucket is traversed.
        for mounted in self
            .mounted_owners
            .changed_keys_with_work(&previous.mounted_owners)
            .0
        {
            for snapshot in [previous, self] {
                if let Some(owners) = snapshot.mounted_owners.get(&mounted) {
                    for owner in owners.iter().copied() {
                        changes.insert(owner, self.owner_change(previous, owner));
                    }
                }
            }
        }
        changes.into_values().collect()
    }

    fn owner_change(
        &self,
        previous: &Self,
        owner: UiSelectionOwnerIdentity,
    ) -> UiSelectionAppearanceChange {
        UiSelectionAppearanceChange::Owner {
            owner,
            previous: previous.owners.get(&owner).map(|row| row.incarnation),
            current: self.owners.get(&owner).map(|row| row.incarnation),
        }
    }
}

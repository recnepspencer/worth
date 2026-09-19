use std::collections::{BTreeMap, BTreeSet};
use worth_ui_host_contract::UiMountedInstanceIdentity;

impl super::UiAppearanceOwnerSnapshot {
    pub(crate) fn hover_changed_instances(
        &self,
        predecessor: &Self,
    ) -> Box<[UiMountedInstanceIdentity]> {
        let primary = |snapshot: &Self| {
            snapshot
                .pointer_presence()
                .into_iter()
                .flat_map(|owner| owner.primary_postures())
                .collect::<BTreeMap<_, _>>()
        };
        let previous = primary(predecessor);
        let current = primary(self);
        let surfaces = previous
            .keys()
            .chain(current.keys())
            .copied()
            .collect::<BTreeSet<_>>();
        let mut instances = BTreeSet::new();
        for surface in surfaces {
            let old = previous.get(&surface);
            let new = current.get(&surface);
            let unchanged = old
                .zip(new)
                .is_some_and(|(old, new)| old.appearance_dependency_eq(*new));
            if !unchanged {
                instances.extend(old.and_then(|posture| posture.target()));
                instances.extend(new.and_then(|posture| posture.target()));
            }
        }
        instances.into_iter().collect()
    }

    pub(crate) fn pressed_changed_instances(
        &self,
        predecessor: &Self,
    ) -> Box<[UiMountedInstanceIdentity]> {
        let pressed = |snapshot: &Self| {
            snapshot
                .pressed()
                .into_iter()
                .flat_map(|owner| owner.postures())
                .map(|posture| (posture.pointer(), *posture))
                .collect::<BTreeMap<_, _>>()
        };
        let previous = pressed(predecessor);
        let current = pressed(self);
        let pointers = previous
            .keys()
            .chain(current.keys())
            .copied()
            .collect::<BTreeSet<_>>();
        let mut instances = BTreeSet::new();
        for pointer in pointers {
            let old = previous.get(&pointer);
            let new = current.get(&pointer);
            let unchanged = old
                .zip(new)
                .is_some_and(|(old, new)| old.appearance_dependency_eq(*new));
            if !unchanged {
                instances.extend(old.map(|posture| posture.target()));
                instances.extend(new.map(|posture| posture.target()));
            }
        }
        instances.into_iter().collect()
    }
}

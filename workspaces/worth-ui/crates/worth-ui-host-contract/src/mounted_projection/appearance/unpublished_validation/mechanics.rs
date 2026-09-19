use std::collections::HashSet;

use super::super::UiUnpublishedAppearanceFragment;
use crate::{
    UiMountedAppearanceMechanic, UiMountedAppearanceMechanicChange,
    UiMountedAppearanceMechanicIdentity,
};

pub(super) fn identities(
    fragment: &UiUnpublishedAppearanceFragment,
) -> Vec<UiMountedAppearanceMechanicIdentity> {
    let mut seen = HashSet::new();
    let mut identities: Vec<UiMountedAppearanceMechanicIdentity> = Vec::new();
    for identity in fragment
        .work
        .successor()
        .mechanics()
        .iter()
        .map(UiMountedAppearanceMechanic::identity)
        .chain(
            fragment
                .work
                .changes()
                .iter()
                .flat_map(|change| match change {
                    UiMountedAppearanceMechanicChange::Insert(mechanic) => {
                        vec![mechanic.identity()]
                    }
                    UiMountedAppearanceMechanicChange::Replace {
                        predecessor,
                        successor,
                    } => vec![predecessor.clone(), successor.identity()],
                    UiMountedAppearanceMechanicChange::Remove(identity) => vec![identity.clone()],
                }),
        )
    {
        if seen.insert(identity.clone()) {
            identities.push(identity);
        }
    }
    identities
}

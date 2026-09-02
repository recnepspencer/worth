use std::collections::BTreeMap;

use super::snapshot::{
    UiBackdropInstanceIdentity, UiOverlayBackdropRow, UiOverlayStackParticipant,
};

pub(super) fn changed_backdrop_count(
    previous: &[UiOverlayStackParticipant],
    current: &[UiOverlayStackParticipant],
) -> usize {
    let previous = backdrop_rows(previous);
    let current = backdrop_rows(current);
    let mut identities = BTreeMap::new();
    identities.extend(previous.keys().map(|identity| (*identity, ())));
    identities.extend(current.keys().map(|identity| (*identity, ())));
    identities
        .keys()
        .filter(|identity| previous.get(identity) != current.get(identity))
        .count()
}

fn backdrop_rows(
    participants: &[UiOverlayStackParticipant],
) -> BTreeMap<UiBackdropInstanceIdentity, &UiOverlayBackdropRow> {
    participants
        .iter()
        .filter_map(|participant| match participant {
            UiOverlayStackParticipant::Portal(_) => None,
            UiOverlayStackParticipant::Backdrop(row) => Some((row.identity(), row)),
        })
        .collect()
}

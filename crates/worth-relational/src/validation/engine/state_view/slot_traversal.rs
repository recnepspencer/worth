//! Selected-state scans visit logical slots, including sparse applied identities.
use super::InvariantStateView;
use crate::identity::data::PartitionId;
use crate::storage::residency::RelationalRecordHeadArenaView;
use crate::storage::substrate::RecordKind;

impl<'state> InvariantStateView<'state> {
    pub(crate) fn entity_slots(&self, id: PartitionId) -> impl Iterator<Item = usize> + 'state {
        occupied_slots(
            self.state
                .get_current_partition(id)
                .map(|view| view.entity_arena()),
            self.state
                .base_current_partition(id)
                .map(|view| view.entity_arena()),
        )
    }

    pub(crate) fn relation_slots(&self, id: PartitionId) -> impl Iterator<Item = usize> + 'state {
        occupied_slots(
            self.state
                .get_current_partition(id)
                .map(|view| view.relation_arena()),
            self.state
                .base_current_partition(id)
                .map(|view| view.relation_arena()),
        )
    }
}

fn occupied_slots<'state, K: RecordKind + 'state>(
    visible: Option<RelationalRecordHeadArenaView<'state, K>>,
    base: Option<RelationalRecordHeadArenaView<'state, K>>,
) -> impl Iterator<Item = usize> + 'state {
    let mut visible = visible
        .into_iter()
        .flat_map(|arena| arena.occupied_slots_iter())
        .peekable();
    let mut base = base
        .into_iter()
        .flat_map(|arena| arena.occupied_slots_iter())
        .peekable();
    std::iter::from_fn(
        move || match (visible.peek().copied(), base.peek().copied()) {
            (Some(left), Some(right)) => match left.cmp(&right) {
                std::cmp::Ordering::Less => visible.next(),
                std::cmp::Ordering::Greater => base.next(),
                std::cmp::Ordering::Equal => {
                    base.next();
                    visible.next()
                }
            },
            (Some(_), None) => visible.next(),
            (None, Some(_)) => base.next(),
            (None, None) => None,
        },
    )
}

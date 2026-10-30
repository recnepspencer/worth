//! Canonical occupied logical slots across a sparse proposal and its base.
use super::InvariantStateView;
use crate::identity::data::PartitionId;

impl InvariantStateView<'_> {
    /// O(V+B) visits and O(log V + log B) paid iterator backing. Rejected
    /// reservations can make logical slots arbitrarily larger than row counts;
    /// neither the allocator frontier nor a dense ordinal is a scan bound.
    pub(crate) fn try_entity_slots<Stop>(
        &self,
        partition_id: PartitionId,
        mut prepare: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<impl Iterator<Item = usize> + '_, Stop> {
        let visible = self.state.get_current_partition(partition_id);
        let base = self
            .state
            .base_current_partition(partition_id)
            .filter(|base| visible.is_none_or(|visible| !base.same_region_view(visible)));
        let visible = visible
            .map(|partition| partition.entity_arena().try_occupied_slots(&mut prepare))
            .transpose()?;
        let base = base
            .map(|partition| partition.entity_arena().try_occupied_slots(&mut prepare))
            .transpose()?;
        Ok(merge_slots(
            visible.into_iter().flatten(),
            base.into_iter().flatten(),
        ))
    }

    pub(crate) fn try_relation_slots<Stop>(
        &self,
        partition_id: PartitionId,
        mut prepare: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<impl Iterator<Item = usize> + '_, Stop> {
        let visible = self.state.get_current_partition(partition_id);
        let base = self
            .state
            .base_current_partition(partition_id)
            .filter(|base| visible.is_none_or(|visible| !base.same_region_view(visible)));
        let visible = visible
            .map(|partition| partition.relation_arena().try_occupied_slots(&mut prepare))
            .transpose()?;
        let base = base
            .map(|partition| partition.relation_arena().try_occupied_slots(&mut prepare))
            .transpose()?;
        Ok(merge_slots(
            visible.into_iter().flatten(),
            base.into_iter().flatten(),
        ))
    }
}

fn merge_slots(
    visible: impl Iterator<Item = usize>,
    base: impl Iterator<Item = usize>,
) -> impl Iterator<Item = usize> {
    let mut visible = visible.peekable();
    let mut base = base.peekable();
    std::iter::from_fn(move || match (visible.peek(), base.peek()) {
        (Some(left), Some(right)) => match left.cmp(right) {
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
    })
}

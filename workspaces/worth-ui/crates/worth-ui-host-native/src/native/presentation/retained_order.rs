use std::collections::HashMap;
use std::hash::Hash;

use worth_ui_retained_order::UiRetainedOrderIndex;

pub(super) struct UiNativeRetainedOrder<Identity> {
    index: UiRetainedOrderIndex<Identity>,
}

pub(super) struct UiNativeRetainedOrderSnapshot<Identity> {
    entries: Vec<OriginalOrderEntry<Identity>>,
}

struct OriginalOrderEntry<Identity> {
    identity: Identity,
    predecessor: Option<Identity>,
    existed: bool,
    rank: Option<usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::native) enum UiNativeRetainedOrderDenial {
    DuplicateIdentity,
    MissingIdentity,
    MissingPredecessor,
    SelfPredecessor,
    CapacityExceeded,
}

impl<Identity> UiNativeRetainedOrder<Identity>
where
    Identity: Copy + Eq + Hash,
{
    pub(super) fn initial(
        identities: impl IntoIterator<Item = Identity>,
    ) -> Result<Self, UiNativeRetainedOrderDenial> {
        let mut order = Self {
            index: UiRetainedOrderIndex::new(usize::from(
                crate::UiNativeMechanicsCapacities::QUALIFIED.retained_commands,
            )),
        };
        let mut previous = None;
        for identity in identities {
            order.insert_after(identity, previous)?;
            previous = Some(identity);
        }
        Ok(order)
    }

    pub(super) fn remove(&mut self, identity: Identity) -> Result<(), UiNativeRetainedOrderDenial> {
        self.index
            .remove(identity)
            .then_some(())
            .ok_or(UiNativeRetainedOrderDenial::MissingIdentity)
    }

    pub(super) fn place_after(
        &mut self,
        identity: Identity,
        predecessor: Option<Identity>,
    ) -> Result<(), UiNativeRetainedOrderDenial> {
        if predecessor == Some(identity) {
            return Err(UiNativeRetainedOrderDenial::SelfPredecessor);
        }
        if predecessor.is_some_and(|value| !self.index.contains(value)) {
            return Err(UiNativeRetainedOrderDenial::MissingPredecessor);
        }
        if self.index.contains(identity) {
            self.remove(identity)?;
        }
        self.insert_after(identity, predecessor)
    }

    pub(super) fn ordered(&self) -> impl ExactSizeIterator<Item = Identity> + '_ {
        self.index.ordered()
    }

    pub(super) fn contains(&self, identity: Identity) -> bool {
        self.index.contains(identity)
    }

    pub(super) fn take_cost(&self) -> worth_ui_retained_order::UiRetainedOrderCost {
        self.index.take_cost()
    }

    pub(super) fn neighbors(
        &self,
        identity: Identity,
    ) -> Result<(Option<Identity>, Option<Identity>), UiNativeRetainedOrderDenial> {
        let rank = self
            .index
            .rank(identity)
            .ok_or(UiNativeRetainedOrderDenial::MissingIdentity)?;
        Ok((
            rank.checked_sub(1)
                .and_then(|value| self.index.identity_at(value)),
            self.index.identity_at(rank + 1),
        ))
    }

    pub(super) fn successor_after(
        &self,
        predecessor: Option<Identity>,
    ) -> Result<Option<Identity>, UiNativeRetainedOrderDenial> {
        match predecessor {
            Some(predecessor) => self
                .index
                .rank(predecessor)
                .map(|rank| self.index.identity_at(rank + 1))
                .ok_or(UiNativeRetainedOrderDenial::MissingPredecessor),
            None => Ok(self.index.identity_at(0)),
        }
    }

    pub(super) fn ordered_subset(
        &self,
        identities: impl IntoIterator<Item = Identity>,
    ) -> Result<Vec<Identity>, UiNativeRetainedOrderDenial> {
        let mut ranked = identities
            .into_iter()
            .map(|identity| {
                self.index
                    .rank(identity)
                    .map(|rank| (rank, identity))
                    .ok_or(UiNativeRetainedOrderDenial::MissingIdentity)
            })
            .collect::<Result<Vec<_>, _>>()?;
        ranked.sort_unstable_by_key(|entry| entry.0);
        Ok(ranked.into_iter().map(|entry| entry.1).collect())
    }

    pub(super) fn snapshot(
        &self,
        identities: impl IntoIterator<Item = Identity>,
    ) -> UiNativeRetainedOrderSnapshot<Identity> {
        let mut seen = HashMap::new();
        let mut entries = Vec::new();
        for identity in identities {
            if seen.insert(identity, ()).is_some() {
                continue;
            }
            let rank = self.index.rank(identity);
            let predecessor = rank
                .and_then(|value| value.checked_sub(1))
                .and_then(|value| self.index.identity_at(value));
            entries.push(OriginalOrderEntry {
                identity,
                predecessor,
                existed: rank.is_some(),
                rank,
            });
        }
        entries.sort_unstable_by_key(|entry| entry.rank);
        UiNativeRetainedOrderSnapshot { entries }
    }

    pub(super) fn restore(
        &mut self,
        snapshot: UiNativeRetainedOrderSnapshot<Identity>,
    ) -> Result<(), UiNativeRetainedOrderDenial> {
        for entry in &snapshot.entries {
            if self.contains(entry.identity) {
                self.remove(entry.identity)?;
            }
        }
        for entry in snapshot.entries.into_iter().filter(|entry| entry.existed) {
            self.insert_after(entry.identity, entry.predecessor)?;
        }
        Ok(())
    }

    fn insert_after(
        &mut self,
        identity: Identity,
        predecessor: Option<Identity>,
    ) -> Result<(), UiNativeRetainedOrderDenial> {
        if self.index.contains(identity) {
            return Err(UiNativeRetainedOrderDenial::DuplicateIdentity);
        }
        let rank = match predecessor {
            Some(predecessor) => self
                .index
                .rank(predecessor)
                .map(|rank| rank + 1)
                .ok_or(UiNativeRetainedOrderDenial::MissingPredecessor)?,
            None => 0,
        };
        self.index
            .insert_at(rank, identity)
            .map_err(|denial| match denial {
                worth_ui_retained_order::UiRetainedOrderDenial::CapacityExceeded => {
                    UiNativeRetainedOrderDenial::CapacityExceeded
                }
                worth_ui_retained_order::UiRetainedOrderDenial::DuplicateIdentity => {
                    UiNativeRetainedOrderDenial::DuplicateIdentity
                }
                worth_ui_retained_order::UiRetainedOrderDenial::InvalidRank => {
                    UiNativeRetainedOrderDenial::MissingPredecessor
                }
            })
    }

    #[cfg(test)]
    fn height(&self) -> usize {
        self.index.height()
    }
}

#[cfg(test)]
mod tests {
    use super::{UiNativeRetainedOrder, UiNativeRetainedOrderDenial};

    #[test]
    fn repeated_front_middle_and_tail_edits_preserve_authored_order() {
        let mut order = UiNativeRetainedOrder::initial([1_u64, 2, 3]).unwrap();
        order.place_after(4, None).unwrap();
        order.place_after(3, Some(1)).unwrap();
        order.place_after(5, Some(2)).unwrap();
        order.remove(1).unwrap();
        assert_eq!(order.ordered().collect::<Vec<_>>(), vec![4, 3, 2, 5]);
    }

    #[test]
    fn invalid_edits_deny_without_mutating_retained_order() {
        let mut order = UiNativeRetainedOrder::initial([1_u64, 2]).unwrap();
        assert_eq!(
            order.place_after(1, Some(9)),
            Err(UiNativeRetainedOrderDenial::MissingPredecessor)
        );
        assert_eq!(order.ordered().collect::<Vec<_>>(), vec![1, 2]);
        assert_eq!(
            order.place_after(1, Some(1)),
            Err(UiNativeRetainedOrderDenial::SelfPredecessor)
        );
        assert_eq!(
            order.remove(9),
            Err(UiNativeRetainedOrderDenial::MissingIdentity)
        );
    }

    #[test]
    fn repeated_insertions_into_one_gap_keep_a_bounded_balanced_index() {
        let mut order = UiNativeRetainedOrder::initial([1_u64, 2]).unwrap();
        for identity in 3..=4_096 {
            order.place_after(identity, Some(1)).unwrap();
        }
        assert_eq!(order.ordered().count(), 4_096);
        assert_eq!(
            order.ordered().take(4).collect::<Vec<_>>(),
            vec![1, 4_096, 4_095, 4_094]
        );
        assert!(
            order.height() <= 18,
            "AVL height exceeded its logarithmic bound"
        );
        let expected = order.ordered().collect::<Vec<_>>();
        order.take_cost();
        assert_eq!(
            order.place_after(4_097, Some(1)),
            Err(UiNativeRetainedOrderDenial::CapacityExceeded)
        );
        let denied = order.take_cost();
        assert_eq!(denied.live_entries(), 4_096);
        assert_eq!(denied.allocated_slots(), 4_096);
        assert_eq!(denied.high_water_entries(), 4_096);
        assert_eq!(denied.rotations(), 0);
        assert_eq!(order.ordered().collect::<Vec<_>>(), expected);
    }
}

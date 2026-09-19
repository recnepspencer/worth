//! Bounded indexed sequence mechanics shared by retained UI consumers.
//!
//! This crate owns no UI identity, protocol, runtime truth, or host authority.
//! Native and headless hosts wrap this index with their own semantic validation.

#![forbid(unsafe_code)]

mod cost;
mod index;

use std::hash::Hash;

use index::BoundedOrderIndex;

pub use cost::UiRetainedOrderCost;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiRetainedOrderDenial {
    CapacityExceeded,
    DuplicateIdentity,
    InvalidRank,
}

pub struct UiRetainedOrderIndex<Identity> {
    index: BoundedOrderIndex<Identity>,
}

impl<Identity> UiRetainedOrderIndex<Identity>
where
    Identity: Copy + Eq + Hash,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            index: BoundedOrderIndex::new(capacity),
        }
    }

    pub fn contains(&self, identity: Identity) -> bool {
        self.index.contains(identity)
    }

    pub fn insert_at(
        &mut self,
        rank: usize,
        identity: Identity,
    ) -> Result<(), UiRetainedOrderDenial> {
        self.index.insert_at(rank, identity)
    }

    pub fn insert_at_weighted(
        &mut self,
        rank: usize,
        identity: Identity,
        weight: u32,
    ) -> Result<(), UiRetainedOrderDenial> {
        self.index.insert_at_weighted(rank, identity, weight)
    }

    pub fn remove(&mut self, identity: Identity) -> bool {
        self.index.remove(identity)
    }

    pub fn rank(&self, identity: Identity) -> Option<usize> {
        self.index.rank(identity)
    }

    pub fn weight(&self, identity: Identity) -> Option<u32> {
        self.index.weight(identity)
    }

    pub fn update_weight(&mut self, identity: Identity, weight: u32) -> bool {
        self.index.update_weight(identity, weight)
    }

    pub fn first_with_weight_at_least(&self, minimum: u32) -> Option<Identity> {
        self.index.first_with_weight_at_least(minimum)
    }

    pub fn identity_at(&self, rank: usize) -> Option<Identity> {
        self.index.identity_at(rank)
    }

    pub fn ordered(&self) -> impl ExactSizeIterator<Item = Identity> + '_ {
        self.index.ordered()
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn take_cost(&self) -> UiRetainedOrderCost {
        self.index.take_cost()
    }

    #[doc(hidden)]
    pub fn height(&self) -> usize {
        self.index.height()
    }
}

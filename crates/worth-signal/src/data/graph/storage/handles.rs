use serde::{Deserialize, Serialize};

mod retained_charge;
use std::num::NonZeroU32;

pub(crate) trait SetHandle:
    Copy + Eq + std::hash::Hash + Serialize + for<'de> Deserialize<'de>
{
    const EMPTY: Self;

    fn from_index(index: usize) -> Self;
    fn index(self) -> Option<usize>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct DependencySetId(Option<NonZeroU32>);

impl DependencySetId {
    pub const EMPTY: Self = Self(None);
}

impl SetHandle for DependencySetId {
    const EMPTY: Self = Self::EMPTY;

    fn from_index(index: usize) -> Self {
        debug_assert!(index > 0);
        Self(NonZeroU32::new(index as u32))
    }

    fn index(self) -> Option<usize> {
        self.0.map(|index| index.get() as usize)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct SubscriberSetId(Option<NonZeroU32>);

impl SubscriberSetId {
    pub const EMPTY: Self = Self(None);
}

impl SetHandle for SubscriberSetId {
    const EMPTY: Self = Self::EMPTY;

    fn from_index(index: usize) -> Self {
        debug_assert!(index > 0);
        Self(NonZeroU32::new(index as u32))
    }

    fn index(self) -> Option<usize> {
        self.0.map(|index| index.get() as usize)
    }
}

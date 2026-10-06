use std::marker::PhantomData;

use crate::aspects::structs::CanonicalFieldPath;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProjectionMask;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MutationMask;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DiagnosticMask;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AspectMask<Mode> {
    paths: Vec<CanonicalFieldPath>,
    mode: PhantomData<Mode>,
}

impl<Mode> AspectMask<Mode> {
    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        self.paths
            .capacity()
            .saturating_mul(std::mem::size_of::<CanonicalFieldPath>())
            .saturating_add(
                self.paths
                    .iter()
                    .map(CanonicalFieldPath::owned_allocation_capacity_bytes)
                    .sum::<usize>(),
            )
    }

    pub fn new(paths: impl IntoIterator<Item = CanonicalFieldPath>) -> Self {
        let mut paths: Vec<_> = paths.into_iter().collect();
        paths.sort();
        paths.dedup();
        Self {
            paths,
            mode: PhantomData,
        }
    }

    pub fn whole_aspect() -> Self {
        Self {
            paths: Vec::new(),
            mode: PhantomData,
        }
    }

    pub fn paths(&self) -> &[CanonicalFieldPath] {
        &self.paths
    }

    pub fn is_whole_aspect(&self) -> bool {
        self.paths.is_empty()
    }
}

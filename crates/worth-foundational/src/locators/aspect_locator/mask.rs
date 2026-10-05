use std::marker::PhantomData;

use crate::aspects::{
    AspectKey, AspectMask, CanonicalFieldPath, DiagnosticMask, MutationMask, ProjectionMask,
};

use super::super::LocatorAuthority;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AspectMaskLocator<Mode> {
    authority: LocatorAuthority,
    aspect_key: AspectKey,
    paths: Vec<CanonicalFieldPath>,
    mode: PhantomData<Mode>,
}

impl<Mode> AspectMaskLocator<Mode> {
    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        self.aspect_key
            .owned_allocation_capacity_bytes()
            .saturating_add(
                self.paths
                    .capacity()
                    .saturating_mul(std::mem::size_of::<CanonicalFieldPath>()),
            )
            .saturating_add(
                self.paths
                    .iter()
                    .map(CanonicalFieldPath::owned_allocation_capacity_bytes)
                    .sum::<usize>(),
            )
    }

    pub fn paths(&self) -> &[CanonicalFieldPath] {
        &self.paths
    }

    pub const fn authority(&self) -> LocatorAuthority {
        self.authority
    }

    pub fn aspect_key(&self) -> &AspectKey {
        &self.aspect_key
    }
}

impl AspectMaskLocator<ProjectionMask> {
    pub fn projection(
        authority: LocatorAuthority,
        aspect_key: AspectKey,
        mask: &AspectMask<ProjectionMask>,
    ) -> Self {
        Self::from_mask(authority, aspect_key, mask)
    }
}

impl AspectMaskLocator<MutationMask> {
    pub fn mutation(
        authority: LocatorAuthority,
        aspect_key: AspectKey,
        mask: &AspectMask<MutationMask>,
    ) -> Self {
        Self::from_mask(authority, aspect_key, mask)
    }
}

impl AspectMaskLocator<DiagnosticMask> {
    pub fn diagnostic(
        authority: LocatorAuthority,
        aspect_key: AspectKey,
        mask: &AspectMask<DiagnosticMask>,
    ) -> Self {
        Self::from_mask(authority, aspect_key, mask)
    }
}

impl<Mode> AspectMaskLocator<Mode> {
    fn from_mask(
        authority: LocatorAuthority,
        aspect_key: AspectKey,
        mask: &AspectMask<Mode>,
    ) -> Self {
        Self {
            authority,
            aspect_key,
            paths: mask.paths().to_vec(),
            mode: PhantomData,
        }
    }
}

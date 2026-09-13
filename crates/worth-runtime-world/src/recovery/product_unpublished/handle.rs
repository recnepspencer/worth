use crate::identity::ProductUnpublishedOwnerEffectsIdentity;

/// A non-authorizing handle that lets a caller retain or inspect a specific
/// recovery record without turning it into a product commit.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProductUnpublishedRecoveryHandle {
    identity: ProductUnpublishedOwnerEffectsIdentity,
    catalog_affinity: usize,
}

impl ProductUnpublishedRecoveryHandle {
    pub(crate) const fn new(
        identity: ProductUnpublishedOwnerEffectsIdentity,
        catalog_affinity: usize,
    ) -> Self {
        Self {
            identity,
            catalog_affinity,
        }
    }

    pub fn identity(&self) -> &ProductUnpublishedOwnerEffectsIdentity {
        &self.identity
    }

    pub(crate) const fn catalog_affinity(&self) -> usize {
        self.catalog_affinity
    }
}

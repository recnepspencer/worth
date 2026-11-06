//! Original target-meaning coordinates; these values grant no live authority.
use super::{identity_seal, DurableIdentity};

/// Commitment to the complete Native schema, invariant and Source inventory.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HomeProgramTransitionCatalogDigest([u8; 32]);

impl HomeProgramTransitionCatalogDigest {
    pub const fn from_recorded(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}
impl identity_seal::Sealed for HomeProgramTransitionCatalogDigest {}
impl DurableIdentity for HomeProgramTransitionCatalogDigest {}

/// The unique step that installs the successor's complete executable meaning.
/// Its validation includes unchanged retained rows and relations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HomeProgramTransitionTargetMeaning {
    step: u32,
    catalog: HomeProgramTransitionCatalogDigest,
}

impl HomeProgramTransitionTargetMeaning {
    pub const fn from_recorded(step: u32, catalog: HomeProgramTransitionCatalogDigest) -> Self {
        Self { step, catalog }
    }

    pub const fn step(self) -> u32 {
        self.step
    }

    pub const fn catalog(self) -> HomeProgramTransitionCatalogDigest {
        self.catalog
    }

    /// The owner-lowered policy version is part of every complete plan.
    pub const fn whole_candidate_validation_version(self) -> u32 {
        1
    }
}

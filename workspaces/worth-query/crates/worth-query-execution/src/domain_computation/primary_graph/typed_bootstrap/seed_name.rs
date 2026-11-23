//! The persistent name a bootstrap seed declares for its record.
use worth_query_declaration::facade::application_naming::ApplicationNameDiscriminator;

/// The declared role (and optional typed discriminator) a seed's record is
/// born under; the seed's application key is its authored occurrence.
pub(in crate::domain_computation::primary_graph) struct SeedPersistentName {
    pub(in crate::domain_computation::primary_graph) role: String,
    pub(in crate::domain_computation::primary_graph) discriminator: Vec<u8>,
    pub(in crate::domain_computation::primary_graph) references: Vec<[u8; 32]>,
}

impl SeedPersistentName {
    pub(super) fn plain(role: &str) -> Self {
        Self {
            role: role.to_owned(),
            discriminator: Vec::new(),
            references: Vec::new(),
        }
    }

    pub(super) fn discriminated<D: ApplicationNameDiscriminator>(
        role: &str,
        discriminator: &D,
    ) -> Self {
        let (discriminator, references) = discriminator.canonical_bytes_with_references();
        Self {
            role: role.to_owned(),
            discriminator,
            references,
        }
    }

    pub(super) fn retained_bytes(&self) -> usize {
        self.role
            .capacity()
            .saturating_add(self.discriminator.capacity())
            .saturating_add(self.references.capacity().saturating_mul(32))
    }
}

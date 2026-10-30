pub use worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial;
pub use {sha2::Digest, sha2::Sha256};

pub fn certification_material(
    scenario: &'static str,
    publication_ordinal: u64,
) -> PhysicalMutationIdempotencyMaterial {
    let mut digest = sha2::Sha256::new();
    digest.update(b"worth-store.integration-certification-publication.v1");
    digest.update(scenario.as_bytes());
    digest.update(publication_ordinal.to_le_bytes());
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new(
        digest.finalize().into(),
    )
}

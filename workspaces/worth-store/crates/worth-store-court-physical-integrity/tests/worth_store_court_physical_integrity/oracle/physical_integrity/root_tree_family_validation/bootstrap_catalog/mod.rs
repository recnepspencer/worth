#[allow(unused_imports)]
use crate::root_tree_family_validation::bootstrap_catalog::*;

#[test]
fn catalog_literal_vector_is_independent_of_writer_and_validates_directly() {
    assert_eq!(bootstrap_bytes(store(7)), BOOTSTRAP_CATALOG_LITERAL);
    assert_eq!(
        independent_crc32c(&[&BOOTSTRAP_CATALOG_LITERAL]),
        BOOTSTRAP_CATALOG_COMPLETE_CRC32C
    );
    assert_eq!(
        u32::from_le_bytes(BOOTSTRAP_CATALOG_LITERAL[44..48].try_into().unwrap()),
        independent_crc32c(&[
            &BOOTSTRAP_CATALOG_LITERAL[..44],
            &BOOTSTRAP_CATALOG_LITERAL[48..],
        ])
    );
    match validate_bootstrap_catalog(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(
            &BOOTSTRAP_CATALOG_LITERAL,
        ),
        bootstrap_scope(store(7), BOOTSTRAP_OFFSET),
    )
    .0
    {
        worth_store_physical_integrity::BootstrapCatalogIntegrityValidation::Intact(validated) => {
            assert_eq!(validated.current_root_generation().get(), 11);
            let record = validated.into_validation_record();
            assert_eq!(
                record.byte_range_digest(),
                worth_store_physical_integrity::PhysicalIntegrityValidationDigest::crc32c(
                    BOOTSTRAP_CATALOG_COMPLETE_CRC32C
                )
            );
            assert_eq!(
                record.mechanism(),
                worth_store_physical_integrity::PhysicalIntegrityValidationMechanism::Crc32cV1
            );
            assert_eq!(record.mechanism().version(), 1);
        }
        worth_store_physical_integrity::BootstrapCatalogIntegrityValidation::Rejected(
            rejection,
        ) => {
            panic!("literal bootstrap catalog rejected: {rejection:?}")
        }
        worth_store_physical_integrity::BootstrapCatalogIntegrityValidation::ScopeMismatch(
            mismatch,
        ) => {
            panic!("literal bootstrap catalog mismatched scope: {mismatch:?}")
        }
        worth_store_physical_integrity::BootstrapCatalogIntegrityValidation::UnsupportedFormat(
            unsupported,
        ) => {
            panic!("literal bootstrap catalog has unsupported format: {unsupported:?}")
        }
    }
}

//! C.11 inner-frame declarations. Each frame is also carried by an extent-backed
//! C.5 record; this declaration covers the inner frame, not its selected route.

use crate::integrity_declarations::{
    PhysicalIntegrityAlgorithm, PhysicalIntegrityArtifactFamily,
    PhysicalIntegrityChecksumDeclaration, PhysicalIntegrityChecksumField,
    PhysicalIntegrityCoverageBoundary as Boundary, PhysicalIntegrityCoveredRange,
    PhysicalIntegrityFormatDeclaration, PhysicalIntegrityFormatVersion,
};
use crate::BlobRecordKind;

/// Whether `code` is an inner-frame kind this format writes. A selected route
/// classifies a blob record by the same code. An independent reader admits
/// exactly these codes, so a new kind cannot leave it behind unnoticed.
pub const fn blob_record_kind_is_declared(code: u8) -> bool {
    BlobRecordKind::from_code(code).is_some()
}

const ENVELOPE_RANGES: &[PhysicalIntegrityCoveredRange] = &[
    PhysicalIntegrityCoveredRange::new(Boundary::Fixed(0), Boundary::Fixed(16)),
    PhysicalIntegrityCoveredRange::new(Boundary::Fixed(48), Boundary::ArtifactEnd),
];
const CANONICAL_CONTENT_RANGE: &[PhysicalIntegrityCoveredRange] =
    &[PhysicalIntegrityCoveredRange::new(
        Boundary::Fixed(128),
        Boundary::ArtifactEnd,
    )];
const ENVELOPE_SHA256: PhysicalIntegrityChecksumDeclaration =
    PhysicalIntegrityChecksumDeclaration::new(
        PhysicalIntegrityAlgorithm::Sha256,
        ENVELOPE_RANGES,
        PhysicalIntegrityChecksumField::new(Boundary::Fixed(16), Boundary::Fixed(48)),
    );
const CANONICAL_CONTENT_SHA256: PhysicalIntegrityChecksumDeclaration =
    PhysicalIntegrityChecksumDeclaration::new(
        PhysicalIntegrityAlgorithm::Sha256,
        CANONICAL_CONTENT_RANGE,
        PhysicalIntegrityChecksumField::new(Boundary::Fixed(96), Boundary::Fixed(128)),
    );
const CONTROL_CHECKSUMS: &[PhysicalIntegrityChecksumDeclaration] = &[ENVELOPE_SHA256];
const DATA_CHECKSUMS: &[PhysicalIntegrityChecksumDeclaration] =
    &[ENVELOPE_SHA256, CANONICAL_CONTENT_SHA256];

pub const BLOB_RESUME_SESSION_INTEGRITY_DECLARATION: PhysicalIntegrityFormatDeclaration =
    PhysicalIntegrityFormatDeclaration::new(
        PhysicalIntegrityArtifactFamily::BlobResumeSession,
        PhysicalIntegrityFormatVersion::new(1, Some(1)),
        CONTROL_CHECKSUMS,
    );
pub const BLOB_CHUNK_FRAME_INTEGRITY_DECLARATION: PhysicalIntegrityFormatDeclaration =
    PhysicalIntegrityFormatDeclaration::new(
        PhysicalIntegrityArtifactFamily::BlobChunkFrame,
        PhysicalIntegrityFormatVersion::new(1, Some(1)),
        DATA_CHECKSUMS,
    );
pub const BLOB_TREE_NODE_INTEGRITY_DECLARATION: PhysicalIntegrityFormatDeclaration =
    PhysicalIntegrityFormatDeclaration::new(
        PhysicalIntegrityArtifactFamily::BlobTreeNode,
        PhysicalIntegrityFormatVersion::new(1, Some(1)),
        DATA_CHECKSUMS,
    );
pub const BLOB_GENERATION_PUBLICATION_INTEGRITY_DECLARATION: PhysicalIntegrityFormatDeclaration =
    PhysicalIntegrityFormatDeclaration::new(
        PhysicalIntegrityArtifactFamily::BlobGenerationPublication,
        PhysicalIntegrityFormatVersion::new(1, Some(1)),
        CONTROL_CHECKSUMS,
    );
pub const BLOB_DROP_SET_MANIFEST_INTEGRITY_DECLARATION: PhysicalIntegrityFormatDeclaration =
    PhysicalIntegrityFormatDeclaration::new(
        PhysicalIntegrityArtifactFamily::BlobDropSetManifest,
        PhysicalIntegrityFormatVersion::new(1, Some(1)),
        CONTROL_CHECKSUMS,
    );
pub const BLOB_RECLAIM_DESCRIPTOR_INTEGRITY_DECLARATION: PhysicalIntegrityFormatDeclaration =
    PhysicalIntegrityFormatDeclaration::new(
        PhysicalIntegrityArtifactFamily::BlobReclaimDescriptor,
        PhysicalIntegrityFormatVersion::new(1, Some(1)),
        CONTROL_CHECKSUMS,
    );

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declared_kind_codes_are_exactly_the_kinds_the_record_decoder_names() {
        for code in 0..=u8::MAX {
            let kind = BlobRecordKind::from_code(code);
            assert_eq!(
                blob_record_kind_is_declared(code),
                kind.is_some(),
                "code {code}"
            );
            if let Some(kind) = kind {
                assert_eq!(kind as u8, code, "a kind decodes from its own code");
            }
        }
        let newest = BlobRecordKind::ReclaimDescriptorV3 as u8;
        assert!(blob_record_kind_is_declared(
            BlobRecordKind::ChunkReuseClaimV2 as u8
        ));
        assert!(blob_record_kind_is_declared(newest));
        assert!(!blob_record_kind_is_declared(0));
        assert!(!blob_record_kind_is_declared(newest + 1));
    }

    #[test]
    fn chunk_and_tree_declare_both_outer_and_canonical_content_sha() {
        for declaration in [
            BLOB_CHUNK_FRAME_INTEGRITY_DECLARATION,
            BLOB_TREE_NODE_INTEGRITY_DECLARATION,
        ] {
            assert_eq!(declaration.checksums(), DATA_CHECKSUMS);
            let canonical = declaration.checksums()[1];
            assert_eq!(canonical.algorithm(), PhysicalIntegrityAlgorithm::Sha256);
            assert_eq!(canonical.covered_ranges(), CANONICAL_CONTENT_RANGE);
            assert_eq!(
                canonical.field(),
                PhysicalIntegrityChecksumField::new(Boundary::Fixed(96), Boundary::Fixed(128))
            );
        }
        assert_eq!(
            BLOB_RESUME_SESSION_INTEGRITY_DECLARATION.checksums(),
            CONTROL_CHECKSUMS
        );
        assert_eq!(
            BLOB_GENERATION_PUBLICATION_INTEGRITY_DECLARATION.checksums(),
            CONTROL_CHECKSUMS
        );
    }
}

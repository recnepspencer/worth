use worth_store_physical_format::integrity_declarations::{
    families::{
        checkpoint::*, free_space::*, root::*, EXTENT_CHUNK_INTEGRITY_DECLARATION,
        EXTENT_MANIFEST_INTEGRITY_DECLARATION, NAMESPACE_IDENTITY_INTEGRITY_DECLARATION,
        PAGE_FRAME_INTEGRITY_DECLARATION, PHYSICAL_WORK_OBLIGATION_INTEGRITY_DECLARATION,
        SEGMENT_MEMBERSHIP_INTEGRITY_DECLARATION, WAL_FRAME_INTEGRITY_DECLARATION,
    },
    PhysicalIntegrityAlgorithm, PhysicalIntegrityArtifactFamily, PhysicalIntegrityCoverageBoundary,
};

#[test]
fn every_frozen_current_family_has_an_exact_named_declaration() {
    let declarations = [
        NAMESPACE_IDENTITY_INTEGRITY_DECLARATION,
        PHYSICAL_WORK_OBLIGATION_INTEGRITY_DECLARATION,
        PAGE_FRAME_INTEGRITY_DECLARATION,
        EXTENT_CHUNK_INTEGRITY_DECLARATION,
        WAL_FRAME_INTEGRITY_DECLARATION,
        CHECKPOINT_STREAM_HEADER_INTEGRITY_DECLARATION,
        CHECKPOINT_DIRTY_BASIS_INTEGRITY_DECLARATION,
        CHECKPOINT_BINDING_COMPACTION_INTEGRITY_DECLARATION,
        CHECKPOINT_BINDING_INTEGRITY_DECLARATION,
        CHECKPOINT_FOOTER_INTEGRITY_DECLARATION,
        BOOTSTRAP_CATALOG_INTEGRITY_DECLARATION,
        CURRENT_SELECTOR_INTEGRITY_DECLARATION,
        PREVIOUS_SELECTOR_INTEGRITY_DECLARATION,
        ROOT_MANIFEST_INTEGRITY_DECLARATION,
        ROOT_ROUTING_BLOCK_INTEGRITY_DECLARATION,
        SEGMENT_MEMBERSHIP_INTEGRITY_DECLARATION,
        EXTENT_MANIFEST_INTEGRITY_DECLARATION,
        FREE_SPACE_HEADER_INTEGRITY_DECLARATION,
        FREE_SPACE_MEMBERSHIP_BLOCK_INTEGRITY_DECLARATION,
    ];
    let expected = [
        (
            PhysicalIntegrityArtifactFamily::NamespaceIdentity,
            1,
            None,
            1,
        ),
        (
            PhysicalIntegrityArtifactFamily::PhysicalWorkObligation,
            6,
            None,
            1,
        ),
        (PhysicalIntegrityArtifactFamily::PageFrame, 1, Some(2), 1),
        (PhysicalIntegrityArtifactFamily::ExtentChunk, 1, Some(2), 1),
        (PhysicalIntegrityArtifactFamily::WalFrame, 1, None, 2),
        (
            PhysicalIntegrityArtifactFamily::CheckpointStreamHeader,
            1,
            None,
            1,
        ),
        (
            PhysicalIntegrityArtifactFamily::CheckpointDirtyBasis,
            1,
            None,
            1,
        ),
        (
            PhysicalIntegrityArtifactFamily::CheckpointBindingCompaction,
            1,
            None,
            1,
        ),
        (
            PhysicalIntegrityArtifactFamily::CheckpointBinding,
            1,
            None,
            1,
        ),
        (
            PhysicalIntegrityArtifactFamily::CheckpointFooter,
            1,
            None,
            1,
        ),
        (
            PhysicalIntegrityArtifactFamily::BootstrapCatalog,
            1,
            Some(2),
            1,
        ),
        (
            PhysicalIntegrityArtifactFamily::CurrentRootSelector,
            1,
            Some(2),
            1,
        ),
        (
            PhysicalIntegrityArtifactFamily::PreviousRootSelector,
            1,
            Some(2),
            1,
        ),
        (PhysicalIntegrityArtifactFamily::RootManifest, 1, Some(2), 1),
        (
            PhysicalIntegrityArtifactFamily::RootRoutingBlock,
            1,
            Some(2),
            1,
        ),
        (
            PhysicalIntegrityArtifactFamily::SegmentMembership,
            1,
            Some(2),
            1,
        ),
        (
            PhysicalIntegrityArtifactFamily::ExtentManifest,
            1,
            Some(2),
            1,
        ),
        (
            PhysicalIntegrityArtifactFamily::FreeSpaceHeader,
            1,
            Some(2),
            1,
        ),
        (
            PhysicalIntegrityArtifactFamily::FreeSpaceMembershipBlock,
            1,
            Some(2),
            1,
        ),
    ];
    let actual = declarations.map(|declaration| {
        (
            declaration.family(),
            declaration.version().format_version(),
            declaration.version().envelope_schema(),
            declaration.checksums().len(),
        )
    });
    assert_eq!(actual, expected);
}

#[test]
fn common_frame_declarations_name_split_crc32c_coverage() {
    let declaration = CURRENT_SELECTOR_INTEGRITY_DECLARATION;
    assert_eq!(
        declaration.family(),
        PhysicalIntegrityArtifactFamily::CurrentRootSelector
    );
    assert_eq!(declaration.version().envelope_schema(), Some(2));
    let checksum = declaration.checksums()[0];
    assert_eq!(checksum.algorithm(), PhysicalIntegrityAlgorithm::Crc32c);
    assert_eq!(checksum.covered_ranges().len(), 2);
    assert_eq!(
        checksum.covered_ranges()[0].end(),
        PhysicalIntegrityCoverageBoundary::Fixed(44)
    );
    assert_eq!(
        checksum.covered_ranges()[1].start(),
        PhysicalIntegrityCoverageBoundary::Fixed(48)
    );
}

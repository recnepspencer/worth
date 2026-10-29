use {worth_store_physical_format::integrity_declarations::families::checkpoint::*, worth_store_physical_format::integrity_declarations::families::free_space::*, worth_store_physical_format::integrity_declarations::families::root::*, worth_store_physical_format::integrity_declarations::families::EXTENT_CHUNK_INTEGRITY_DECLARATION, worth_store_physical_format::integrity_declarations::families::EXTENT_MANIFEST_INTEGRITY_DECLARATION, worth_store_physical_format::integrity_declarations::families::NAMESPACE_IDENTITY_INTEGRITY_DECLARATION, worth_store_physical_format::integrity_declarations::families::PAGE_FRAME_INTEGRITY_DECLARATION, worth_store_physical_format::integrity_declarations::families::PHYSICAL_WORK_OBLIGATION_INTEGRITY_DECLARATION, worth_store_physical_format::integrity_declarations::families::SEGMENT_MEMBERSHIP_INTEGRITY_DECLARATION, worth_store_physical_format::integrity_declarations::families::WAL_FRAME_INTEGRITY_DECLARATION, worth_store_physical_format::integrity_declarations::PhysicalIntegrityAlgorithm, worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily, worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary};

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
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::NamespaceIdentity,
            1,
            None,
            1,
        ),
        (
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::PhysicalWorkObligation,
            6,
            None,
            1,
        ),
        (worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::PageFrame, 2, Some(2), 1),
        (worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::ExtentChunk, 2, Some(2), 1),
        (worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::WalFrame, 1, None, 2),
        (
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointStreamHeader,
            1,
            None,
            1,
        ),
        (
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointDirtyBasis,
            1,
            None,
            1,
        ),
        (
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointBindingCompaction,
            1,
            None,
            1,
        ),
        (
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointBinding,
            1,
            None,
            1,
        ),
        (
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointFooter,
            1,
            None,
            1,
        ),
        (
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::BootstrapCatalog,
            2,
            Some(2),
            1,
        ),
        (
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CurrentRootSelector,
            2,
            Some(2),
            1,
        ),
        (
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::PreviousRootSelector,
            2,
            Some(2),
            1,
        ),
        (worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::RootManifest, 2, Some(2), 1),
        (
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::RootRoutingBlock,
            2,
            Some(2),
            1,
        ),
        (
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::SegmentMembership,
            2,
            Some(2),
            1,
        ),
        (
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::ExtentManifest,
            2,
            Some(2),
            1,
        ),
        (
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::FreeSpaceHeader,
            2,
            Some(2),
            1,
        ),
        (
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::FreeSpaceMembershipBlock,
            2,
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
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CurrentRootSelector
    );
    assert_eq!(declaration.version().envelope_schema(), Some(2));
    let checksum = declaration.checksums()[0];
    assert_eq!(checksum.algorithm(), worth_store_physical_format::integrity_declarations::PhysicalIntegrityAlgorithm::Crc32c);
    assert_eq!(checksum.covered_ranges().len(), 2);
    assert_eq!(
        checksum.covered_ranges()[0].end(),
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary::Fixed(44)
    );
    assert_eq!(
        checksum.covered_ranges()[1].start(),
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary::Fixed(48)
    );
}

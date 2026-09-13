use core::num::NonZeroU64;

use worth_store_physical_format::integrity_declarations::{
    PhysicalIntegrityArtifactFamily, PhysicalIntegrityFormatDeclaration,
};
use worth_store_physical_format::{
    PhysicalCheckpointIdentity, PhysicalWorkObligationIdentity, WalSegmentIdentity,
};
use worth_store_physical_integrity::{
    CheckpointStreamHeaderScopeIdentity, PhysicalArtifactScope, PhysicalByteRange,
    PhysicalDamageCause, PhysicalFormatField, PhysicalIntegrityVersionAxis,
};

#[path = "physical_artifact_scope_contract/canonical_scope_fixtures.rs"]
mod canonical_scope_fixtures;
use canonical_scope_fixtures::*;

fn assert_scope(
    scope: PhysicalArtifactScope,
    expected_family: PhysicalIntegrityArtifactFamily,
    expected_declaration: PhysicalIntegrityFormatDeclaration,
) {
    assert_eq!(scope.artifact_family(), expected_family);
    assert_eq!(scope.declaration(), expected_declaration);
    assert_eq!(scope.format_version(), expected_declaration.version());
}

#[test]
fn every_phase_four_family_has_a_canonical_scope_and_version_contract() {
    use worth_store_physical_format::integrity_declarations::families::{
        checkpoint::*, free_space::*, root::*, *,
    };

    let store = store(7);
    let format = format();
    let work = PhysicalWorkObligationIdentity::new(
        NonZeroU64::new(1).unwrap(),
        NonZeroU64::new(2).unwrap(),
        NonZeroU64::new(3).unwrap(),
    );
    let checkpoint_header = CheckpointStreamHeaderScopeIdentity::staged(store);
    let checkpoint = PhysicalCheckpointIdentity::new(store, NonZeroU64::new(24).unwrap());
    let scopes = [
        (
            PhysicalArtifactScope::physical_work_obligation(store, work, range(0)),
            PhysicalIntegrityArtifactFamily::PhysicalWorkObligation,
            PHYSICAL_WORK_OBLIGATION_INTEGRITY_DECLARATION,
        ),
        (
            PhysicalArtifactScope::bootstrap_catalog(store, format, range(64)),
            PhysicalIntegrityArtifactFamily::BootstrapCatalog,
            BOOTSTRAP_CATALOG_INTEGRITY_DECLARATION,
        ),
        (
            PhysicalArtifactScope::root_routing_block(
                store,
                format,
                root_routing_identity(),
                range(128),
            ),
            PhysicalIntegrityArtifactFamily::RootRoutingBlock,
            ROOT_ROUTING_BLOCK_INTEGRITY_DECLARATION,
        ),
        (
            PhysicalArtifactScope::segment_membership_block(
                store,
                format,
                segment_membership_identity(),
                range(192),
            ),
            PhysicalIntegrityArtifactFamily::SegmentMembership,
            SEGMENT_MEMBERSHIP_INTEGRITY_DECLARATION,
        ),
        (
            PhysicalArtifactScope::inline_page(store, format, page(), range(256)),
            PhysicalIntegrityArtifactFamily::PageFrame,
            PAGE_FRAME_INTEGRITY_DECLARATION,
        ),
        (
            PhysicalArtifactScope::extent_manifest(store, format, extent_placement(), range(320)),
            PhysicalIntegrityArtifactFamily::ExtentManifest,
            EXTENT_MANIFEST_INTEGRITY_DECLARATION,
        ),
        (
            PhysicalArtifactScope::extent_chunk(store, format, extent_chunk(), range(384)),
            PhysicalIntegrityArtifactFamily::ExtentChunk,
            EXTENT_CHUNK_INTEGRITY_DECLARATION,
        ),
        (
            PhysicalArtifactScope::wal_frame(
                store,
                WalSegmentIdentity::new(20, 21).unwrap(),
                range(448),
            ),
            PhysicalIntegrityArtifactFamily::WalFrame,
            WAL_FRAME_INTEGRITY_DECLARATION,
        ),
        (
            PhysicalArtifactScope::checkpoint_stream_header(checkpoint_header, range(512)),
            PhysicalIntegrityArtifactFamily::CheckpointStreamHeader,
            CHECKPOINT_STREAM_HEADER_INTEGRITY_DECLARATION,
        ),
        (
            PhysicalArtifactScope::checkpoint_dirty_basis(checkpoint, range(576)),
            PhysicalIntegrityArtifactFamily::CheckpointDirtyBasis,
            CHECKPOINT_DIRTY_BASIS_INTEGRITY_DECLARATION,
        ),
        (
            PhysicalArtifactScope::checkpoint_binding_compaction(checkpoint, range(640)),
            PhysicalIntegrityArtifactFamily::CheckpointBindingCompaction,
            CHECKPOINT_BINDING_COMPACTION_INTEGRITY_DECLARATION,
        ),
        (
            PhysicalArtifactScope::checkpoint_binding(checkpoint, range(704)),
            PhysicalIntegrityArtifactFamily::CheckpointBinding,
            CHECKPOINT_BINDING_INTEGRITY_DECLARATION,
        ),
        (
            PhysicalArtifactScope::checkpoint_footer(checkpoint, range(768)),
            PhysicalIntegrityArtifactFamily::CheckpointFooter,
            CHECKPOINT_FOOTER_INTEGRITY_DECLARATION,
        ),
        (
            PhysicalArtifactScope::free_space_header(
                store,
                format,
                free_space_header_identity(),
                range(832),
            ),
            PhysicalIntegrityArtifactFamily::FreeSpaceHeader,
            FREE_SPACE_HEADER_INTEGRITY_DECLARATION,
        ),
        (
            PhysicalArtifactScope::free_space_membership_block(
                store,
                format,
                free_space_membership_identity(),
                range(896),
            ),
            PhysicalIntegrityArtifactFamily::FreeSpaceMembershipBlock,
            FREE_SPACE_MEMBERSHIP_BLOCK_INTEGRITY_DECLARATION,
        ),
    ];

    for (index, (scope, family, declaration)) in scopes.into_iter().enumerate() {
        assert_scope(scope, family, declaration);
        assert_eq!(scope.store_identity(), store);
        assert_eq!(scope.byte_range(), range(index as u64 * 64));
        let carries_durable_format = matches!(
            family,
            PhysicalIntegrityArtifactFamily::BootstrapCatalog
                | PhysicalIntegrityArtifactFamily::RootRoutingBlock
                | PhysicalIntegrityArtifactFamily::SegmentMembership
                | PhysicalIntegrityArtifactFamily::PageFrame
                | PhysicalIntegrityArtifactFamily::ExtentManifest
                | PhysicalIntegrityArtifactFamily::ExtentChunk
                | PhysicalIntegrityArtifactFamily::FreeSpaceHeader
                | PhysicalIntegrityArtifactFamily::FreeSpaceMembershipBlock
        );
        assert_eq!(
            scope.durable_frame_record_format(),
            carries_durable_format.then_some(format)
        );
    }
}

#[test]
fn family_scopes_round_trip_their_canonical_identities() {
    let store = store(7);
    let format = format();
    let work = PhysicalWorkObligationIdentity::new(
        NonZeroU64::new(1).unwrap(),
        NonZeroU64::new(2).unwrap(),
        NonZeroU64::new(3).unwrap(),
    );
    let root = root_routing_identity();
    let segment = segment_membership_identity();
    let page = page();
    let extent = extent_placement();
    let chunk = extent_chunk();
    let wal = WalSegmentIdentity::new(20, 21).unwrap();
    let free_space_header = free_space_header_identity();
    let free_space_membership = free_space_membership_identity();

    assert_eq!(
        PhysicalArtifactScope::physical_work_obligation(store, work, range(0))
            .physical_work_obligation_identity(),
        Some(work)
    );
    assert_eq!(
        PhysicalArtifactScope::root_routing_block(store, format, root, range(64))
            .root_routing_block_identity(),
        Some(root)
    );
    assert_eq!(
        PhysicalArtifactScope::segment_membership_block(store, format, segment, range(128))
            .segment_membership_block_identity(),
        Some(segment)
    );
    assert_eq!(
        PhysicalArtifactScope::inline_page(store, format, page, range(192)).page_identity(),
        Some(page)
    );
    assert_eq!(
        PhysicalArtifactScope::extent_manifest(store, format, extent, range(256))
            .extent_manifest_placement(),
        Some(extent)
    );
    assert_eq!(
        PhysicalArtifactScope::extent_chunk(store, format, chunk, range(320))
            .extent_chunk_coordinate(),
        Some(chunk)
    );
    assert_eq!(
        PhysicalArtifactScope::wal_frame(store, wal, range(384)).wal_segment_identity(),
        Some(wal)
    );
    assert_eq!(
        PhysicalArtifactScope::free_space_header(store, format, free_space_header, range(448))
            .free_space_header_identity(),
        Some(free_space_header)
    );
    assert_eq!(
        PhysicalArtifactScope::free_space_membership_block(
            store,
            format,
            free_space_membership,
            range(512),
        )
        .free_space_membership_block_identity(),
        Some(free_space_membership)
    );
}

#[test]
fn only_checkpoint_header_scope_may_stage_checksummed_stream_identity() {
    let store = store(8);
    let staged = CheckpointStreamHeaderScopeIdentity::staged(store);
    let staged_scope = PhysicalArtifactScope::checkpoint_stream_header(staged, range(0));
    assert_eq!(staged.checkpoint_identity(), None);
    assert_eq!(staged_scope.store_identity(), store);

    let identity = PhysicalCheckpointIdentity::new(store, NonZeroU64::new(9).unwrap());
    let known = CheckpointStreamHeaderScopeIdentity::known(identity);
    let known_header = PhysicalArtifactScope::checkpoint_stream_header(known, range(64));
    assert_eq!(known.checkpoint_identity(), Some(identity));
    assert_eq!(
        known_header.checkpoint_stream_header_identity(),
        Some(known)
    );

    let later_record_scopes: [fn(
        PhysicalCheckpointIdentity,
        PhysicalByteRange,
    ) -> PhysicalArtifactScope; 4] = [
        PhysicalArtifactScope::checkpoint_dirty_basis,
        PhysicalArtifactScope::checkpoint_binding_compaction,
        PhysicalArtifactScope::checkpoint_binding,
        PhysicalArtifactScope::checkpoint_footer,
    ];
    for (index, constructor) in later_record_scopes.into_iter().enumerate() {
        let scope = constructor(identity, range(128 + index as u64 * 64));
        assert_eq!(scope.checkpoint_identity(), Some(identity));
    }
}

#[test]
fn family_specific_version_and_localization_vocabulary_remains_distinct() {
    assert_ne!(
        PhysicalIntegrityVersionAxis::PhysicalWorkObligation,
        PhysicalIntegrityVersionAxis::WalFrame
    );
    assert_ne!(
        PhysicalIntegrityVersionAxis::CheckpointRecordSchema,
        PhysicalIntegrityVersionAxis::EnvelopeSchema
    );
    assert_ne!(
        PhysicalDamageCause::AggregateMismatch,
        PhysicalDamageCause::ChecksumMismatch
    );
    assert_ne!(
        PhysicalFormatField::CheckpointAggregate,
        PhysicalFormatField::Checksum
    );
    assert_ne!(
        PhysicalFormatField::ChunkOrdinal,
        PhysicalFormatField::BlockIdentity
    );
}

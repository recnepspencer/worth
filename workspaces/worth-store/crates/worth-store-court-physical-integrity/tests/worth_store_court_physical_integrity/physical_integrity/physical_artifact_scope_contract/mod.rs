#[allow(unused_imports)]
use crate::physical_artifact_scope_contract::*;

#[test]
fn every_family_has_a_canonical_scope_and_version_contract() {
    use worth_store_physical_format::integrity_declarations::families::{
        checkpoint::*, free_space::*, root::*, *,
    };

    let store = store(7);
    let format = format();
    let work = worth_store_physical_format::PhysicalWorkObligationIdentity::new(
        core::num::NonZeroU64::new(1).unwrap(),
        core::num::NonZeroU64::new(2).unwrap(),
        core::num::NonZeroU64::new(3).unwrap(),
    );
    let checkpoint_header =
        worth_store_physical_integrity::CheckpointStreamHeaderScopeIdentity::staged(store);
    let checkpoint = worth_store_physical_format::PhysicalCheckpointIdentity::new(
        store,
        core::num::NonZeroU64::new(24).unwrap(),
    );
    let scopes = [
        (
            worth_store_physical_integrity::PhysicalArtifactScope::physical_work_obligation(store, work, range(0)),
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::PhysicalWorkObligation,
            PHYSICAL_WORK_OBLIGATION_INTEGRITY_DECLARATION,
        ),
        (
            worth_store_physical_integrity::PhysicalArtifactScope::bootstrap_catalog(store, format, range(64)),
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::BootstrapCatalog,
            BOOTSTRAP_CATALOG_INTEGRITY_DECLARATION,
        ),
        (
            worth_store_physical_integrity::PhysicalArtifactScope::root_routing_block(
                store,
                format,
                root_routing_identity(),
                range(128),
            ),
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::RootRoutingBlock,
            ROOT_ROUTING_BLOCK_INTEGRITY_DECLARATION,
        ),
        (
            worth_store_physical_integrity::PhysicalArtifactScope::segment_membership_block(
                store,
                format,
                segment_membership_identity(),
                range(192),
            ),
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::SegmentMembership,
            SEGMENT_MEMBERSHIP_INTEGRITY_DECLARATION,
        ),
        (
            worth_store_physical_integrity::PhysicalArtifactScope::inline_page(store, format, page(), range(256)),
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::PageFrame,
            PAGE_FRAME_INTEGRITY_DECLARATION,
        ),
        (
            worth_store_physical_integrity::PhysicalArtifactScope::extent_manifest(store, format, extent_placement(), range(320)),
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::ExtentManifest,
            EXTENT_MANIFEST_INTEGRITY_DECLARATION,
        ),
        (
            worth_store_physical_integrity::PhysicalArtifactScope::extent_chunk(
                store,
                format,
                extent_chunk(),
                range(384),
                arena_range(),
            ),
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::ExtentChunk,
            EXTENT_CHUNK_INTEGRITY_DECLARATION,
        ),
        (
            worth_store_physical_integrity::PhysicalArtifactScope::wal_frame(
                store,
                worth_store_physical_format::WalSegmentIdentity::new(20, 21).unwrap(),
                range(448),
            ),
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::WalFrame,
            WAL_FRAME_INTEGRITY_DECLARATION,
        ),
        (
            worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_stream_header(checkpoint_header, range(512)),
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointStreamHeader,
            CHECKPOINT_STREAM_HEADER_INTEGRITY_DECLARATION,
        ),
        (
            worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_dirty_basis(checkpoint, range(576)),
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointDirtyBasis,
            CHECKPOINT_DIRTY_BASIS_INTEGRITY_DECLARATION,
        ),
        (
            worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_binding_compaction(checkpoint, range(640)),
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointBindingCompaction,
            CHECKPOINT_BINDING_COMPACTION_INTEGRITY_DECLARATION,
        ),
        (
            worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_binding(checkpoint, range(704)),
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointBinding,
            CHECKPOINT_BINDING_INTEGRITY_DECLARATION,
        ),
        (
            worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_footer(checkpoint, range(768)),
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointFooter,
            CHECKPOINT_FOOTER_INTEGRITY_DECLARATION,
        ),
        (
            worth_store_physical_integrity::PhysicalArtifactScope::free_space_header(
                store,
                format,
                free_space_header_identity(),
                range(832),
            ),
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::FreeSpaceHeader,
            FREE_SPACE_HEADER_INTEGRITY_DECLARATION,
        ),
        (
            worth_store_physical_integrity::PhysicalArtifactScope::free_space_membership_block(
                store,
                format,
                free_space_membership_identity(),
                range(896),
            ),
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::FreeSpaceMembershipBlock,
            FREE_SPACE_MEMBERSHIP_BLOCK_INTEGRITY_DECLARATION,
        ),
    ];

    for (index, (scope, family, declaration)) in scopes.into_iter().enumerate() {
        assert_scope(scope, family, declaration);
        assert_eq!(scope.store_identity(), store);
        assert_eq!(scope.byte_range(), range(index as u64 * 64));
        let carries_durable_format = matches!(
            family,
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::BootstrapCatalog
                | worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::RootRoutingBlock
                | worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::SegmentMembership
                | worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::PageFrame
                | worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::ExtentManifest
                | worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::ExtentChunk
                | worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::FreeSpaceHeader
                | worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::FreeSpaceMembershipBlock
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
    let work = worth_store_physical_format::PhysicalWorkObligationIdentity::new(
        core::num::NonZeroU64::new(1).unwrap(),
        core::num::NonZeroU64::new(2).unwrap(),
        core::num::NonZeroU64::new(3).unwrap(),
    );
    let root = root_routing_identity();
    let segment = segment_membership_identity();
    let page = page();
    let extent = extent_placement();
    let chunk = extent_chunk();
    let wal = worth_store_physical_format::WalSegmentIdentity::new(20, 21).unwrap();
    let free_space_header = free_space_header_identity();
    let free_space_membership = free_space_membership_identity();

    assert_eq!(
        worth_store_physical_integrity::PhysicalArtifactScope::physical_work_obligation(
            store,
            work,
            range(0)
        )
        .physical_work_obligation_identity(),
        Some(work)
    );
    assert_eq!(
        worth_store_physical_integrity::PhysicalArtifactScope::root_routing_block(
            store,
            format,
            root,
            range(64)
        )
        .root_routing_block_identity(),
        Some(root)
    );
    assert_eq!(
        worth_store_physical_integrity::PhysicalArtifactScope::segment_membership_block(
            store,
            format,
            segment,
            range(128)
        )
        .segment_membership_block_identity(),
        Some(segment)
    );
    assert_eq!(
        worth_store_physical_integrity::PhysicalArtifactScope::inline_page(
            store,
            format,
            page,
            range(192)
        )
        .page_identity(),
        Some(page)
    );
    assert_eq!(
        worth_store_physical_integrity::PhysicalArtifactScope::extent_manifest(
            store,
            format,
            extent,
            range(256)
        )
        .extent_manifest_placement(),
        Some(extent)
    );
    assert_eq!(
        worth_store_physical_integrity::PhysicalArtifactScope::extent_chunk(
            store,
            format,
            chunk,
            range(320),
            arena_range()
        )
        .extent_chunk_coordinate(),
        Some(chunk)
    );
    assert_eq!(
        worth_store_physical_integrity::PhysicalArtifactScope::wal_frame(store, wal, range(384))
            .wal_segment_identity(),
        Some(wal)
    );
    assert_eq!(
        worth_store_physical_integrity::PhysicalArtifactScope::free_space_header(
            store,
            format,
            free_space_header,
            range(448)
        )
        .free_space_header_identity(),
        Some(free_space_header)
    );
    assert_eq!(
        worth_store_physical_integrity::PhysicalArtifactScope::free_space_membership_block(
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
    let staged = worth_store_physical_integrity::CheckpointStreamHeaderScopeIdentity::staged(store);
    let staged_scope =
        worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_stream_header(
            staged,
            range(0),
        );
    assert_eq!(staged.checkpoint_identity(), None);
    assert_eq!(staged_scope.store_identity(), store);

    let identity = worth_store_physical_format::PhysicalCheckpointIdentity::new(
        store,
        core::num::NonZeroU64::new(9).unwrap(),
    );
    let known =
        worth_store_physical_integrity::CheckpointStreamHeaderScopeIdentity::known(identity);
    let known_header =
        worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_stream_header(
            known,
            range(64),
        );
    assert_eq!(known.checkpoint_identity(), Some(identity));
    assert_eq!(
        known_header.checkpoint_stream_header_identity(),
        Some(known)
    );

    let later_record_scopes: [fn(
        PhysicalCheckpointIdentity,
        PhysicalByteRange,
    ) -> PhysicalArtifactScope; 4] = [
        worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_dirty_basis,
        worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_binding_compaction,
        worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_binding,
        worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_footer,
    ];
    for (index, constructor) in later_record_scopes.into_iter().enumerate() {
        let scope = constructor(identity, range(128 + index as u64 * 64));
        assert_eq!(scope.checkpoint_identity(), Some(identity));
    }
}

#[test]
fn family_specific_version_and_localization_vocabulary_remains_distinct() {
    assert_ne!(
        worth_store_physical_integrity::PhysicalIntegrityVersionAxis::PhysicalWorkObligation,
        worth_store_physical_integrity::PhysicalIntegrityVersionAxis::WalFrame
    );
    assert_ne!(
        worth_store_physical_integrity::PhysicalIntegrityVersionAxis::CheckpointRecordSchema,
        worth_store_physical_integrity::PhysicalIntegrityVersionAxis::EnvelopeSchema
    );
    assert_ne!(
        worth_store_physical_integrity::PhysicalDamageCause::AggregateMismatch,
        worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch
    );
    assert_ne!(
        worth_store_physical_integrity::PhysicalFormatField::CheckpointAggregate,
        worth_store_physical_integrity::PhysicalFormatField::Checksum
    );
    assert_ne!(
        worth_store_physical_integrity::PhysicalFormatField::ChunkOrdinal,
        worth_store_physical_integrity::PhysicalFormatField::BlockIdentity
    );
}

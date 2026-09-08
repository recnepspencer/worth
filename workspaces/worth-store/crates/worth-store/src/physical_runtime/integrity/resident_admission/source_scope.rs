use worth_store_buffer_pool::PhysicalFrameLease;
use worth_store_physical_format::RecordArtifactFile;
use worth_store_physical_integrity::{PhysicalArtifactScope, UntrustedPhysicalArtifact};

use super::denial::ResidentIntegrityAdmissionDenial;

pub(super) fn require_exact_resident_source<'lease>(
    lease: &'lease PhysicalFrameLease,
    scope: PhysicalArtifactScope,
) -> Result<UntrustedPhysicalArtifact<'lease>, ResidentIntegrityAdmissionDenial> {
    let coordinate = lease.key().coordinate();
    let range = scope.byte_range();
    let exact_range =
        range.offset() == coordinate.offset() && range.length() == u64::from(coordinate.length());
    if scope.store_identity() != lease.key().store()
        || !exact_range
        || !artifact_matches_scope(coordinate.artifact(), scope)
    {
        return Err(ResidentIntegrityAdmissionDenial::SourceScopeMismatch);
    }
    Ok(UntrustedPhysicalArtifact::from_bounded_bytes(lease))
}

pub(in crate::physical_runtime) fn artifact_matches_scope(
    artifact: RecordArtifactFile,
    scope: PhysicalArtifactScope,
) -> bool {
    use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily as Family;

    match scope.artifact_family() {
        Family::BootstrapCatalog => artifact == RecordArtifactFile::BootstrapCatalog,
        Family::CurrentRootSelector => artifact == RecordArtifactFile::CurrentRootSelector,
        Family::PreviousRootSelector => artifact == RecordArtifactFile::PreviousRootSelector,
        Family::RootManifest => scope
            .root_generation()
            .is_some_and(|generation| artifact == RecordArtifactFile::RootManifest { generation }),
        Family::RootRoutingBlock => scope.root_routing_block_identity().is_some_and(|identity| {
            artifact
                == (RecordArtifactFile::RootRoutingBlock {
                    generation: identity.reference().generation(),
                    block: identity.reference().block(),
                })
        }),
        Family::SegmentMembership => {
            scope
                .segment_membership_block_identity()
                .is_some_and(|identity| {
                    artifact
                        == (RecordArtifactFile::SegmentMembershipBlock {
                            generation: identity.reference().generation(),
                            block: identity.reference().block(),
                        })
                })
        }
        Family::PageFrame => scope.page_identity().is_some_and(|page| {
            matches!(artifact, RecordArtifactFile::Segment { segment, .. }
                if segment == page.segment_id().get())
        }),
        Family::ExtentManifest => scope.extent_manifest_placement().is_some_and(|placement| {
            artifact
                == (RecordArtifactFile::ExtentManifest {
                    extent: placement.extent().get(),
                    generation: placement.extent_generation(),
                })
        }),
        Family::ExtentChunk => scope.extent_chunk_coordinate().is_some_and(|coordinate| {
            artifact
                == (RecordArtifactFile::Extent {
                    extent: coordinate.extent_cell().extent_id().get(),
                    generation: coordinate.extent_cell().generation().get(),
                })
        }),
        Family::FreeSpaceHeader => scope.free_space_header_identity().is_some_and(|identity| {
            artifact
                == (RecordArtifactFile::FreeSpaceManifest {
                    generation: identity.generation().get(),
                })
        }),
        Family::FreeSpaceMembershipBlock => scope
            .free_space_membership_block_identity()
            .is_some_and(|identity| {
                artifact
                    == (RecordArtifactFile::FreeSpaceMembershipBlock {
                        generation: identity.reference().generation(),
                        block: identity.reference().block(),
                    })
            }),
        Family::NamespaceIdentity
        | Family::PhysicalWorkObligation
        | Family::WalFrame
        | Family::CheckpointStreamHeader
        | Family::CheckpointDirtyBasis
        | Family::CheckpointBindingCompaction
        | Family::CheckpointBinding
        | Family::CheckpointFooter => false,
    }
}

pub(super) fn require_exact_page_source<'lease>(
    lease: &'lease PhysicalFrameLease,
    scope: PhysicalArtifactScope,
    expected_segment: RecordArtifactFile,
) -> Result<UntrustedPhysicalArtifact<'lease>, ResidentIntegrityAdmissionDenial> {
    if !page_artifact_matches_expected(lease.key().coordinate().artifact(), expected_segment, scope)
    {
        return Err(ResidentIntegrityAdmissionDenial::SourceScopeMismatch);
    }
    let coordinate = lease.key().coordinate();
    let range = scope.byte_range();
    if scope.store_identity() != lease.key().store()
        || range.offset() != coordinate.offset()
        || range.length() != u64::from(coordinate.length())
    {
        return Err(ResidentIntegrityAdmissionDenial::SourceScopeMismatch);
    }
    Ok(UntrustedPhysicalArtifact::from_bounded_bytes(lease))
}

fn page_artifact_matches_expected(
    actual: RecordArtifactFile,
    expected: RecordArtifactFile,
    scope: PhysicalArtifactScope,
) -> bool {
    scope.page_identity().is_some_and(|page| {
        matches!(expected, RecordArtifactFile::Segment { segment, .. }
            if segment == page.segment_id().get())
            && actual == expected
    })
}

#[cfg(test)]
mod tests {
    use worth_store_physical_format::store_namespace::{
        ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
    };
    use worth_store_physical_format::{
        DurableArtifactCrc32c, DurableExtentRecordPlacement, ExtentChunkCoordinate,
        FreeSpaceBlockReference, FreeSpaceHeaderScopeIdentity, FreeSpaceKey,
        FreeSpaceMembershipBlockScopeIdentity, ManifestBlockReference, PageGenerationCell,
        PersistedRecordIdentity, PhysicalExtentId, PhysicalGeneration, PhysicalGenerationAuthority,
        PhysicalPageId, PhysicalRecordFormatDeclaration, PhysicalSegmentId, PhysicalTreeIdentity,
        RecordAllocationClass, RootRoutingBlockScopeIdentity, SegmentManifestBlockReference,
        SegmentMembershipBlockScopeIdentity, SegmentPageKey,
    };
    use worth_store_physical_integrity::PhysicalByteRange;

    use super::*;

    #[test]
    fn every_current_c6_family_has_an_exact_non_root_only_mapping() {
        let store = store();
        let format = format();
        let range = PhysicalByteRange::new(0, 64).unwrap();
        let root = root_reference();
        let segment = segment_reference();
        let free = free_reference();
        let extent = extent_cell();
        let scopes = [
            (
                RecordArtifactFile::BootstrapCatalog,
                PhysicalArtifactScope::bootstrap_catalog(store, format, range),
            ),
            (
                RecordArtifactFile::CurrentRootSelector,
                PhysicalArtifactScope::current_root_selector(store, format, range),
            ),
            (
                RecordArtifactFile::PreviousRootSelector,
                PhysicalArtifactScope::previous_root_selector(store, format, range),
            ),
            (
                RecordArtifactFile::RootManifest { generation: 7 },
                PhysicalArtifactScope::root_manifest(store, format, 7, range).unwrap(),
            ),
            (
                RecordArtifactFile::RootRoutingBlock {
                    generation: root.generation(),
                    block: root.block(),
                },
                PhysicalArtifactScope::root_routing_block(
                    store,
                    format,
                    RootRoutingBlockScopeIdentity::new(tree(), root),
                    range,
                ),
            ),
            (
                RecordArtifactFile::SegmentMembershipBlock {
                    generation: segment.generation(),
                    block: segment.block(),
                },
                PhysicalArtifactScope::segment_membership_block(
                    store,
                    format,
                    SegmentMembershipBlockScopeIdentity::new(tree(), segment),
                    range,
                ),
            ),
            (
                RecordArtifactFile::ExtentManifest {
                    extent: extent.extent_id().get(),
                    generation: extent.generation().get(),
                },
                PhysicalArtifactScope::extent_manifest(
                    store,
                    format,
                    DurableExtentRecordPlacement::new(record(7), extent, 1024).unwrap(),
                    range,
                ),
            ),
            (
                RecordArtifactFile::Extent {
                    extent: extent.extent_id().get(),
                    generation: extent.generation().get(),
                },
                PhysicalArtifactScope::extent_chunk(
                    store,
                    format,
                    ExtentChunkCoordinate::new(record(7), extent, 1024, 0, 1).unwrap(),
                    range,
                ),
            ),
            (
                RecordArtifactFile::FreeSpaceManifest { generation: 22 },
                PhysicalArtifactScope::free_space_header(
                    store,
                    format,
                    FreeSpaceHeaderScopeIdentity::new(
                        PhysicalGeneration::from_raw(22).unwrap(),
                        tree(),
                        Some(free),
                        DurableArtifactCrc32c::new(23),
                    ),
                    range,
                ),
            ),
            (
                RecordArtifactFile::FreeSpaceMembershipBlock {
                    generation: free.generation(),
                    block: free.block(),
                },
                PhysicalArtifactScope::free_space_membership_block(
                    store,
                    format,
                    FreeSpaceMembershipBlockScopeIdentity::new(tree(), free),
                    range,
                ),
            ),
        ];

        for (artifact, scope) in scopes {
            assert!(artifact_matches_scope(artifact, scope));
            let substitute = if artifact == RecordArtifactFile::BootstrapCatalog {
                RecordArtifactFile::CurrentRootSelector
            } else {
                RecordArtifactFile::BootstrapCatalog
            };
            assert!(!artifact_matches_scope(substitute, scope));
        }

        let page_scope = PhysicalArtifactScope::inline_page(store, format, page(), range);
        assert!(artifact_matches_scope(
            RecordArtifactFile::Segment {
                segment: 2,
                generation: 99,
            },
            page_scope,
        ));
        assert!(page_artifact_matches_expected(
            RecordArtifactFile::Segment {
                segment: 2,
                generation: 4,
            },
            RecordArtifactFile::Segment {
                segment: 2,
                generation: 4,
            },
            page_scope,
        ));
        assert!(!page_artifact_matches_expected(
            RecordArtifactFile::Segment {
                segment: 2,
                generation: 99,
            },
            RecordArtifactFile::Segment {
                segment: 2,
                generation: 4,
            },
            page_scope,
        ));
    }

    fn store() -> worth_store_physical_format::store_namespace::StableStoreIdentity {
        StoreNamespaceIdentityRecord::new(
            StoreNamespaceVersion::CURRENT,
            ProposedStoreIdentity::from_nonzero_bytes([81; 16]).unwrap(),
        )
        .published_identity()
    }

    fn format() -> PhysicalRecordFormatDeclaration {
        PhysicalRecordFormatDeclaration::builder().admit().unwrap()
    }

    fn record(ordinal: u64) -> PersistedRecordIdentity {
        PersistedRecordIdentity::new([9; 16], ordinal).unwrap()
    }

    fn tree() -> PhysicalTreeIdentity {
        PhysicalTreeIdentity::new(10).unwrap()
    }

    fn page() -> PageGenerationCell {
        PhysicalGenerationAuthority::for_canonical_physical_format()
            .page_cell(
                PhysicalSegmentId::from_raw(2).unwrap(),
                PhysicalPageId::from_raw(3).unwrap(),
            )
            .with_page_generation(PhysicalGeneration::from_raw(4).unwrap())
    }

    fn extent_cell() -> worth_store_physical_format::RecordExtentGenerationCell {
        PhysicalGenerationAuthority::for_canonical_physical_format()
            .record_extent_cell(PhysicalExtentId::from_raw(5).unwrap())
            .with_extent_generation(PhysicalGeneration::from_raw(6).unwrap())
    }

    fn root_reference() -> ManifestBlockReference {
        ManifestBlockReference::new(11, 12, 0, 13, record(1), record(2)).unwrap()
    }

    fn segment_reference() -> SegmentManifestBlockReference {
        SegmentManifestBlockReference::new(
            14,
            15,
            0,
            16,
            SegmentPageKey::new(
                PhysicalSegmentId::from_raw(1).unwrap(),
                PhysicalPageId::from_raw(1).unwrap(),
            ),
            SegmentPageKey::new(
                PhysicalSegmentId::from_raw(1).unwrap(),
                PhysicalPageId::from_raw(2).unwrap(),
            ),
        )
        .unwrap()
    }

    fn free_reference() -> FreeSpaceBlockReference {
        FreeSpaceBlockReference::new(
            17,
            18,
            0,
            19,
            FreeSpaceKey::new(RecordAllocationClass::InlinePage, 1).unwrap(),
            FreeSpaceKey::new(RecordAllocationClass::Extent, 2).unwrap(),
        )
        .unwrap()
    }
}

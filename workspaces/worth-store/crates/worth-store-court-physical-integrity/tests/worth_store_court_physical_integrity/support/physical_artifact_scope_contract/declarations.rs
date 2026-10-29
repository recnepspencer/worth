pub(crate) use {core::num::NonZeroU64};

pub(crate) use {worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily, worth_store_physical_format::integrity_declarations::PhysicalIntegrityFormatDeclaration};
pub(crate) use {worth_store_physical_format::PhysicalCheckpointIdentity, worth_store_physical_format::PhysicalWorkObligationIdentity, worth_store_physical_format::WalSegmentIdentity};
pub(crate) use {worth_store_physical_integrity::CheckpointStreamHeaderScopeIdentity, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityVersionAxis};


pub(crate) use {crate::canonical_scope_fixtures::arena_range, crate::canonical_scope_fixtures::extent_chunk, crate::canonical_scope_fixtures::extent_placement, crate::canonical_scope_fixtures::format, crate::canonical_scope_fixtures::free_space_header_identity, crate::canonical_scope_fixtures::free_space_membership_identity, crate::canonical_scope_fixtures::page, crate::canonical_scope_fixtures::range, crate::canonical_scope_fixtures::root_routing_identity, crate::canonical_scope_fixtures::segment_membership_identity, crate::canonical_scope_fixtures::store};

pub(crate) fn assert_scope(
    scope: PhysicalArtifactScope,
    expected_family: PhysicalIntegrityArtifactFamily,
    expected_declaration: PhysicalIntegrityFormatDeclaration,
) {
    assert_eq!(scope.artifact_family(), expected_family);
    assert_eq!(scope.declaration(), expected_declaration);
    assert_eq!(scope.format_version(), expected_declaration.version());
}
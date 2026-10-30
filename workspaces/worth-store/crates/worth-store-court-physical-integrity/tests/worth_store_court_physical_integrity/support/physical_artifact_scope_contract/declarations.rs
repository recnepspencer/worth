pub(crate) use core::num::NonZeroU64;

pub(crate) use worth_store_physical_format::integrity_declarations::{
    PhysicalIntegrityArtifactFamily, PhysicalIntegrityFormatDeclaration,
};
pub(crate) use worth_store_physical_format::{
    PhysicalCheckpointIdentity, PhysicalWorkObligationIdentity, WalSegmentIdentity,
};
pub(crate) use worth_store_physical_integrity::{
    CheckpointStreamHeaderScopeIdentity, PhysicalArtifactScope, PhysicalByteRange,
    PhysicalDamageCause, PhysicalFormatField, PhysicalIntegrityVersionAxis,
};

pub(crate) use crate::canonical_scope_fixtures::{
    arena_range, extent_chunk, extent_placement, format, free_space_header_identity,
    free_space_membership_identity, page, range, root_routing_identity,
    segment_membership_identity, store,
};

pub(crate) fn assert_scope(
    scope: PhysicalArtifactScope,
    expected_family: PhysicalIntegrityArtifactFamily,
    expected_declaration: PhysicalIntegrityFormatDeclaration,
) {
    assert_eq!(scope.artifact_family(), expected_family);
    assert_eq!(scope.declaration(), expected_declaration);
    assert_eq!(scope.format_version(), expected_declaration.version());
}

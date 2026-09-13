use worth_store_physical_format::integrity_declarations::{
    families::{
        checkpoint::{
            CHECKPOINT_BINDING_COMPACTION_INTEGRITY_DECLARATION,
            CHECKPOINT_BINDING_INTEGRITY_DECLARATION, CHECKPOINT_DIRTY_BASIS_INTEGRITY_DECLARATION,
            CHECKPOINT_FOOTER_INTEGRITY_DECLARATION,
            CHECKPOINT_STREAM_HEADER_INTEGRITY_DECLARATION,
        },
        free_space::{
            FREE_SPACE_HEADER_INTEGRITY_DECLARATION,
            FREE_SPACE_MEMBERSHIP_BLOCK_INTEGRITY_DECLARATION,
        },
        root::{
            BOOTSTRAP_CATALOG_INTEGRITY_DECLARATION, CURRENT_SELECTOR_INTEGRITY_DECLARATION,
            PREVIOUS_SELECTOR_INTEGRITY_DECLARATION, ROOT_MANIFEST_INTEGRITY_DECLARATION,
            ROOT_ROUTING_BLOCK_INTEGRITY_DECLARATION,
        },
        EXTENT_CHUNK_INTEGRITY_DECLARATION, EXTENT_MANIFEST_INTEGRITY_DECLARATION,
        PAGE_FRAME_INTEGRITY_DECLARATION, PHYSICAL_WORK_OBLIGATION_INTEGRITY_DECLARATION,
        SEGMENT_MEMBERSHIP_INTEGRITY_DECLARATION, WAL_FRAME_INTEGRITY_DECLARATION,
    },
    PhysicalIntegrityArtifactFamily, PhysicalIntegrityFormatDeclaration,
    PhysicalIntegrityFormatVersion,
};
use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_format::PhysicalRecordFormatDeclaration;

use super::identity::PhysicalArtifactScopeIdentity;
use crate::localization::PhysicalByteRange;

/// Exact descriptive scope against which one bounded artifact is validated.
///
/// Construction proves no byte validity or source provenance. Family-specific
/// constructors accept canonical physical-format identities and deliberately
/// omit fields that exist only inside unvalidated checksummed framing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalArtifactScope {
    pub(super) store: StableStoreIdentity,
    pub(super) identity: PhysicalArtifactScopeIdentity,
    pub(super) range: PhysicalByteRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalArtifactScopeDenial {
    ZeroRootGeneration,
}

impl PhysicalArtifactScope {
    pub(super) const fn new(
        store: StableStoreIdentity,
        identity: PhysicalArtifactScopeIdentity,
        range: PhysicalByteRange,
    ) -> Self {
        Self {
            store,
            identity,
            range,
        }
    }

    pub const fn store_identity(self) -> StableStoreIdentity {
        self.store
    }

    pub const fn artifact_family(self) -> PhysicalIntegrityArtifactFamily {
        use PhysicalArtifactScopeIdentity as Identity;
        match self.identity {
            Identity::PhysicalWorkObligation(_) => {
                PhysicalIntegrityArtifactFamily::PhysicalWorkObligation
            }
            Identity::BootstrapCatalog(_) => PhysicalIntegrityArtifactFamily::BootstrapCatalog,
            Identity::CurrentRootSelector(_) => {
                PhysicalIntegrityArtifactFamily::CurrentRootSelector
            }
            Identity::PreviousRootSelector(_) => {
                PhysicalIntegrityArtifactFamily::PreviousRootSelector
            }
            Identity::RootManifest { .. } => PhysicalIntegrityArtifactFamily::RootManifest,
            Identity::RootRoutingBlock { .. } => PhysicalIntegrityArtifactFamily::RootRoutingBlock,
            Identity::SegmentMembershipBlock { .. } => {
                PhysicalIntegrityArtifactFamily::SegmentMembership
            }
            Identity::InlinePage { .. } => PhysicalIntegrityArtifactFamily::PageFrame,
            Identity::ExtentManifest { .. } => PhysicalIntegrityArtifactFamily::ExtentManifest,
            Identity::ExtentChunk { .. } => PhysicalIntegrityArtifactFamily::ExtentChunk,
            Identity::WalFrame(_) => PhysicalIntegrityArtifactFamily::WalFrame,
            Identity::CheckpointStreamHeader(_) => {
                PhysicalIntegrityArtifactFamily::CheckpointStreamHeader
            }
            Identity::CheckpointDirtyBasis(_) => {
                PhysicalIntegrityArtifactFamily::CheckpointDirtyBasis
            }
            Identity::CheckpointBindingCompaction(_) => {
                PhysicalIntegrityArtifactFamily::CheckpointBindingCompaction
            }
            Identity::CheckpointBinding(_) => PhysicalIntegrityArtifactFamily::CheckpointBinding,
            Identity::CheckpointFooter(_) => PhysicalIntegrityArtifactFamily::CheckpointFooter,
            Identity::FreeSpaceHeader { .. } => PhysicalIntegrityArtifactFamily::FreeSpaceHeader,
            Identity::FreeSpaceMembershipBlock { .. } => {
                PhysicalIntegrityArtifactFamily::FreeSpaceMembershipBlock
            }
        }
    }

    pub const fn declaration(self) -> PhysicalIntegrityFormatDeclaration {
        use PhysicalArtifactScopeIdentity as Identity;
        match self.identity {
            Identity::PhysicalWorkObligation(_) => PHYSICAL_WORK_OBLIGATION_INTEGRITY_DECLARATION,
            Identity::BootstrapCatalog(_) => BOOTSTRAP_CATALOG_INTEGRITY_DECLARATION,
            Identity::CurrentRootSelector(_) => CURRENT_SELECTOR_INTEGRITY_DECLARATION,
            Identity::PreviousRootSelector(_) => PREVIOUS_SELECTOR_INTEGRITY_DECLARATION,
            Identity::RootManifest { .. } => ROOT_MANIFEST_INTEGRITY_DECLARATION,
            Identity::RootRoutingBlock { .. } => ROOT_ROUTING_BLOCK_INTEGRITY_DECLARATION,
            Identity::SegmentMembershipBlock { .. } => SEGMENT_MEMBERSHIP_INTEGRITY_DECLARATION,
            Identity::InlinePage { .. } => PAGE_FRAME_INTEGRITY_DECLARATION,
            Identity::ExtentManifest { .. } => EXTENT_MANIFEST_INTEGRITY_DECLARATION,
            Identity::ExtentChunk { .. } => EXTENT_CHUNK_INTEGRITY_DECLARATION,
            Identity::WalFrame(_) => WAL_FRAME_INTEGRITY_DECLARATION,
            Identity::CheckpointStreamHeader(_) => CHECKPOINT_STREAM_HEADER_INTEGRITY_DECLARATION,
            Identity::CheckpointDirtyBasis(_) => CHECKPOINT_DIRTY_BASIS_INTEGRITY_DECLARATION,
            Identity::CheckpointBindingCompaction(_) => {
                CHECKPOINT_BINDING_COMPACTION_INTEGRITY_DECLARATION
            }
            Identity::CheckpointBinding(_) => CHECKPOINT_BINDING_INTEGRITY_DECLARATION,
            Identity::CheckpointFooter(_) => CHECKPOINT_FOOTER_INTEGRITY_DECLARATION,
            Identity::FreeSpaceHeader { .. } => FREE_SPACE_HEADER_INTEGRITY_DECLARATION,
            Identity::FreeSpaceMembershipBlock { .. } => {
                FREE_SPACE_MEMBERSHIP_BLOCK_INTEGRITY_DECLARATION
            }
        }
    }

    pub const fn format_version(self) -> PhysicalIntegrityFormatVersion {
        self.declaration().version()
    }

    pub const fn byte_range(self) -> PhysicalByteRange {
        self.range
    }

    pub const fn durable_frame_record_format(self) -> Option<PhysicalRecordFormatDeclaration> {
        use PhysicalArtifactScopeIdentity as Identity;
        match self.identity {
            Identity::CurrentRootSelector(format)
            | Identity::PreviousRootSelector(format)
            | Identity::BootstrapCatalog(format) => Some(format),
            Identity::RootManifest {
                record_format: format,
                ..
            }
            | Identity::RootRoutingBlock {
                record_format: format,
                ..
            }
            | Identity::SegmentMembershipBlock {
                record_format: format,
                ..
            }
            | Identity::InlinePage {
                record_format: format,
                ..
            }
            | Identity::ExtentManifest {
                record_format: format,
                ..
            }
            | Identity::ExtentChunk {
                record_format: format,
                ..
            }
            | Identity::FreeSpaceHeader {
                record_format: format,
                ..
            }
            | Identity::FreeSpaceMembershipBlock {
                record_format: format,
                ..
            } => Some(format),
            Identity::PhysicalWorkObligation(_)
            | Identity::WalFrame(_)
            | Identity::CheckpointStreamHeader(_)
            | Identity::CheckpointDirtyBasis(_)
            | Identity::CheckpointBindingCompaction(_)
            | Identity::CheckpointBinding(_)
            | Identity::CheckpointFooter(_) => None,
        }
    }

    /// Phase 3 durable-frame accessor retained for selector/root callers.
    pub(crate) const fn record_format(self) -> PhysicalRecordFormatDeclaration {
        match self.durable_frame_record_format() {
            Some(format) => format,
            None => panic!("artifact family has no durable-frame record format"),
        }
    }
}

pub(super) fn encode_durable_artifact_scope_prefix(
    scope: PhysicalArtifactScope,
    family_tag: u8,
    target: &mut [u8; 43],
) {
    target[..16].copy_from_slice(&scope.store.bytes());
    target[16] = family_tag;
    target[17..25].copy_from_slice(&scope.range.offset().to_le_bytes());
    target[25..33].copy_from_slice(&scope.range.length().to_le_bytes());
    target[33..43].copy_from_slice(&scope.record_format().canonical_identity_bytes());
}

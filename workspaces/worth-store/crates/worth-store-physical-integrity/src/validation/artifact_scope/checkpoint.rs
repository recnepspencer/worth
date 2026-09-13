use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_format::{durable_artifact_checksum, PhysicalCheckpointIdentity};

use super::{PhysicalArtifactScope, PhysicalArtifactScopeIdentity};
use crate::localization::PhysicalByteRange;

/// Expected identity available before a checkpoint stream header is inspected.
///
/// The sequence may be omitted only for the header because it exists inside
/// that record's checksummed framing. All later record scopes require the
/// canonical checkpoint identity exposed by an admitted header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckpointStreamHeaderScopeIdentity {
    StagedFromChecksummedStream(StableStoreIdentity),
    Known(PhysicalCheckpointIdentity),
}

impl CheckpointStreamHeaderScopeIdentity {
    pub const fn staged(store: StableStoreIdentity) -> Self {
        Self::StagedFromChecksummedStream(store)
    }

    pub const fn known(identity: PhysicalCheckpointIdentity) -> Self {
        Self::Known(identity)
    }

    pub const fn store_identity(self) -> StableStoreIdentity {
        match self {
            Self::StagedFromChecksummedStream(store) => store,
            Self::Known(identity) => identity.store_identity(),
        }
    }

    pub const fn checkpoint_identity(self) -> Option<PhysicalCheckpointIdentity> {
        match self {
            Self::StagedFromChecksummedStream(_) => None,
            Self::Known(identity) => Some(identity),
        }
    }
}

impl PhysicalArtifactScope {
    pub const fn checkpoint_stream_header(
        identity: CheckpointStreamHeaderScopeIdentity,
        range: PhysicalByteRange,
    ) -> Self {
        Self::new(
            identity.store_identity(),
            PhysicalArtifactScopeIdentity::CheckpointStreamHeader(identity),
            range,
        )
    }

    pub const fn checkpoint_dirty_basis(
        identity: PhysicalCheckpointIdentity,
        range: PhysicalByteRange,
    ) -> Self {
        Self::checkpoint_record(
            identity,
            PhysicalArtifactScopeIdentity::CheckpointDirtyBasis(identity),
            range,
        )
    }

    pub const fn checkpoint_binding_compaction(
        identity: PhysicalCheckpointIdentity,
        range: PhysicalByteRange,
    ) -> Self {
        Self::checkpoint_record(
            identity,
            PhysicalArtifactScopeIdentity::CheckpointBindingCompaction(identity),
            range,
        )
    }

    pub const fn checkpoint_binding(
        identity: PhysicalCheckpointIdentity,
        range: PhysicalByteRange,
    ) -> Self {
        Self::checkpoint_record(
            identity,
            PhysicalArtifactScopeIdentity::CheckpointBinding(identity),
            range,
        )
    }

    pub const fn checkpoint_footer(
        identity: PhysicalCheckpointIdentity,
        range: PhysicalByteRange,
    ) -> Self {
        Self::checkpoint_record(
            identity,
            PhysicalArtifactScopeIdentity::CheckpointFooter(identity),
            range,
        )
    }

    const fn checkpoint_record(
        identity: PhysicalCheckpointIdentity,
        scope_identity: PhysicalArtifactScopeIdentity,
        range: PhysicalByteRange,
    ) -> Self {
        Self::new(identity.store_identity(), scope_identity, range)
    }

    pub const fn checkpoint_stream_header_identity(
        self,
    ) -> Option<CheckpointStreamHeaderScopeIdentity> {
        match self.identity {
            PhysicalArtifactScopeIdentity::CheckpointStreamHeader(identity) => Some(identity),
            _ => None,
        }
    }

    pub const fn checkpoint_identity(self) -> Option<PhysicalCheckpointIdentity> {
        match self.identity {
            PhysicalArtifactScopeIdentity::CheckpointStreamHeader(identity) => {
                identity.checkpoint_identity()
            }
            PhysicalArtifactScopeIdentity::CheckpointDirtyBasis(identity)
            | PhysicalArtifactScopeIdentity::CheckpointBindingCompaction(identity)
            | PhysicalArtifactScopeIdentity::CheckpointBinding(identity)
            | PhysicalArtifactScopeIdentity::CheckpointFooter(identity) => Some(identity),
            _ => None,
        }
    }

    pub(crate) const fn is_checkpoint_dirty_basis(self) -> bool {
        matches!(
            self.identity,
            PhysicalArtifactScopeIdentity::CheckpointDirtyBasis(_)
        )
    }

    pub(crate) const fn is_checkpoint_binding_compaction(self) -> bool {
        matches!(
            self.identity,
            PhysicalArtifactScopeIdentity::CheckpointBindingCompaction(_)
        )
    }

    pub(crate) const fn is_checkpoint_binding(self) -> bool {
        matches!(
            self.identity,
            PhysicalArtifactScopeIdentity::CheckpointBinding(_)
        )
    }

    pub(crate) const fn is_checkpoint_footer(self) -> bool {
        matches!(
            self.identity,
            PhysicalArtifactScopeIdentity::CheckpointFooter(_)
        )
    }

    pub(crate) fn checkpoint_exact_scope_digest(self) -> Option<u32> {
        let mut preimage = [0_u8; 42];
        preimage[..16].copy_from_slice(&self.store_identity().bytes());
        let (family, known_identity) = match self.identity {
            PhysicalArtifactScopeIdentity::CheckpointStreamHeader(identity) => {
                (1, identity.checkpoint_identity())
            }
            PhysicalArtifactScopeIdentity::CheckpointDirtyBasis(identity) => (2, Some(identity)),
            PhysicalArtifactScopeIdentity::CheckpointBindingCompaction(identity) => {
                (3, Some(identity))
            }
            PhysicalArtifactScopeIdentity::CheckpointBinding(identity) => (4, Some(identity)),
            PhysicalArtifactScopeIdentity::CheckpointFooter(identity) => (5, Some(identity)),
            _ => return None,
        };
        preimage[16] = family;
        if let Some(identity) = known_identity {
            preimage[17] = 1;
            preimage[18..26].copy_from_slice(&identity.sequence().get().to_le_bytes());
        }
        preimage[26..34].copy_from_slice(&self.byte_range().offset().to_le_bytes());
        preimage[34..42].copy_from_slice(&self.byte_range().length().to_le_bytes());
        Some(durable_artifact_checksum(&preimage))
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use worth_store_physical_format::store_namespace::{
        ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
    };

    use super::{CheckpointStreamHeaderScopeIdentity, PhysicalArtifactScope};
    use crate::PhysicalByteRange;
    use worth_store_physical_format::PhysicalCheckpointIdentity;

    fn identity(sequence: u64) -> PhysicalCheckpointIdentity {
        identity_for(7, sequence)
    }

    fn identity_for(store_byte: u8, sequence: u64) -> PhysicalCheckpointIdentity {
        let store = StoreNamespaceIdentityRecord::new(
            StoreNamespaceVersion::CURRENT,
            ProposedStoreIdentity::from_nonzero_bytes([store_byte; 16]).unwrap(),
        )
        .published_identity();
        PhysicalCheckpointIdentity::new(store, NonZeroU64::new(sequence).unwrap())
    }

    #[test]
    fn checkpoint_kind_predicates_are_exact() {
        let identity = identity(3);
        let range = PhysicalByteRange::new(10, 20).unwrap();
        let scopes = [
            PhysicalArtifactScope::checkpoint_dirty_basis(identity, range),
            PhysicalArtifactScope::checkpoint_binding_compaction(identity, range),
            PhysicalArtifactScope::checkpoint_binding(identity, range),
            PhysicalArtifactScope::checkpoint_footer(identity, range),
        ];
        for (index, scope) in scopes.into_iter().enumerate() {
            assert_eq!(scope.is_checkpoint_dirty_basis(), index == 0);
            assert_eq!(scope.is_checkpoint_binding_compaction(), index == 1);
            assert_eq!(scope.is_checkpoint_binding(), index == 2);
            assert_eq!(scope.is_checkpoint_footer(), index == 3);
        }
    }

    #[test]
    fn exact_digest_binds_kind_identity_posture_and_range() {
        let checkpoint = identity(3);
        let other_sequence = identity(4);
        let other_store = identity_for(8, 3);
        let range = PhysicalByteRange::new(10, 20).unwrap();
        let shifted = PhysicalByteRange::new(11, 20).unwrap();
        let staged = PhysicalArtifactScope::checkpoint_stream_header(
            CheckpointStreamHeaderScopeIdentity::staged(checkpoint.store_identity()),
            range,
        );
        let known = PhysicalArtifactScope::checkpoint_stream_header(
            CheckpointStreamHeaderScopeIdentity::known(checkpoint),
            range,
        );
        let dirty = PhysicalArtifactScope::checkpoint_dirty_basis(checkpoint, range);
        assert_ne!(
            staged.checkpoint_exact_scope_digest(),
            known.checkpoint_exact_scope_digest()
        );
        let alternatives = [
            PhysicalArtifactScope::checkpoint_binding_compaction(checkpoint, range),
            PhysicalArtifactScope::checkpoint_dirty_basis(other_sequence, range),
            PhysicalArtifactScope::checkpoint_dirty_basis(other_store, range),
            PhysicalArtifactScope::checkpoint_dirty_basis(checkpoint, shifted),
        ];
        for alternative in alternatives {
            assert_ne!(
                dirty.checkpoint_exact_scope_digest(),
                alternative.checkpoint_exact_scope_digest()
            );
        }
    }
}

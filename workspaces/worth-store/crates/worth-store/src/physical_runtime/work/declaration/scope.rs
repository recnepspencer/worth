use sha2::{Digest, Sha256};
use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};

use super::{
    PhysicalCheckpointWorkScope, PhysicalRootPublicationWorkAction,
    PhysicalRootPublicationWorkScope, PhysicalWalAppendScope, PhysicalWalBarrierScope,
    PhysicalWalReclamationScope, PhysicalWorkDeclarationDenial,
};

const MAX_PHYSICAL_SCOPE_MEMBERS: usize = 256;

/// Exact, non-overlapping artifact ranges belonging to one work identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalWorkScope {
    members: PhysicalWorkScopeMembers,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PhysicalWorkScopeMembers {
    Inspection(worth_store_physical_format::PhysicalArtifactReadRange),
    Artifact(RecordArtifactFile),
    One(RecordFrameCoordinate),
    Batch(Box<[RecordFrameCoordinate]>),
    Checkpoint(PhysicalCheckpointWorkScope),
    WalAppend(PhysicalWalAppendScope),
    WalBarrier(PhysicalWalBarrierScope),
    WalReclamation(PhysicalWalReclamationScope),
    RootPublication(PhysicalRootPublicationWorkScope),
}

impl PhysicalWorkScope {
    pub(in crate::physical_runtime) const fn inspection(
        range: worth_store_physical_format::PhysicalArtifactReadRange,
    ) -> Self {
        Self {
            members: PhysicalWorkScopeMembers::Inspection(range),
        }
    }

    pub const fn inspection_target(
        &self,
    ) -> Option<worth_store_physical_format::PhysicalArtifactReadRange> {
        match self.members {
            PhysicalWorkScopeMembers::Inspection(range) => Some(range),
            _ => None,
        }
    }

    pub fn artifact(artifact: RecordArtifactFile) -> Self {
        Self {
            members: PhysicalWorkScopeMembers::Artifact(artifact),
        }
    }

    pub fn one(coordinate: RecordFrameCoordinate) -> Self {
        Self {
            members: PhysicalWorkScopeMembers::One(coordinate),
        }
    }

    pub fn batch(
        coordinates: impl IntoIterator<Item = RecordFrameCoordinate>,
    ) -> Result<Self, PhysicalWorkDeclarationDenial> {
        let mut exact = Vec::new();
        for coordinate in coordinates {
            if exact.len() == MAX_PHYSICAL_SCOPE_MEMBERS {
                return Err(PhysicalWorkDeclarationDenial::ScopeCapacityExceeded);
            }
            exact.push(coordinate);
        }
        if exact.is_empty() {
            return Err(PhysicalWorkDeclarationDenial::EmptyScope);
        }
        if exact.len() == 1 {
            return Err(PhysicalWorkDeclarationDenial::BatchRequiresMultipleMembers);
        }
        exact.sort_unstable();
        require_disjoint_members(&exact)?;
        Ok(Self {
            members: PhysicalWorkScopeMembers::Batch(exact.into_boxed_slice()),
        })
    }

    pub(in crate::physical_runtime) const fn wal_append(scope: PhysicalWalAppendScope) -> Self {
        Self {
            members: PhysicalWorkScopeMembers::WalAppend(scope),
        }
    }

    pub(in crate::physical_runtime) const fn wal_barrier(scope: PhysicalWalBarrierScope) -> Self {
        Self {
            members: PhysicalWorkScopeMembers::WalBarrier(scope),
        }
    }

    pub(in crate::physical_runtime) const fn checkpoint(
        scope: PhysicalCheckpointWorkScope,
    ) -> Self {
        Self {
            members: PhysicalWorkScopeMembers::Checkpoint(scope),
        }
    }

    pub(in crate::physical_runtime) const fn wal_reclamation(
        scope: PhysicalWalReclamationScope,
    ) -> Self {
        Self {
            members: PhysicalWorkScopeMembers::WalReclamation(scope),
        }
    }

    pub(in crate::physical_runtime) const fn root_publication(
        scope: PhysicalRootPublicationWorkScope,
    ) -> Self {
        Self {
            members: PhysicalWorkScopeMembers::RootPublication(scope),
        }
    }

    pub fn coordinates(&self) -> &[RecordFrameCoordinate] {
        match &self.members {
            PhysicalWorkScopeMembers::Artifact(_) | PhysicalWorkScopeMembers::Inspection(_) => &[],
            PhysicalWorkScopeMembers::Checkpoint(_)
            | PhysicalWorkScopeMembers::WalAppend(_)
            | PhysicalWorkScopeMembers::WalBarrier(_) => &[],
            PhysicalWorkScopeMembers::WalReclamation(_)
            | PhysicalWorkScopeMembers::RootPublication(_) => &[],
            PhysicalWorkScopeMembers::One(coordinate) => std::slice::from_ref(coordinate),
            PhysicalWorkScopeMembers::Batch(coordinates) => coordinates,
        }
    }

    pub const fn artifact_target(&self) -> Option<RecordArtifactFile> {
        match &self.members {
            PhysicalWorkScopeMembers::Artifact(artifact) => Some(*artifact),
            PhysicalWorkScopeMembers::One(_)
            | PhysicalWorkScopeMembers::Batch(_)
            | PhysicalWorkScopeMembers::Inspection(_) => None,
            PhysicalWorkScopeMembers::Checkpoint(_)
            | PhysicalWorkScopeMembers::WalAppend(_)
            | PhysicalWorkScopeMembers::WalBarrier(_) => None,
            PhysicalWorkScopeMembers::WalReclamation(_)
            | PhysicalWorkScopeMembers::RootPublication(_) => None,
        }
    }

    pub const fn wal_append_target(&self) -> Option<PhysicalWalAppendScope> {
        match &self.members {
            PhysicalWorkScopeMembers::WalAppend(scope) => Some(*scope),
            _ => None,
        }
    }

    pub const fn wal_barrier_target(&self) -> Option<PhysicalWalBarrierScope> {
        match &self.members {
            PhysicalWorkScopeMembers::WalBarrier(scope) => Some(*scope),
            _ => None,
        }
    }

    pub(in crate::physical_runtime) const fn checkpoint_target(
        &self,
    ) -> Option<PhysicalCheckpointWorkScope> {
        match &self.members {
            PhysicalWorkScopeMembers::Checkpoint(scope) => Some(*scope),
            _ => None,
        }
    }

    pub(in crate::physical_runtime) const fn wal_reclamation_target(
        &self,
    ) -> Option<PhysicalWalReclamationScope> {
        match &self.members {
            PhysicalWorkScopeMembers::WalReclamation(scope) => Some(*scope),
            _ => None,
        }
    }

    pub(in crate::physical_runtime) const fn root_publication_target(
        &self,
    ) -> Option<PhysicalRootPublicationWorkScope> {
        match &self.members {
            PhysicalWorkScopeMembers::RootPublication(scope) => Some(*scope),
            _ => None,
        }
    }

    pub const fn member_count(&self) -> usize {
        match &self.members {
            PhysicalWorkScopeMembers::Inspection(_)
            | PhysicalWorkScopeMembers::Artifact(_)
            | PhysicalWorkScopeMembers::One(_)
            | PhysicalWorkScopeMembers::Checkpoint(_)
            | PhysicalWorkScopeMembers::WalAppend(_)
            | PhysicalWorkScopeMembers::WalBarrier(_) => 1,
            PhysicalWorkScopeMembers::WalReclamation(_)
            | PhysicalWorkScopeMembers::RootPublication(_) => 1,
            PhysicalWorkScopeMembers::Batch(coordinates) => coordinates.len(),
        }
    }

    pub(in crate::physical_runtime::work) fn stable_digest(&self) -> [u8; 32] {
        let mut digest = Sha256::new();
        digest.update(b"worth-store.physical-work-scope.v1");
        digest.update((self.member_count() as u64).to_le_bytes());
        if let Some(range) = self.inspection_target() {
            super::inspection_digest::include(&mut digest, range);
            return digest.finalize().into();
        }
        if let PhysicalWorkScopeMembers::Artifact(artifact) = &self.members {
            digest.update(b"artifact");
            let name = artifact.file_name();
            digest.update((name.len() as u64).to_le_bytes());
            digest.update(name.as_bytes());
            return digest.finalize().into();
        }
        if let PhysicalWorkScopeMembers::WalAppend(scope) = &self.members {
            digest.update(b"wal-append");
            digest.update(scope.segment().to_le_bytes());
            digest.update(scope.generation().to_le_bytes());
            digest.update(scope.offset().to_le_bytes());
            digest.update(scope.byte_count().to_le_bytes());
            return digest.finalize().into();
        }
        if let PhysicalWorkScopeMembers::WalBarrier(scope) = &self.members {
            digest.update(b"wal-barrier");
            digest.update(scope.group());
            digest.update(scope.membership());
            digest.update(scope.group_member_count().to_le_bytes());
            digest.update(scope.segment().to_le_bytes());
            digest.update(scope.generation().to_le_bytes());
            digest.update(scope.lsn_start().to_le_bytes());
            digest.update(scope.lsn_end_exclusive().to_le_bytes());
            digest.update(scope.append_offset().to_le_bytes());
            digest.update(scope.append_byte_count().to_le_bytes());
            return digest.finalize().into();
        }
        if let PhysicalWorkScopeMembers::Checkpoint(scope) = &self.members {
            digest.update(b"checkpoint");
            digest.update(scope.checkpoint().store_identity().bytes());
            digest.update(scope.checkpoint().sequence().get().to_le_bytes());
            match scope.action() {
                super::PhysicalCheckpointWorkAction::CreateCandidate { byte_count } => {
                    digest.update([1]);
                    digest.update(byte_count.to_le_bytes());
                }
                super::PhysicalCheckpointWorkAction::AppendCandidate { offset, byte_count } => {
                    digest.update([2]);
                    digest.update(offset.to_le_bytes());
                    digest.update(byte_count.to_le_bytes());
                }
                super::PhysicalCheckpointWorkAction::SynchronizeCandidate => digest.update([3]),
                super::PhysicalCheckpointWorkAction::RemoveCandidate => digest.update([4]),
                super::PhysicalCheckpointWorkAction::PublishCandidate => digest.update([5]),
                super::PhysicalCheckpointWorkAction::SynchronizeNamespace => digest.update([6]),
            }
            return digest.finalize().into();
        }
        if let PhysicalWorkScopeMembers::WalReclamation(scope) = &self.members {
            digest.update(b"wal-reclamation");
            digest.update(scope.checkpoint().store_identity().bytes());
            digest.update(scope.checkpoint().sequence().get().to_le_bytes());
            digest.update(scope.compaction_generation().to_le_bytes());
            digest.update(scope.compaction_digest());
            digest.update(scope.retained_boundary().get().to_le_bytes());
            digest.update(scope.segment().segment().get().to_le_bytes());
            digest.update(scope.segment().generation().get().to_le_bytes());
            digest.update(scope.lsn_range().start().get().to_le_bytes());
            digest.update(scope.lsn_range().end_exclusive().get().to_le_bytes());
            digest.update(scope.byte_count().to_le_bytes());
            return digest.finalize().into();
        }
        if let PhysicalWorkScopeMembers::RootPublication(scope) = &self.members {
            digest.update(b"root-publication");
            digest.update(scope.publication().stable_digest());
            match scope.action() {
                PhysicalRootPublicationWorkAction::SynchronizeCandidateArtifact { artifact } => {
                    digest.update([1]);
                    let name = artifact.file_name();
                    digest.update((name.len() as u64).to_le_bytes());
                    digest.update(name.as_bytes());
                }
                PhysicalRootPublicationWorkAction::ReplaceBootstrapCatalog => digest.update([2]),
                PhysicalRootPublicationWorkAction::SynchronizeParentNamespace => digest.update([3]),
            }
            return digest.finalize().into();
        }
        digest.update(b"ranges");
        for coordinate in self.coordinates() {
            let artifact = coordinate.artifact().file_name();
            digest.update((artifact.len() as u64).to_le_bytes());
            digest.update(artifact.as_bytes());
            digest.update(coordinate.offset().to_le_bytes());
            digest.update(coordinate.length().to_le_bytes());
        }
        digest.finalize().into()
    }
}

fn require_disjoint_members(
    members: &[RecordFrameCoordinate],
) -> Result<(), PhysicalWorkDeclarationDenial> {
    for pair in members.windows(2) {
        let left = pair[0];
        let right = pair[1];
        if left == right {
            return Err(PhysicalWorkDeclarationDenial::DuplicateScopeMember);
        }
        if left.artifact() == right.artifact()
            && left.offset().saturating_add(u64::from(left.length())) > right.offset()
        {
            return Err(PhysicalWorkDeclarationDenial::OverlappingScopeMembers);
        }
    }
    Ok(())
}

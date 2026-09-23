use worth_store_physical_format::{PhysicalArtifactReadTarget, RecordArtifactFile};

use super::{PhysicalEffectAccess, PhysicalEffectFootprint, PhysicalEffectKey};
use crate::physical_runtime::work::{
    PhysicalCheckpointWorkAction, PhysicalRootPublicationWorkAction, PhysicalWorkIntent,
    PhysicalWorkOperationFamily, PhysicalWorkScope,
};

pub(in crate::physical_runtime) fn lower_effect_footprint(
    intent: &PhysicalWorkIntent,
) -> PhysicalEffectFootprint {
    PhysicalEffectFootprint::new(
        intent.identity().store(),
        lower_scope(intent.scope(), access_for(intent.operation())),
    )
}

pub(super) fn lower_scope(
    scope: &PhysicalWorkScope,
    access: PhysicalEffectAccess,
) -> Vec<PhysicalEffectKey> {
    if let Some(inspection) = scope.inspection_target() {
        return vec![lower_inspection(inspection)];
    }
    if let Some(artifact) = scope.artifact_removal_target() {
        return vec![PhysicalEffectKey::DeleteArtifact { artifact }];
    }
    if let Some(artifact) = scope.artifact_target() {
        return vec![lower_named_artifact(artifact, access)];
    }
    if let Some(wal) = scope.wal_append_target() {
        return vec![PhysicalEffectKey::Wal {
            segment: wal.segment(),
            generation: wal.generation(),
            start: wal.offset(),
            end: wal.offset().saturating_add(wal.byte_count()),
            access: PhysicalEffectAccess::Write,
        }];
    }
    if let Some(barrier) = scope.wal_barrier_target() {
        return vec![PhysicalEffectKey::Wal {
            segment: barrier.segment(),
            generation: barrier.generation(),
            start: barrier.append_offset(),
            end: barrier
                .append_offset()
                .saturating_add(barrier.append_byte_count()),
            access: PhysicalEffectAccess::Write,
        }];
    }
    if let Some(checkpoint) = scope.checkpoint_target() {
        return vec![lower_checkpoint(checkpoint.action())];
    }
    if let Some(reclamation) = scope.wal_reclamation_target() {
        let segment = reclamation.segment();
        return vec![PhysicalEffectKey::DeleteWal {
            segment: segment.segment().get(),
            generation: segment.generation().get(),
        }];
    }
    if let Some(root) = scope.root_publication_target() {
        return lower_root(root.action());
    }
    let mut keys = Vec::with_capacity(scope.coordinates().len());
    for coordinate in scope.coordinates() {
        keys.push(range_or_allocator(
            coordinate.artifact(),
            coordinate.offset(),
            coordinate
                .offset()
                .saturating_add(u64::from(coordinate.length())),
            access,
        ));
    }
    assert!(
        !keys.is_empty(),
        "every physical work scope lowers to a coordination key"
    );
    keys
}

fn lower_inspection(
    range: worth_store_physical_format::PhysicalArtifactReadRange,
) -> PhysicalEffectKey {
    let end = range.offset().saturating_add(u64::from(range.length()));
    match range.target() {
        PhysicalArtifactReadTarget::Record(artifact) => {
            range_or_allocator(artifact, range.offset(), end, PhysicalEffectAccess::Read)
        }
        PhysicalArtifactReadTarget::Wal(segment) => PhysicalEffectKey::Wal {
            segment: segment.segment().get(),
            generation: segment.generation().get(),
            start: range.offset(),
            end,
            access: PhysicalEffectAccess::Read,
        },
        PhysicalArtifactReadTarget::Checkpoint(_) => PhysicalEffectKey::Checkpoint {
            start: range.offset(),
            end,
            whole: false,
            access: PhysicalEffectAccess::Read,
        },
        PhysicalArtifactReadTarget::PhysicalWork(identity) => PhysicalEffectKey::Obligation {
            identity,
            access: PhysicalEffectAccess::Read,
        },
    }
}

fn lower_named_artifact(
    artifact: RecordArtifactFile,
    access: PhysicalEffectAccess,
) -> PhysicalEffectKey {
    match allocator_key(artifact) {
        Some(key) => key,
        None => PhysicalEffectKey::WholeArtifact { artifact, access },
    }
}

fn lower_checkpoint(action: PhysicalCheckpointWorkAction) -> PhysicalEffectKey {
    match action {
        PhysicalCheckpointWorkAction::SynchronizeNamespace => PhysicalEffectKey::Namespace,
        PhysicalCheckpointWorkAction::CreateCandidate { byte_count } => {
            PhysicalEffectKey::Checkpoint {
                start: 0,
                end: byte_count,
                whole: false,
                access: PhysicalEffectAccess::Write,
            }
        }
        PhysicalCheckpointWorkAction::AppendCandidate { offset, byte_count } => {
            PhysicalEffectKey::Checkpoint {
                start: offset,
                end: offset.saturating_add(byte_count),
                whole: false,
                access: PhysicalEffectAccess::Write,
            }
        }
        PhysicalCheckpointWorkAction::SynchronizeCandidate
        | PhysicalCheckpointWorkAction::RemoveCandidate
        | PhysicalCheckpointWorkAction::PublishCandidate => PhysicalEffectKey::Checkpoint {
            start: 0,
            end: u64::MAX,
            whole: true,
            access: PhysicalEffectAccess::Write,
        },
    }
}

fn lower_root(action: PhysicalRootPublicationWorkAction) -> Vec<PhysicalEffectKey> {
    match action {
        PhysicalRootPublicationWorkAction::SynchronizeParentNamespace => {
            vec![
                PhysicalEffectKey::Namespace,
                PhysicalEffectKey::RootPublication,
            ]
        }
        PhysicalRootPublicationWorkAction::ReplaceBootstrapCatalog => vec![
            PhysicalEffectKey::WholeArtifact {
                artifact: RecordArtifactFile::BootstrapCatalog,
                access: PhysicalEffectAccess::Write,
            },
            PhysicalEffectKey::RootPublication,
        ],
        PhysicalRootPublicationWorkAction::SynchronizeCandidateArtifact { artifact } => vec![
            PhysicalEffectKey::WholeArtifact {
                artifact,
                access: PhysicalEffectAccess::Write,
            },
            PhysicalEffectKey::RootPublication,
        ],
    }
}

fn range_or_allocator(
    artifact: RecordArtifactFile,
    start: u64,
    end: u64,
    access: PhysicalEffectAccess,
) -> PhysicalEffectKey {
    match allocator_key(artifact) {
        Some(key) => key,
        None => PhysicalEffectKey::Range {
            artifact,
            start,
            end,
            access,
        },
    }
}

fn allocator_key(artifact: RecordArtifactFile) -> Option<PhysicalEffectKey> {
    match artifact {
        RecordArtifactFile::FreeSpaceManifest { generation } => {
            Some(PhysicalEffectKey::Allocator {
                generation,
                block: None,
            })
        }
        RecordArtifactFile::FreeSpaceMembershipBlock { generation, block } => {
            Some(PhysicalEffectKey::Allocator {
                generation,
                block: Some(block),
            })
        }
        _ => None,
    }
}

const fn access_for(operation: PhysicalWorkOperationFamily) -> PhysicalEffectAccess {
    match operation {
        PhysicalWorkOperationFamily::ArtifactMetadataRead
        | PhysicalWorkOperationFamily::ArtifactRangeRead => PhysicalEffectAccess::Read,
        PhysicalWorkOperationFamily::ArtifactRangeWrite
        | PhysicalWorkOperationFamily::ArtifactPublication
        | PhysicalWorkOperationFamily::CheckpointCapture
        | PhysicalWorkOperationFamily::WalAppend
        | PhysicalWorkOperationFamily::DurabilityBarrier
        | PhysicalWorkOperationFamily::WalReclamation
        | PhysicalWorkOperationFamily::RootPublication => PhysicalEffectAccess::Write,
    }
}

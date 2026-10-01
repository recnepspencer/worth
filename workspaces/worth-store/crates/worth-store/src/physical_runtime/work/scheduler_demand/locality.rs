use sha2::{Digest, Sha256};
use worth_store_io_scheduler::{QueueLocalityIdentity, QueueLocalityRange};
use worth_store_physical_format::RecordArtifactFile;

use crate::physical_runtime::{PhysicalRootPublicationWorkAction, PhysicalWorkScope};

pub(super) fn physical_locality(
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    scope: &PhysicalWorkScope,
) -> QueueLocalityIdentity {
    let mut digest = Sha256::new();
    digest.update(b"worth-store.physical-queue-locality.v1");
    digest.update(store.bytes());
    digest.update((scope.member_count() as u64).to_le_bytes());
    if let Some(artifact) = scope.artifact_target() {
        update_artifact(&mut digest, artifact);
        let identity = digest.finalize().into();
        let mut artifact_identity = Sha256::new();
        artifact_identity.update(b"worth-store.physical-queue-artifact.v1");
        artifact_identity.update(store.bytes());
        update_artifact(&mut artifact_identity, artifact);
        let range = QueueLocalityRange::new(artifact_identity.finalize().into(), 0, u64::MAX)
            .expect("artifact-wide locality is nonempty");
        return QueueLocalityIdentity::from_ranges(identity, [range])
            .expect("one artifact-wide locality range is valid");
    }
    if let Some(wal) = scope.wal_append_target() {
        digest.update(b"wal");
        digest.update(wal.segment().to_le_bytes());
        digest.update(wal.generation().to_le_bytes());
        digest.update(wal.offset().to_le_bytes());
        digest.update(wal.byte_count().to_le_bytes());
        let identity = digest.finalize().into();
        let mut artifact = Sha256::new();
        artifact.update(b"worth-store.physical-queue-wal-artifact.v1");
        artifact.update(store.bytes());
        artifact.update(wal.segment().to_le_bytes());
        artifact.update(wal.generation().to_le_bytes());
        let range = QueueLocalityRange::new(
            artifact.finalize().into(),
            wal.offset(),
            wal.offset().saturating_add(wal.byte_count()),
        )
        .expect("an admitted WAL append range is nonempty");
        return QueueLocalityIdentity::from_ranges(identity, [range])
            .expect("one WAL append locality range is valid");
    }
    if let Some(barrier) = scope.wal_barrier_target() {
        digest.update(b"wal-barrier");
        digest.update(barrier.group());
        digest.update(barrier.membership());
        digest.update(barrier.group_member_count().to_le_bytes());
        digest.update(barrier.segment().to_le_bytes());
        digest.update(barrier.generation().to_le_bytes());
        digest.update(barrier.lsn_start().to_le_bytes());
        digest.update(barrier.lsn_end_exclusive().to_le_bytes());
        digest.update(barrier.append_offset().to_le_bytes());
        digest.update(barrier.append_byte_count().to_le_bytes());
        let identity = digest.finalize().into();
        let mut artifact = Sha256::new();
        artifact.update(b"worth-store.physical-queue-wal-artifact.v1");
        artifact.update(store.bytes());
        artifact.update(barrier.segment().to_le_bytes());
        artifact.update(barrier.generation().to_le_bytes());
        let end_exclusive = barrier
            .append_offset()
            .checked_add(barrier.append_byte_count())
            .expect("an admitted WAL barrier interval cannot overflow");
        let range = QueueLocalityRange::new(
            artifact.finalize().into(),
            barrier.append_offset(),
            end_exclusive,
        )
        .expect("an admitted WAL barrier interval is nonempty");
        return QueueLocalityIdentity::from_ranges(identity, [range])
            .expect("one WAL barrier locality range is valid");
    }
    if let Some(root) = scope.root_publication_target() {
        digest.update(b"root-publication");
        digest.update(root.publication().stable_digest());
        let artifact = match root.action() {
            PhysicalRootPublicationWorkAction::SynchronizeCandidateArtifact { artifact } => {
                digest.update(b"candidate-sync");
                artifact
            }
            PhysicalRootPublicationWorkAction::ReplaceBootstrapCatalog => {
                digest.update(b"catalog-replacement");
                root.publication().catalog_candidate()
            }
            PhysicalRootPublicationWorkAction::SynchronizeParentNamespace => {
                digest.update(b"namespace-sync");
                RecordArtifactFile::BootstrapCatalog
            }
        };
        update_artifact(&mut digest, artifact);
        let identity = digest.finalize().into();
        let mut artifact_identity = Sha256::new();
        artifact_identity.update(b"worth-store.physical-queue-artifact.v1");
        artifact_identity.update(store.bytes());
        update_artifact(&mut artifact_identity, artifact);
        let range = QueueLocalityRange::new(artifact_identity.finalize().into(), 0, u64::MAX)
            .expect("root-publication artifact locality is nonempty");
        return QueueLocalityIdentity::from_ranges(identity, [range])
            .expect("one root-publication artifact locality range is valid");
    }
    for coordinate in scope.coordinates() {
        update_artifact(&mut digest, coordinate.artifact());
        digest.update(coordinate.offset().to_le_bytes());
        digest.update(coordinate.length().to_le_bytes());
    }
    let identity = digest.finalize().into();
    let ranges = scope.coordinates().iter().map(|coordinate| {
        let mut artifact = Sha256::new();
        artifact.update(b"worth-store.physical-queue-artifact.v1");
        artifact.update(store.bytes());
        update_artifact(&mut artifact, coordinate.artifact());
        QueueLocalityRange::new(
            artifact.finalize().into(),
            coordinate.offset(),
            coordinate
                .offset()
                .saturating_add(u64::from(coordinate.length())),
        )
        .expect("admitted physical scope ranges are nonempty")
    });
    QueueLocalityIdentity::from_ranges(identity, ranges)
        .expect("admitted physical scope ranges are sorted and disjoint")
}

fn update_artifact(digest: &mut Sha256, artifact: RecordArtifactFile) {
    let (tag, first, second): (u8, u64, u64) = match artifact {
        RecordArtifactFile::BootstrapCatalog => (1, 0, 0),
        RecordArtifactFile::CurrentRootSelector => (12, 0, 0),
        RecordArtifactFile::PreviousRootSelector => (13, 0, 0),
        RecordArtifactFile::RootSelectorCandidate { role, publication } => match role {
            worth_store_physical_format::RootSelectorRole::Current => (14, publication, 0),
            worth_store_physical_format::RootSelectorRole::Previous => (15, publication, 0),
        },
        RecordArtifactFile::CatalogCandidate { publication } => (2, publication, 0),
        RecordArtifactFile::RootManifest { generation } => (3, generation, 0),
        RecordArtifactFile::RootRoutingBlock { generation, block } => (4, generation, block),
        RecordArtifactFile::ReleaseCustodyHeadBlock { generation, block } => {
            (17, generation, block)
        }
        RecordArtifactFile::Segment {
            segment,
            generation,
        } => (5, segment, generation),
        RecordArtifactFile::SegmentManifest {
            segment,
            generation,
        } => (6, segment, generation),
        RecordArtifactFile::SegmentMembershipBlock { generation, block } => (7, generation, block),
        RecordArtifactFile::ExtentArena { arena } => (16, arena, 0),
        RecordArtifactFile::FreeSpaceManifest { generation } => (10, generation, 0),
        RecordArtifactFile::FreeSpaceMembershipBlock { generation, block } => {
            (11, generation, block)
        }
    };
    digest.update([tag]);
    digest.update(first.to_le_bytes());
    digest.update(second.to_le_bytes());
}

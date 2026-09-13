mod snapshot_codec;

use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use fs4::FileExt;
use worth_store_authority::StoreCurrentAuthorityIdentity;
use worth_store_physical_backend::{
    reach_storage_boundary, ProductionStorageBoundaryControl, ProductionStorageBoundarySeam,
    StorageBoundaryRegion,
};

use crate::{
    DurabilityReplayIdentity, ReplicationLineageIdentity, ReplicationPeerId,
    ReplicationPeerProgress, ReplicationSourceEpoch,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplicationPeerCapacity(usize);

#[derive(Debug, Clone)]
pub(crate) struct StoredReplicationPeerProgress {
    pub peer_id: ReplicationPeerId,
    pub source_epoch: ReplicationSourceEpoch,
    pub lineage: ReplicationLineageIdentity,
    pub current_authority: StoreCurrentAuthorityIdentity,
    pub security_scope_fingerprint: [u8; 32],
    pub replay: DurabilityReplayIdentity,
}

#[derive(Debug)]
pub(crate) struct ReplicationProgressStore {
    directory: PathBuf,
    capacity: ReplicationPeerCapacity,
    current_authority: StoreCurrentAuthorityIdentity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReplicationProgressStoreError {
    AuthorityMismatch,
    CapacityExceeded,
    Io,
}

pub(super) struct Snapshot {
    pub(super) generation: u64,
    pub(super) authority: StoreCurrentAuthorityIdentity,
    pub(super) records: BTreeMap<ReplicationPeerId, StoredReplicationPeerProgress>,
}

impl ReplicationPeerCapacity {
    pub fn new(peers: usize) -> Option<Self> {
        (peers > 0).then_some(Self(peers))
    }

    pub const fn get(self) -> usize {
        self.0
    }
}

impl ReplicationProgressStore {
    pub(crate) fn open(
        directory: &Path,
        capacity: ReplicationPeerCapacity,
        current_authority: StoreCurrentAuthorityIdentity,
    ) -> Result<
        (
            Self,
            BTreeMap<ReplicationPeerId, StoredReplicationPeerProgress>,
        ),
        ReplicationProgressStoreError,
    > {
        std::fs::create_dir_all(directory).map_err(|_| ReplicationProgressStoreError::Io)?;
        let store = Self {
            directory: directory.to_path_buf(),
            capacity,
            current_authority,
        };
        let lock = store.open_lock()?;
        lock.lock_exclusive()
            .map_err(|_| ReplicationProgressStoreError::Io)?;
        let snapshot = store.latest_snapshot()?;
        if snapshot
            .as_ref()
            .is_some_and(|snapshot| snapshot.authority != current_authority)
        {
            return Err(ReplicationProgressStoreError::AuthorityMismatch);
        }
        let records = snapshot.map_or_else(BTreeMap::new, |snapshot| snapshot.records);
        if records.len() > capacity.get() {
            return Err(ReplicationProgressStoreError::CapacityExceeded);
        }
        Ok((store, records))
    }

    pub(crate) fn persist_controlled(
        &self,
        progress: &ReplicationPeerProgress,
        control: &impl ProductionStorageBoundaryControl,
    ) -> Result<StoredReplicationPeerProgress, ReplicationProgressStoreError> {
        let lock = self.open_lock()?;
        lock.lock_exclusive()
            .map_err(|_| ReplicationProgressStoreError::Io)?;
        let prior = self.latest_snapshot()?;
        if prior
            .as_ref()
            .is_some_and(|snapshot| snapshot.authority != self.current_authority)
        {
            return Err(ReplicationProgressStoreError::AuthorityMismatch);
        }
        let generation = prior.as_ref().map_or(Ok(1), |snapshot| {
            snapshot
                .generation
                .checked_add(1)
                .ok_or(ReplicationProgressStoreError::Io)
        })?;
        let mut records = prior.map_or_else(BTreeMap::new, |snapshot| snapshot.records);
        if !records.contains_key(progress.peer_id()) && records.len() >= self.capacity.get() {
            return Err(ReplicationProgressStoreError::CapacityExceeded);
        }
        let stored = stored_progress(progress);
        records.insert(stored.peer_id.clone(), stored.clone());
        let encoded = snapshot_codec::encode(generation, self.current_authority, &records)?;
        let target = self.snapshot_path((generation & 1) as usize);
        let mut snapshot = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(target)
            .map_err(|_| ReplicationProgressStoreError::Io)?;
        snapshot
            .write_all(&encoded)
            .map_err(|_| ReplicationProgressStoreError::Io)?;
        reach_storage_boundary(
            control,
            ProductionStorageBoundarySeam::ReplicationProgressSnapshotWrite,
            &mut snapshot,
            StorageBoundaryRegion::new(0, encoded.len() as u64),
        )
        .map_err(|_| ReplicationProgressStoreError::Io)?;
        snapshot
            .sync_all()
            .map_err(|_| ReplicationProgressStoreError::Io)?;
        sync_directory(&self.directory)?;
        reach_storage_boundary(
            control,
            ProductionStorageBoundarySeam::ReplicationProgressSnapshotDurable,
            &mut snapshot,
            StorageBoundaryRegion::new(0, encoded.len() as u64),
        )
        .map_err(|_| ReplicationProgressStoreError::Io)?;
        Ok(stored)
    }

    fn latest_snapshot(&self) -> Result<Option<Snapshot>, ReplicationProgressStoreError> {
        use snapshot_codec::SnapshotSlot;

        let left = snapshot_codec::read(&self.snapshot_path(0))?;
        let right = snapshot_codec::read(&self.snapshot_path(1))?;
        Ok(match (left, right) {
            (SnapshotSlot::Corrupt, _) | (_, SnapshotSlot::Corrupt) => {
                return Err(ReplicationProgressStoreError::Io)
            }
            (SnapshotSlot::Valid(left), SnapshotSlot::Valid(right)) => {
                Some(if left.generation >= right.generation {
                    left
                } else {
                    right
                })
            }
            (SnapshotSlot::Valid(snapshot), SnapshotSlot::Missing | SnapshotSlot::Torn)
            | (SnapshotSlot::Missing | SnapshotSlot::Torn, SnapshotSlot::Valid(snapshot)) => {
                Some(snapshot)
            }
            (SnapshotSlot::Missing, SnapshotSlot::Missing) => None,
            (SnapshotSlot::Torn, SnapshotSlot::Missing)
            | (SnapshotSlot::Missing, SnapshotSlot::Torn) => None,
            (SnapshotSlot::Torn, SnapshotSlot::Torn) => {
                return Err(ReplicationProgressStoreError::Io)
            }
        })
    }

    fn open_lock(&self) -> Result<File, ReplicationProgressStoreError> {
        OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.directory.join("replication-progress.lock"))
            .map_err(|_| ReplicationProgressStoreError::Io)
    }

    fn snapshot_path(&self, slot: usize) -> PathBuf {
        self.directory
            .join(format!("replication-progress-{slot}.snapshot"))
    }
}

fn stored_progress(progress: &ReplicationPeerProgress) -> StoredReplicationPeerProgress {
    StoredReplicationPeerProgress {
        peer_id: progress.peer_id().clone(),
        source_epoch: progress.source_epoch(),
        lineage: progress.lineage().clone(),
        current_authority: progress.current_authority,
        security_scope_fingerprint: progress.security_scope.stable_fingerprint(),
        replay: progress.replay_identity().clone(),
    }
}

#[cfg(windows)]
fn sync_directory(path: &Path) -> Result<(), ReplicationProgressStoreError> {
    use std::os::windows::fs::OpenOptionsExt;
    OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(0x0200_0000)
        .open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| ReplicationProgressStoreError::Io)
}

#[cfg(not(windows))]
fn sync_directory(path: &Path) -> Result<(), ReplicationProgressStoreError> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| ReplicationProgressStoreError::Io)
}

//! The stream readers decide their bounds like every whole reader, with the
//! file's real length: past a declared ceiling is damage, then past the grant
//! is the grant's, then past the observation's bytes is the observation's.
//! The checkpoint stream's length no fact declares; a selected WAL member's
//! length its selection declares.

use std::ffi::{OsStr, OsString};

use super::*;
use crate::filesystem_media::{
    ArtifactTreeDirectoryEntry, ArtifactTreeListingAllocator, ArtifactTreeListingStorageChange,
    ArtifactTreePathAllocationBoundary, ArtifactTreePathAllocator, ArtifactTreeReadAllocator,
    ArtifactTreeStorageAllocator,
};
use crate::recovery_media::{
    ObservedWalArtifact, RecoverySelectedWalReadOutcome, RecoveryWalListingAllocationMode,
    RecoveryWalReadSelection, RecoveryWalReadStorage, RecoveryWalSelectionMismatch,
};
use FilesystemObservationBound::ObservationBytes;

/// Plain storage, listed by the caller.
struct Plain;
impl ArtifactTreeStorageAllocator for Plain {
    type Denial = DeniedAllocation;
}
impl ArtifactTreePathAllocator for Plain {
    type PathBacking = ();
    fn admit_path_backing(
        &mut self,
        _: ArtifactTreePathAllocationBoundary,
        _: u64,
    ) -> Result<(), DeniedAllocation> {
        Ok(())
    }
}
impl ArtifactTreeListingAllocator for Plain {
    fn listing_storage_change(
        &mut self,
        _: ArtifactTreeListingStorageChange,
    ) -> Result<(), DeniedAllocation> {
        Ok(())
    }
    fn allocate_listing_roster(
        &mut self,
        count: usize,
    ) -> Result<Vec<ArtifactTreeDirectoryEntry>, DeniedAllocation> {
        Ok(Vec::with_capacity(count))
    }
}
impl ArtifactTreeReadAllocator for Plain {
    fn allocate_read_buffer(&mut self, length: usize) -> Result<Vec<u8>, DeniedAllocation> {
        Ok(vec![0; length])
    }
}
impl RecoveryWalReadStorage for Plain {
    fn listing_allocation_mode(&self) -> RecoveryWalListingAllocationMode {
        RecoveryWalListingAllocationMode::CallerManaged
    }
    fn allocate_wal_roster(
        &mut self,
        count: usize,
    ) -> Result<Vec<ObservedWalArtifact>, DeniedAllocation> {
        Ok(Vec::with_capacity(count))
    }
    fn allocate_context(&mut self, count: usize) -> Result<OsString, DeniedAllocation> {
        Ok(OsString::with_capacity(count))
    }
}

/// Every listed file, each declared `declared` bytes long.
struct Declared(u64);
impl RecoveryWalReadSelection for Declared {
    fn matches_listing(&self, _: &mut [ArtifactTreeDirectoryEntry]) -> bool {
        true
    }
    fn expected_file_length(&self, _: &OsStr) -> Option<u64> {
        Some(self.0)
    }
}

/// Where a selected inventory of `files`, each declared `declared` bytes,
/// stopped under `granted` and `reader` observation bytes; `None` for one
/// that read, and its mismatch for one that met its selection's drift.
fn selected(
    files: &[usize],
    declared: u64,
    granted: u64,
    reader: u64,
) -> Result<Option<Stop>, RecoveryWalSelectionMismatch> {
    let (_parent, _observer, mut discovery) = discovery(
        |root| {
            let wal = root.join("families/wal");
            std::fs::create_dir_all(&wal).unwrap();
            for (index, length) in files.iter().enumerate() {
                std::fs::write(wal.join(format!("{index}.wal")), vec![3; *length]).unwrap();
            }
        },
        8,
        reader,
    );
    let read = discovery.read_selected_wal_artifacts_with_storage(
        segments(4),
        grant(granted),
        &Declared(declared),
        &mut Plain,
    );
    match read {
        TransitionOutcome::Success(RecoverySelectedWalReadOutcome::Mismatch(mismatch)) => {
            Err(mismatch)
        }
        other => Ok(stopped(&other)),
    }
}

#[test]
fn a_selected_wal_member_exactly_at_its_declared_length_reads() {
    assert_eq!(selected(&[8], 8, 8, 64), Ok(None));
    assert_eq!(selected(&[8, 8], 8, 16, 64), Ok(None));
}

/// One byte past its declared length, however much the grant holds, is
/// damage with the real length: never the grant's limit.
#[test]
fn a_selected_wal_member_past_its_declared_length_is_damage_with_its_real_length() {
    let past = |length| Ok(Some(Stop::PastCeiling { length, ceiling: 8 }));
    assert_eq!(selected(&[9], 8, 64, 64), past(9));
    assert_eq!(selected(&[13], 8, 64, 64), past(13));
    // A grant or an observation too small for the member is no reason to
    // call its damage a limit.
    assert_eq!(selected(&[9], 8, 4, 64), past(9));
    assert_eq!(selected(&[9], 8, 64, 4), past(9));
    // A shorter member is drift from its selection.
    assert_eq!(
        selected(&[7], 8, 64, 64),
        Err(RecoveryWalSelectionMismatch::FileLength {
            expected: 8,
            observed: 7
        })
    );
}

/// A member exactly at its declared length, past what the grant left, is the
/// grant's, with what the members before it took plus its own length.
#[test]
fn a_selected_wal_member_at_its_declared_length_past_the_grant_is_the_grants() {
    assert_eq!(
        selected(&[8], 8, 7, 64),
        Ok(Some(Stop::PastGrant {
            granted: 7,
            length: 8
        }))
    );
    assert_eq!(
        selected(&[8, 8], 8, 15, 64),
        Ok(Some(Stop::PastGrant {
            granted: 15,
            length: 16
        }))
    );
    assert_eq!(
        selected(&[8, 8], 8, 64, 15),
        Ok(Some(Stop::Observation(ObservationBytes, 16, 15)))
    );
}

/// Where a checkpoint stream of `length` bytes stopped under `granted` and
/// `reader` observation bytes, read by each stream reader.
fn checkpoint_stops(length: usize, granted: u64, reader: u64) -> Vec<Option<Stop>> {
    let observe = || {
        discovery(
            |root| {
                std::fs::write(root.join("families/checkpoint.current"), vec![1; length]).unwrap()
            },
            4,
            reader,
        )
    };
    let (_addressed_parent, _observer, mut addressed) = observe();
    let (_allocated_parent, _observer, mut allocated) = observe();
    vec![
        stopped(&addressed.read(checkpoint(), grant(granted))),
        stopped(
            &allocated.read_with_allocator(checkpoint(), grant(granted), |length| {
                Ok::<_, DeniedAllocation>(vec![0; length])
            }),
        ),
    ]
}

/// The checkpoint stream declares no ceiling: any length the grant and the
/// observation admit reads.
#[test]
fn the_checkpoint_stream_is_bounded_by_its_grant_and_the_observation_alone() {
    assert_eq!(checkpoint_stops(4096, 4096, 4096), vec![None; 2]);
    let past_grant = Some(Stop::PastGrant {
        granted: 4096,
        length: 4097,
    });
    assert_eq!(checkpoint_stops(4097, 4096, 8192), vec![past_grant; 2]);
    let observation = Some(Stop::Observation(ObservationBytes, 4097, 4096));
    assert_eq!(checkpoint_stops(4097, 8192, 4096), vec![observation; 2]);
    // An uncharged checkpoint read stops only at the observation.
    let (_parent, _observer, mut discovery) = discovery(
        |root| std::fs::write(root.join("families/checkpoint.current"), [1; 9]).unwrap(),
        4,
        8,
    );
    assert_eq!(
        stopped(&discovery.read(checkpoint(), uncharged())),
        Some(Stop::Observation(ObservationBytes, 9, 8))
    );
}

/// A checkpoint stream a claim declares is bounded by that declaration first:
/// at it reads, one byte past is damage with the real length however much the
/// grant and the observation hold, and the grant still refuses within it.
#[test]
fn a_declared_checkpoint_stream_past_its_declaration_is_damage_with_its_real_length() {
    let read = |length: usize, declared: u64, granted: u64| {
        let (_parent, _observer, mut discovery) = discovery(
            |root| {
                std::fs::write(root.join("families/checkpoint.current"), vec![1; length]).unwrap()
            },
            4,
            64,
        );
        let ceiling = ArtifactCeiling::declared(
            crate::recovery_media::StreamArtifact::CurrentCheckpoint,
            declared,
        );
        stopped(&discovery.read(ceiling, grant(granted)))
    };
    assert_eq!(read(8, 8, 8), None);
    let past = Some(Stop::PastCeiling {
        length: 9,
        ceiling: 8,
    });
    assert_eq!(read(9, 8, 64), past);
    assert_eq!(read(9, 8, 4), past);
    assert_eq!(
        read(8, 8, 7),
        Some(Stop::PastGrant {
            granted: 7,
            length: 8
        })
    );
}

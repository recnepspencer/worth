use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use worth_foundational::CanonicalDigestId;

use sha2::{Digest, Sha256};
use worth_store_physical_backend::ControlMediaFault;

use super::{OperationalOperationId, OperationalWorkflowKind};

const RECENT_WORKFLOW_LIMIT: usize = 1_024;
const INDEX_LEVELS: usize = u64::BITS as usize;
const ENTRY_BYTES: usize = 33;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ArchivedWorkflowKind {
    BackupTerminal,
    NonBackup(OperationalWorkflowKind),
}

pub(super) struct ArchivedWorkflowIndex {
    recent: HashMap<CanonicalDigestId, ArchivedWorkflowKind>,
    levels: [Option<ArchivedWorkflowSegment>; INDEX_LEVELS],
}

struct ArchivedWorkflowSegment {
    file: File,
    entries: u64,
}

impl ArchivedWorkflowIndex {
    pub(super) fn empty() -> Self {
        Self {
            recent: HashMap::new(),
            levels: std::array::from_fn(|_| None),
        }
    }

    pub(super) fn lookup(
        &mut self,
        operation: &OperationalOperationId,
    ) -> Result<Option<ArchivedWorkflowKind>, ControlMediaFault> {
        let key = operation_key(operation);
        if let Some(kind) = self.recent.get(&key) {
            return Ok(Some(*kind));
        }
        for segment in self.levels.iter_mut().flatten() {
            if let Some(kind) = segment.lookup(key)? {
                return Ok(Some(kind));
            }
        }
        Ok(None)
    }

    pub(super) fn insert(
        &mut self,
        operation: &OperationalOperationId,
        kind: ArchivedWorkflowKind,
    ) -> Result<(), ControlMediaFault> {
        if self.lookup(operation)?.is_some() {
            return Err(ControlMediaFault::DuplicateTransitionConflict);
        }
        if self.recent.len() == RECENT_WORKFLOW_LIMIT {
            self.flush_recent()?;
        }
        self.recent
            .try_reserve(1)
            .map_err(|_| ControlMediaFault::AllocationFailed)?;
        self.recent.insert(operation_key(operation), kind);
        Ok(())
    }

    fn flush_recent(&mut self) -> Result<(), ControlMediaFault> {
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(self.recent.len())
            .map_err(|_| ControlMediaFault::AllocationFailed)?;
        entries.extend(self.recent.drain());
        entries.sort_unstable_by_key(|(key, _)| *key);
        self.absorb(ArchivedWorkflowSegment::from_sorted(&entries)?)
    }

    fn absorb(&mut self, mut segment: ArchivedWorkflowSegment) -> Result<(), ControlMediaFault> {
        for level in &mut self.levels {
            match level.take() {
                None => {
                    *level = Some(segment);
                    return Ok(());
                }
                Some(existing) => segment = ArchivedWorkflowSegment::merge(existing, segment)?,
            }
        }
        Err(ControlMediaFault::GenerationExhausted)
    }
}

impl ArchivedWorkflowSegment {
    fn from_sorted(
        entries: &[(CanonicalDigestId, ArchivedWorkflowKind)],
    ) -> Result<Self, ControlMediaFault> {
        let mut file = tempfile::tempfile()?;
        for (key, kind) in entries {
            write_entry(&mut file, *key, *kind)?;
        }
        file.seek(SeekFrom::Start(0))?;
        Ok(Self {
            file,
            entries: u64::try_from(entries.len())
                .map_err(|_| ControlMediaFault::GenerationExhausted)?,
        })
    }

    fn lookup(
        &mut self,
        key: CanonicalDigestId,
    ) -> Result<Option<ArchivedWorkflowKind>, ControlMediaFault> {
        let mut low = 0u64;
        let mut high = self.entries;
        while low < high {
            let middle = low + (high - low) / 2;
            let (observed, kind) = self.read_at(middle)?;
            match observed.cmp(&key) {
                Ordering::Less => low = middle + 1,
                Ordering::Greater => high = middle,
                Ordering::Equal => return Ok(Some(kind)),
            }
        }
        Ok(None)
    }

    fn merge(mut left: Self, mut right: Self) -> Result<Self, ControlMediaFault> {
        left.file.seek(SeekFrom::Start(0))?;
        right.file.seek(SeekFrom::Start(0))?;
        let mut output = tempfile::tempfile()?;
        let mut left_entry = read_next(&mut left.file)?;
        let mut right_entry = read_next(&mut right.file)?;
        while left_entry.is_some() || right_entry.is_some() {
            let next = match (left_entry, right_entry) {
                (Some(left_value), Some(right_value)) => match left_value.0.cmp(&right_value.0) {
                    Ordering::Less => {
                        left_entry = read_next(&mut left.file)?;
                        left_value
                    }
                    Ordering::Greater => {
                        right_entry = read_next(&mut right.file)?;
                        right_value
                    }
                    Ordering::Equal => return Err(ControlMediaFault::DuplicateTransitionConflict),
                },
                (Some(value), None) => {
                    left_entry = read_next(&mut left.file)?;
                    value
                }
                (None, Some(value)) => {
                    right_entry = read_next(&mut right.file)?;
                    value
                }
                (None, None) => break,
            };
            write_entry(&mut output, next.0, next.1)?;
        }
        output.seek(SeekFrom::Start(0))?;
        Ok(Self {
            file: output,
            entries: left
                .entries
                .checked_add(right.entries)
                .ok_or(ControlMediaFault::GenerationExhausted)?,
        })
    }

    fn read_at(
        &mut self,
        index: u64,
    ) -> Result<(CanonicalDigestId, ArchivedWorkflowKind), ControlMediaFault> {
        let offset = index
            .checked_mul(ENTRY_BYTES as u64)
            .ok_or(ControlMediaFault::DerivedTransitionIndexCorrupt)?;
        self.file.seek(SeekFrom::Start(offset))?;
        read_next(&mut self.file)?.ok_or(ControlMediaFault::DerivedTransitionIndexCorrupt)
    }
}

fn operation_key(operation: &OperationalOperationId) -> CanonicalDigestId {
    let mut digest = Sha256::new();
    digest.update(b"worth-store-archived-workflow-index-v1\0");
    digest.update(operation.as_str().as_bytes());
    CanonicalDigestId::new(digest.finalize().into())
}

fn write_entry(
    file: &mut File,
    key: CanonicalDigestId,
    kind: ArchivedWorkflowKind,
) -> Result<(), ControlMediaFault> {
    file.write_all(key.bytes())?;
    file.write_all(&[encode_kind(kind)])?;
    Ok(())
}

fn read_next(
    file: &mut File,
) -> Result<Option<(CanonicalDigestId, ArchivedWorkflowKind)>, ControlMediaFault> {
    let mut bytes = [0; ENTRY_BYTES];
    let first = file.read(&mut bytes)?;
    if first == 0 {
        return Ok(None);
    }
    file.read_exact(&mut bytes[first..])
        .map_err(|_| ControlMediaFault::DerivedTransitionIndexCorrupt)?;
    let mut key = [0; 32];
    key.copy_from_slice(&bytes[..32]);
    Ok(Some((CanonicalDigestId::new(key), decode_kind(bytes[32])?)))
}

const fn encode_kind(kind: ArchivedWorkflowKind) -> u8 {
    match kind {
        ArchivedWorkflowKind::BackupTerminal => 1,
        ArchivedWorkflowKind::NonBackup(OperationalWorkflowKind::OfflineInspection) => 2,
        ArchivedWorkflowKind::NonBackup(OperationalWorkflowKind::Restore) => 3,
        ArchivedWorkflowKind::NonBackup(OperationalWorkflowKind::PointInTimeRecovery) => 4,
        ArchivedWorkflowKind::NonBackup(OperationalWorkflowKind::Rollback) => 5,
        ArchivedWorkflowKind::NonBackup(OperationalWorkflowKind::Repair) => 6,
        ArchivedWorkflowKind::NonBackup(OperationalWorkflowKind::ReplicaBootstrap) => 7,
        ArchivedWorkflowKind::NonBackup(OperationalWorkflowKind::ReplicaPromotion) => 8,
        ArchivedWorkflowKind::NonBackup(OperationalWorkflowKind::ForensicAcquisition) => 9,
        ArchivedWorkflowKind::NonBackup(OperationalWorkflowKind::Backup) => 10,
    }
}

fn decode_kind(encoded: u8) -> Result<ArchivedWorkflowKind, ControlMediaFault> {
    let kind = match encoded {
        1 => ArchivedWorkflowKind::BackupTerminal,
        2 => ArchivedWorkflowKind::NonBackup(OperationalWorkflowKind::OfflineInspection),
        3 => ArchivedWorkflowKind::NonBackup(OperationalWorkflowKind::Restore),
        4 => ArchivedWorkflowKind::NonBackup(OperationalWorkflowKind::PointInTimeRecovery),
        5 => ArchivedWorkflowKind::NonBackup(OperationalWorkflowKind::Rollback),
        6 => ArchivedWorkflowKind::NonBackup(OperationalWorkflowKind::Repair),
        7 => ArchivedWorkflowKind::NonBackup(OperationalWorkflowKind::ReplicaBootstrap),
        8 => ArchivedWorkflowKind::NonBackup(OperationalWorkflowKind::ReplicaPromotion),
        9 => ArchivedWorkflowKind::NonBackup(OperationalWorkflowKind::ForensicAcquisition),
        10 => ArchivedWorkflowKind::NonBackup(OperationalWorkflowKind::Backup),
        _ => return Err(ControlMediaFault::DerivedTransitionIndexCorrupt),
    };
    Ok(kind)
}

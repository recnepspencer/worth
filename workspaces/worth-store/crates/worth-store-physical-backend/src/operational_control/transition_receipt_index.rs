use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use worth_foundational::CanonicalDigestId;

use sha2::{Digest, Sha256};
use worth_store_authority::ControlStoreGeneration;

use super::{ControlMediaFault, DurableControlRecordBytes};

const RECENT_TRANSITION_LIMIT: usize = 1_024;
const INDEX_LEVELS: usize = u64::BITS as usize;
const INDEX_ENTRY_BYTES: usize = 32 + 8 + 32 + 32;

#[derive(Debug)]
pub(super) struct TransitionReceiptIndex {
    recent: HashMap<CanonicalDigestId, IndexedTransitionReceipt>,
    levels: [Option<TransitionIndexSegment>; INDEX_LEVELS],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct IndexedTransitionReceipt {
    key: CanonicalDigestId,
    generation: ControlStoreGeneration,
    payload_digest: CanonicalDigestId,
    prefix_digest: CanonicalDigestId,
}

#[derive(Debug)]
struct TransitionIndexSegment {
    file: File,
    entries: u64,
}

impl TransitionReceiptIndex {
    pub(super) fn empty() -> Self {
        Self {
            recent: HashMap::new(),
            levels: std::array::from_fn(|_| None),
        }
    }

    pub(super) fn lookup(
        &mut self,
        transition_identity: &str,
    ) -> Result<Option<IndexedTransitionReceipt>, ControlMediaFault> {
        let key = transition_key(transition_identity);
        if let Some(receipt) = self.recent.get(&key) {
            return Ok(Some(*receipt));
        }
        for segment in self.levels.iter_mut().flatten() {
            if let Some(receipt) = segment.lookup(key)? {
                return Ok(Some(receipt));
            }
        }
        Ok(None)
    }

    pub(super) fn insert_record(
        &mut self,
        record: &DurableControlRecordBytes,
    ) -> Result<(), ControlMediaFault> {
        if self.lookup(record.transition_identity())?.is_some() {
            return Err(ControlMediaFault::DuplicateTransitionConflict);
        }
        self.insert(IndexedTransitionReceipt::from_record(record))
    }

    pub(super) fn insert_receipt(
        &mut self,
        transition_identity: &str,
        generation: ControlStoreGeneration,
        payload_digest: [u8; 32],
        prefix_digest: [u8; 32],
    ) -> Result<(), ControlMediaFault> {
        self.insert(IndexedTransitionReceipt {
            key: transition_key(transition_identity),
            generation,
            payload_digest: CanonicalDigestId::new(payload_digest),
            prefix_digest: CanonicalDigestId::new(prefix_digest),
        })
    }

    pub(super) fn absorb(
        &mut self,
        mut suffix: TransitionReceiptIndex,
    ) -> Result<(), ControlMediaFault> {
        let mut recent = Vec::new();
        recent
            .try_reserve_exact(suffix.recent.len())
            .map_err(|_| ControlMediaFault::AllocationFailed)?;
        recent.extend(suffix.recent.drain().map(|(_, receipt)| receipt));
        for receipt in recent {
            self.insert(receipt)?;
        }
        for segment in suffix.levels.into_iter().flatten() {
            self.absorb_segment(segment)?;
        }
        Ok(())
    }

    fn insert(&mut self, receipt: IndexedTransitionReceipt) -> Result<(), ControlMediaFault> {
        if self.recent.contains_key(&receipt.key) {
            return Err(ControlMediaFault::DuplicateTransitionConflict);
        }
        if self.recent.len() == RECENT_TRANSITION_LIMIT {
            self.flush_recent()?;
        }
        self.recent
            .try_reserve(1)
            .map_err(|_| ControlMediaFault::AllocationFailed)?;
        self.recent.insert(receipt.key, receipt);
        Ok(())
    }

    fn flush_recent(&mut self) -> Result<(), ControlMediaFault> {
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(self.recent.len())
            .map_err(|_| ControlMediaFault::AllocationFailed)?;
        entries.extend(self.recent.drain().map(|(_, receipt)| receipt));
        entries.sort_unstable_by_key(|receipt| receipt.key);
        self.absorb_segment(TransitionIndexSegment::from_sorted(&entries)?)
    }

    fn absorb_segment(
        &mut self,
        mut segment: TransitionIndexSegment,
    ) -> Result<(), ControlMediaFault> {
        for level in &mut self.levels {
            match level.take() {
                None => {
                    *level = Some(segment);
                    return Ok(());
                }
                Some(existing) => segment = TransitionIndexSegment::merge(existing, segment)?,
            }
        }
        Err(ControlMediaFault::GenerationExhausted)
    }
}

impl IndexedTransitionReceipt {
    fn from_record(record: &DurableControlRecordBytes) -> Self {
        Self {
            key: transition_key(record.transition_identity()),
            generation: record.generation(),
            payload_digest: CanonicalDigestId::new(Sha256::digest(record.payload()).into()),
            prefix_digest: CanonicalDigestId::new(record.prefix_digest()),
        }
    }

    pub(super) const fn generation(self) -> ControlStoreGeneration {
        self.generation
    }

    pub(super) const fn payload_digest(self) -> [u8; 32] {
        *self.payload_digest.bytes()
    }

    pub(super) const fn prefix_digest(self) -> [u8; 32] {
        *self.prefix_digest.bytes()
    }
}

impl TransitionIndexSegment {
    fn from_sorted(entries: &[IndexedTransitionReceipt]) -> Result<Self, ControlMediaFault> {
        let mut file = tempfile::tempfile()?;
        for receipt in entries {
            write_receipt(&mut file, *receipt)?;
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
    ) -> Result<Option<IndexedTransitionReceipt>, ControlMediaFault> {
        let mut low = 0u64;
        let mut high = self.entries;
        while low < high {
            let middle = low + (high - low) / 2;
            let receipt = self.read_at(middle)?;
            match receipt.key.cmp(&key) {
                Ordering::Less => low = middle + 1,
                Ordering::Greater => high = middle,
                Ordering::Equal => return Ok(Some(receipt)),
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
                (Some(left_value), Some(right_value)) => match left_value.key.cmp(&right_value.key)
                {
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
                (Some(left_value), None) => {
                    left_entry = read_next(&mut left.file)?;
                    left_value
                }
                (None, Some(right_value)) => {
                    right_entry = read_next(&mut right.file)?;
                    right_value
                }
                (None, None) => break,
            };
            write_receipt(&mut output, next)?;
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

    fn read_at(&mut self, index: u64) -> Result<IndexedTransitionReceipt, ControlMediaFault> {
        let offset = index
            .checked_mul(INDEX_ENTRY_BYTES as u64)
            .ok_or(ControlMediaFault::DerivedTransitionIndexCorrupt)?;
        self.file.seek(SeekFrom::Start(offset))?;
        read_next(&mut self.file)?.ok_or(ControlMediaFault::DerivedTransitionIndexCorrupt)
    }
}

fn transition_key(transition_identity: &str) -> CanonicalDigestId {
    let mut digest = Sha256::new();
    digest.update(b"worth-store-control-transition-index-v1\0");
    digest.update(transition_identity.as_bytes());
    CanonicalDigestId::new(digest.finalize().into())
}

fn write_receipt(
    file: &mut File,
    receipt: IndexedTransitionReceipt,
) -> Result<(), ControlMediaFault> {
    file.write_all(receipt.key.bytes())?;
    file.write_all(&receipt.generation.get().to_le_bytes())?;
    file.write_all(receipt.payload_digest.bytes())?;
    file.write_all(receipt.prefix_digest.bytes())?;
    Ok(())
}

fn read_next(file: &mut File) -> Result<Option<IndexedTransitionReceipt>, ControlMediaFault> {
    let mut bytes = [0; INDEX_ENTRY_BYTES];
    let first = file.read(&mut bytes)?;
    if first == 0 {
        return Ok(None);
    }
    file.read_exact(&mut bytes[first..])
        .map_err(|_| ControlMediaFault::DerivedTransitionIndexCorrupt)?;
    let mut key = [0; 32];
    key.copy_from_slice(&bytes[..32]);
    let mut generation = [0; 8];
    generation.copy_from_slice(&bytes[32..40]);
    let generation = ControlStoreGeneration::from_raw(u64::from_le_bytes(generation))
        .ok_or(ControlMediaFault::DerivedTransitionIndexCorrupt)?;
    let mut payload_digest = [0; 32];
    payload_digest.copy_from_slice(&bytes[40..72]);
    let mut prefix_digest = [0; 32];
    prefix_digest.copy_from_slice(&bytes[72..104]);
    Ok(Some(IndexedTransitionReceipt {
        key: CanonicalDigestId::new(key),
        generation,
        payload_digest: CanonicalDigestId::new(payload_digest),
        prefix_digest: CanonicalDigestId::new(prefix_digest),
    }))
}

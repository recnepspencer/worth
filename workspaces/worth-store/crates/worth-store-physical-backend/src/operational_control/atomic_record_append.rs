use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::sync::Mutex;

use fs4::FileExt;
use sha2::{Digest, Sha256};
use worth_store_authority::ControlStoreGeneration;

use super::control_tail_state::{
    load_tail_state_from_path, lock_tail, synchronize_tail, ControlTailState,
};
use super::{
    encode_record, extend_prefix_digest, scan_durable_prefix, validate_record_lengths,
    ControlMediaFault, ControlMediaIdentity, ControlMediaLocation, ControlRecoveryObjectHandle,
    DurableControlRecordBytes, PhysicalControlRecoveryObjectStore,
};

mod shared_journal_read;

#[derive(Debug)]
pub struct PhysicalOperationalControlStore {
    location: ControlMediaLocation,
    identity: ControlMediaIdentity,
    recovery_objects: PhysicalControlRecoveryObjectStore,
    tail: Mutex<Option<ControlTailState>>,
}

#[derive(Debug)]
pub struct PhysicalControlStoreInspection {
    identity: ControlMediaIdentity,
    records: Vec<DurableControlRecordBytes>,
    damage: Option<ControlMediaFault>,
    prefix_digest: [u8; 32],
}

#[derive(Debug)]
pub struct PhysicalControlStoreSummary {
    identity: ControlMediaIdentity,
    last_generation: Option<ControlStoreGeneration>,
    record_count: u64,
    damage: Option<ControlMediaFault>,
    prefix_digest: [u8; 32],
}

impl PhysicalControlStoreSummary {
    pub const fn identity(&self) -> ControlMediaIdentity {
        self.identity
    }

    pub const fn last_generation(&self) -> Option<ControlStoreGeneration> {
        self.last_generation
    }

    pub const fn record_count(&self) -> u64 {
        self.record_count
    }

    pub fn damage(&self) -> Option<&ControlMediaFault> {
        self.damage.as_ref()
    }

    pub const fn prefix_digest(&self) -> [u8; 32] {
        self.prefix_digest
    }
}

impl PhysicalControlStoreInspection {
    pub const fn identity(&self) -> ControlMediaIdentity {
        self.identity
    }

    pub fn records(&self) -> &[DurableControlRecordBytes] {
        &self.records
    }

    pub fn last_generation(&self) -> Option<ControlStoreGeneration> {
        self.records
            .last()
            .map(DurableControlRecordBytes::generation)
    }

    pub fn damage(&self) -> Option<&ControlMediaFault> {
        self.damage.as_ref()
    }

    pub const fn prefix_digest(&self) -> [u8; 32] {
        self.prefix_digest
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalControlAppendReceipt {
    generation: ControlStoreGeneration,
    idempotent_replay: bool,
    prefix_records_scanned: u64,
    prefix_digest: [u8; 32],
}

impl PhysicalControlAppendReceipt {
    pub const fn generation(self) -> ControlStoreGeneration {
        self.generation
    }

    pub const fn idempotent_replay(self) -> bool {
        self.idempotent_replay
    }

    pub const fn prefix_records_scanned(self) -> u64 {
        self.prefix_records_scanned
    }

    pub const fn prefix_digest(self) -> [u8; 32] {
        self.prefix_digest
    }
}

impl PhysicalOperationalControlStore {
    pub fn open(location: ControlMediaLocation) -> Result<Self, ControlMediaFault> {
        let existed = location.path().exists();
        if let Some(parent) = location.path().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let journal = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(location.path())?;
        journal.sync_all()?;
        if !existed && let Some(parent) = location.path().parent() {
            crate::directory_durability::sync_directory(parent)?;
        }
        let identity = ControlMediaIdentity::open(&location, &journal)?;
        let recovery_objects = PhysicalControlRecoveryObjectStore::open(&location)?;
        let tail = load_tail_state_from_path(location.path()).ok();
        Ok(Self {
            location,
            identity,
            recovery_objects,
            tail: Mutex::new(tail),
        })
    }

    pub const fn identity(&self) -> ControlMediaIdentity {
        self.identity
    }

    pub fn publish_recovery_object(
        &self,
        content: &[u8],
    ) -> Result<ControlRecoveryObjectHandle, ControlMediaFault> {
        self.verify_current_media_identity()?;
        let handle = self.recovery_objects.publish(content)?;
        self.verify_current_media_identity()?;
        Ok(handle)
    }

    pub fn read_recovery_object(
        &self,
        handle: ControlRecoveryObjectHandle,
    ) -> Result<Vec<u8>, ControlMediaFault> {
        self.verify_current_media_identity()?;
        let content = self.recovery_objects.read(handle)?;
        self.verify_current_media_identity()?;
        Ok(content)
    }

    pub fn inspect(&self) -> Result<PhysicalControlStoreInspection, ControlMediaFault> {
        self.with_shared_journal(|file| {
            let mut records = Vec::new();
            let scan = scan_durable_prefix(file, |record| {
                records
                    .try_reserve(1)
                    .map_err(|_| ControlMediaFault::AllocationFailed)?;
                records.push(record);
                Ok(())
            });
            let (prefix_digest, damage) = match scan {
                Ok(summary) => (summary.prefix_digest(), None),
                Err(damage) => (
                    records
                        .last()
                        .map_or([0; 32], DurableControlRecordBytes::prefix_digest),
                    Some(damage),
                ),
            };
            Ok(PhysicalControlStoreInspection {
                identity: self.identity,
                records,
                damage,
                prefix_digest,
            })
        })
    }

    pub fn inspect_summary(&self) -> Result<PhysicalControlStoreSummary, ControlMediaFault> {
        self.with_shared_journal(|file| {
            let mut last_generation = None;
            let mut record_count = 0u64;
            let mut prefix_digest = [0; 32];
            let result = scan_durable_prefix(file, |record| {
                last_generation = Some(record.generation());
                record_count += 1;
                prefix_digest = record.prefix_digest();
                Ok(())
            });
            let (last_generation, record_count, damage) = match result {
                Ok(summary) => (summary.last_generation(), summary.record_count(), None),
                Err(damage) => (last_generation, record_count, Some(damage)),
            };
            Ok(PhysicalControlStoreSummary {
                identity: self.identity,
                last_generation,
                record_count,
                damage,
                prefix_digest,
            })
        })
    }

    pub fn visit_records(
        &self,
        mut observe: impl FnMut(DurableControlRecordBytes),
    ) -> Result<PhysicalControlStoreSummary, ControlMediaFault> {
        self.with_shared_journal(|file| {
            let summary = scan_durable_prefix(file, |record| {
                observe(record);
                Ok(())
            })?;
            Ok(PhysicalControlStoreSummary {
                identity: self.identity,
                last_generation: summary.last_generation(),
                record_count: summary.record_count(),
                damage: None,
                prefix_digest: summary.prefix_digest(),
            })
        })
    }

    pub fn observe_current_prefix(
        &self,
    ) -> Result<Option<(ControlStoreGeneration, [u8; 32])>, ControlMediaFault> {
        let summary = self.inspect_summary()?;
        if let Some(damage) = summary.damage {
            return Err(damage);
        }
        Ok(summary
            .last_generation
            .map(|generation| (generation, summary.prefix_digest)))
    }

    pub fn compare_exchange_append(
        &self,
        expected: Option<ControlStoreGeneration>,
        transition_identity: &str,
        payload: &[u8],
    ) -> Result<PhysicalControlAppendReceipt, ControlMediaFault> {
        let mut file = read_write_file(self.location.path())?;
        file.lock_exclusive()?;
        let result = lock_tail(&self.tail).and_then(|mut tail| {
            self.verify_open_journal(&file)?;
            let receipt = append_locked(
                &mut file,
                &mut tail,
                Some(expected),
                transition_identity,
                payload,
            );
            if receipt.is_err() {
                *tail = None;
            }
            let receipt = receipt?;
            self.verify_open_journal(&file)?;
            Ok(receipt)
        });
        let unlock_result = file.unlock();
        match (result, unlock_result) {
            (Ok(receipt), Ok(())) => Ok(receipt),
            (Err(error), _) => Err(error),
            (Ok(_), Err(error)) => Err(error.into()),
        }
    }

    pub fn append_at_current_tail(
        &self,
        transition_identity: &str,
        payload: &[u8],
    ) -> Result<PhysicalControlAppendReceipt, ControlMediaFault> {
        let mut file = read_write_file(self.location.path())?;
        file.lock_exclusive()?;
        let result = lock_tail(&self.tail).and_then(|mut tail| {
            self.verify_open_journal(&file)?;
            let receipt = append_locked(&mut file, &mut tail, None, transition_identity, payload);
            if receipt.is_err() {
                *tail = None;
            }
            let receipt = receipt?;
            self.verify_open_journal(&file)?;
            Ok(receipt)
        });
        let unlock_result = file.unlock();
        match (result, unlock_result) {
            (Ok(receipt), Ok(())) => Ok(receipt),
            (Err(error), _) => Err(error),
            (Ok(_), Err(error)) => Err(error.into()),
        }
    }

    fn verify_current_media_identity(&self) -> Result<(), ControlMediaFault> {
        self.with_shared_journal(|_| Ok(()))
    }

    fn verify_open_journal(&self, file: &File) -> Result<(), ControlMediaFault> {
        let observed = ControlMediaIdentity::observe(&self.location, file)?;
        if observed != self.identity {
            return Err(ControlMediaFault::ControlMediaIdentityChanged {
                expected: self.identity.fingerprint(),
                observed: observed.fingerprint(),
            });
        }
        Ok(())
    }
}

fn read_write_file(path: &std::path::Path) -> Result<File, ControlMediaFault> {
    Ok(OpenOptions::new().read(true).write(true).open(path)?)
}

fn append_locked(
    file: &mut File,
    tail: &mut Option<ControlTailState>,
    expected: Option<Option<ControlStoreGeneration>>,
    transition_identity: &str,
    payload: &[u8],
) -> Result<PhysicalControlAppendReceipt, ControlMediaFault> {
    validate_record_lengths(transition_identity, payload)?;
    let (prefix_records_scanned, state) = synchronize_tail(file, tail)?;
    let payload_digest = Sha256::digest(payload).into();
    if let Some(existing) = state.transition_receipts.lookup(transition_identity)? {
        if existing.payload_digest() != payload_digest {
            return Err(ControlMediaFault::DuplicateTransitionConflict);
        }
        return Ok(PhysicalControlAppendReceipt {
            generation: existing.generation(),
            idempotent_replay: true,
            prefix_records_scanned,
            prefix_digest: existing.prefix_digest(),
        });
    }
    let actual = state.last_generation;
    if let Some(expected) = expected
        && actual != expected
    {
        return Err(ControlMediaFault::GenerationMismatch { expected, actual });
    }
    let generation = match actual {
        Some(current) => current
            .next()
            .ok_or(ControlMediaFault::GenerationExhausted)?,
        None => ControlStoreGeneration::initial(),
    };
    let frame = encode_record(
        generation,
        state.prefix_digest,
        transition_identity,
        payload,
    )?;
    let mut frame_checksum = [0; 32];
    frame_checksum.copy_from_slice(&frame[frame.len() - 32..]);
    file.seek(SeekFrom::End(0))?;
    file.write_all(&frame)?;
    file.sync_all()?;
    state.observed_bytes = state
        .observed_bytes
        .checked_add(frame.len() as u64)
        .ok_or(ControlMediaFault::GenerationExhausted)?;
    state.last_generation = Some(generation);
    state.record_count = state
        .record_count
        .checked_add(1)
        .ok_or(ControlMediaFault::GenerationExhausted)?;
    state.prefix_digest = extend_prefix_digest(state.prefix_digest, frame_checksum);
    state.last_frame_checksum = Some(frame_checksum);
    state.transition_receipts.insert_receipt(
        transition_identity,
        generation,
        payload_digest,
        state.prefix_digest,
    )?;
    state.observed_modified = file.metadata()?.modified()?;
    Ok(PhysicalControlAppendReceipt {
        generation,
        idempotent_replay: false,
        prefix_records_scanned,
        prefix_digest: state.prefix_digest,
    })
}

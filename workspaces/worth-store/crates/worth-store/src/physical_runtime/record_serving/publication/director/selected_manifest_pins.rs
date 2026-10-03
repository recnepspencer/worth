//! Selected C.11 manifest custody for checkpoint binding retention.

use std::collections::HashSet;

use worth_store_physical_format::{
    decode_blob_record, BlobRecordKind, BlobRecordV1, PersistedRecordIdentity, RootPublicationCell,
    BLOB_RECORD_HEADER_BYTES, MAXIMUM_DROP_SET_RECORDS,
};

use super::RecordPublicationDirector;
use crate::physical_runtime::record_serving::{
    PhysicalRecordId, PhysicalRecordReader, RecordByteLimit, RecordCountLimit, RecordReadLimits,
    RecordScanOutcome, RecordScanRequest,
};

#[path = "selected_manifest_pins/budget.rs"]
mod budget;
#[path = "selected_manifest_pins/read_deferred.rs"]
mod deferred_read;

pub(in crate::physical_runtime) use budget::checkpoint_pin_scan_bytes;
use budget::{checkpoint_pin_scan_bound, PinScanBound};
use deferred_read::read_deferred;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) struct SelectedBlobManifestPin {
    record: PersistedRecordIdentity,
    store: [u8; 16],
    attempt: [u8; 16],
    basis_digest: [u8; 32],
}

pub(in crate::physical_runtime) struct SelectedBlobManifestPins {
    root: RootPublicationCell,
    pins: Vec<SelectedBlobManifestPin>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum SelectedBlobManifestPinDenial {
    RootUnavailable,
    ScanUnavailable,
    ReadUnavailable,
    MalformedManifest,
    ConflictingAttempt,
    BudgetExceeded,
}

impl SelectedBlobManifestPin {
    pub(in crate::physical_runtime) const fn store(self) -> [u8; 16] {
        self.store
    }

    pub(in crate::physical_runtime) const fn attempt(self) -> [u8; 16] {
        self.attempt
    }
}

impl SelectedBlobManifestPins {
    pub(in crate::physical_runtime) const fn root(&self) -> RootPublicationCell {
        self.root
    }

    pub(in crate::physical_runtime) fn pins(&self) -> &[SelectedBlobManifestPin] {
        &self.pins
    }

    #[cfg(test)]
    pub(in crate::physical_runtime) fn empty_for_test() -> Self {
        let root = worth_store_physical_format::DurablePhysicalRootManifest::builder(1, 1, 2, 1)
            .admit()
            .expect("test root")
            .root_cell();
        Self {
            root,
            pins: Vec::new(),
        }
    }

    #[cfg(test)]
    pub(in crate::physical_runtime) fn one_for_test(store: [u8; 16], attempt: [u8; 16]) -> Self {
        let mut selected = Self::empty_for_test();
        selected.pins.push(SelectedBlobManifestPin {
            record: PersistedRecordIdentity::new([9; 16], 1).expect("test record"),
            store,
            attempt,
            basis_digest: [7; 32],
        });
        selected
    }
}

impl RecordPublicationDirector {
    /// Captures a protected selected root and reads its canonical C.5 routes.
    /// Failure leaves checkpoint publication unavailable; it cannot erase pins.
    pub(in crate::physical_runtime) fn selected_blob_manifest_pins(
        &self,
        maximum_pins: usize,
        memory_budget: u64,
    ) -> Result<SelectedBlobManifestPins, SelectedBlobManifestPinDenial> {
        let (root, protection) = self
            .capture_read_root()
            .map_err(|_| SelectedBlobManifestPinDenial::RootUnavailable)?;
        let runtime = self
            .runtime
            .upgrade()
            .ok_or(SelectedBlobManifestPinDenial::RootUnavailable)?;
        let root_cell = root.root_cell();
        // The roster is held by the standing capture reservation, whose
        // envelope includes this exact bound; the scan takes no pool bytes.
        let PinScanBound {
            scratch_bytes,
            roster_bytes: roster_budget,
        } = checkpoint_pin_scan_bound(
            maximum_pins,
            memory_budget,
            u64::from(self.format.declaration().page_size().bytes()),
        )?;
        let reader = PhysicalRecordReader {
            execution: crate::physical_runtime::instance::PhysicalStoreWorkRuntime::execution(
                &runtime,
                self.generation,
            ),
            protection,
            store: self.durability.store_identity(),
            format: self.format,
            access: self.access,
            current_root: root,
            generation: self.generation,
            runtime: std::sync::Arc::downgrade(&runtime),
            lifecycle: self.reader_factory.reader(),
            residency: self.residency.clone().for_rebuild(),
        };
        let mut scratch = Vec::new();
        scratch
            .try_reserve_exact(scratch_bytes)
            .map_err(|_| SelectedBlobManifestPinDenial::BudgetExceeded)?;
        scratch.resize(scratch_bytes, 0);
        let mut scan = reader
            .scan_rebuild(
                RecordScanRequest::from_start()
                    .with_batch_limit(RecordCountLimit::new(1).expect("one selected route"))
                    .with_payload_limit(
                        RecordByteLimit::new(scratch_bytes as u32)
                            .expect("control frame limit is nonzero"),
                    ),
            )
            .map_err(|_| SelectedBlobManifestPinDenial::ScanUnavailable)?;
        let mut pins = Vec::new();
        let mut attempts = HashSet::new();
        loop {
            let (complete, deferred) = match scan
                .read_next_into(&mut scratch)
                .map_err(|_| SelectedBlobManifestPinDenial::ScanUnavailable)?
            {
                RecordScanOutcome::Completed(_) => break,
                RecordScanOutcome::Batch(batch) => {
                    let row = &batch.records()[0];
                    let record = row.record_id().persisted();
                    let deferred = if let Some(bytes) = batch.payload(0) {
                        observe_manifest(
                            bytes,
                            record,
                            self.durability.store_identity().bytes(),
                            root_cell.generation().get(),
                            maximum_pins,
                            roster_budget,
                            scratch_bytes,
                            &mut pins,
                            &mut attempts,
                        )?;
                        None
                    } else {
                        Some((record, row.declared_payload_bytes()))
                    };
                    (batch.is_complete(), deferred)
                }
            };
            if let Some((record, bytes)) = deferred {
                if bytes <= scratch_bytes as u64 {
                    let used = read_deferred(scan.protected_reader(), record, bytes, &mut scratch)?;
                    observe_manifest(
                        &scratch[..used],
                        record,
                        self.durability.store_identity().bytes(),
                        root_cell.generation().get(),
                        maximum_pins,
                        roster_budget,
                        scratch_bytes,
                        &mut pins,
                        &mut attempts,
                    )?;
                } else if is_oversized_manifest(scan.protected_reader(), record)? {
                    return Err(SelectedBlobManifestPinDenial::BudgetExceeded);
                }
            }
            if complete {
                break;
            }
        }
        Ok(SelectedBlobManifestPins {
            root: root_cell,
            pins,
        })
    }
}

fn observe_manifest(
    bytes: &[u8],
    record: PersistedRecordIdentity,
    store: [u8; 16],
    selected_root_generation: u64,
    maximum_pins: usize,
    memory_budget: u64,
    scratch_bytes: usize,
    pins: &mut Vec<SelectedBlobManifestPin>,
    attempts: &mut HashSet<[u8; 16]>,
) -> Result<(), SelectedBlobManifestPinDenial> {
    if !bytes.starts_with(b"WRC11BLB") || !is_manifest_kind(bytes.get(8).copied()) {
        return Ok(());
    }
    // The count is a bounded wire field. Charge both temporary identity arrays
    // used by the decoder, plus worst-case roster/hash capacities, before it
    // allocates either array or an attempt-index bucket.
    let count_offset = manifest_count_offset(bytes)?;
    let count = bytes
        .get(count_offset..count_offset + 2)
        .map(|raw| u16::from_le_bytes([raw[0], raw[1]]) as usize)
        .filter(|count| (1..=MAXIMUM_DROP_SET_RECORDS).contains(count))
        .ok_or(SelectedBlobManifestPinDenial::MalformedManifest)?;
    if pins.len() >= maximum_pins {
        return Err(SelectedBlobManifestPinDenial::BudgetExceeded);
    }
    let next = pins.len() + 1;
    let temporary = count
        .checked_mul(std::mem::size_of::<PersistedRecordIdentity>())
        .and_then(|bytes| bytes.checked_mul(2))
        .ok_or(SelectedBlobManifestPinDenial::BudgetExceeded)?;
    let pin_capacity = pins.capacity().max(next.saturating_mul(4));
    // HashSet's bucket array includes control bytes and may grow ahead of len.
    // Four 32-byte buckets per entry conservatively covers its growth and
    // avoids a per-insert tree allocation with an opaque node layout.
    let attempt_capacity = attempts.capacity().max(next.saturating_mul(4));
    let charged = scratch_bytes
        .checked_add(temporary)
        .and_then(|bytes| {
            bytes.checked_add(
                pin_capacity.checked_mul(std::mem::size_of::<SelectedBlobManifestPin>())?,
            )
        })
        .and_then(|bytes| bytes.checked_add(attempt_capacity.checked_mul(32)?))
        // Binding compaction copies one 32-byte material per pin. Include a
        // conservative second array for allocator growth at the handoff.
        .and_then(|bytes| bytes.checked_add(next.checked_mul(64)?))
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<HashSet<[u8; 16]>>()))
        .ok_or(SelectedBlobManifestPinDenial::BudgetExceeded)?;
    if charged as u64 > memory_budget {
        return Err(SelectedBlobManifestPinDenial::BudgetExceeded);
    }
    pins.try_reserve_exact(1)
        .map_err(|_| SelectedBlobManifestPinDenial::BudgetExceeded)?;
    attempts
        .try_reserve(1)
        .map_err(|_| SelectedBlobManifestPinDenial::BudgetExceeded)?;
    let decoded =
        decode_blob_record(bytes).map_err(|_| SelectedBlobManifestPinDenial::MalformedManifest)?;
    let (manifest_store, attempt, basis_digest) = match decoded {
        BlobRecordV1::DropSetManifest(manifest) => (
            manifest.store(),
            manifest.reclaim_attempt(),
            manifest.source_basis_digest(),
        ),
        BlobRecordV1::DropSetManifestV2(manifest)
            if manifest.never_reserved_slot_generation() <= selected_root_generation =>
        {
            (
                manifest.store(),
                manifest.reclaim_attempt(),
                manifest.source_basis_digest(),
            )
        }
        BlobRecordV1::DropSetManifestV3(manifest)
            if manifest.never_reserved_slot_generation() <= selected_root_generation =>
        {
            (
                manifest.store(),
                manifest.reclaim_attempt(),
                manifest.source_basis_digest(),
            )
        }
        _ => return Err(SelectedBlobManifestPinDenial::MalformedManifest),
    };
    if manifest_store != store || !attempts.insert(attempt) {
        return Err(SelectedBlobManifestPinDenial::ConflictingAttempt);
    }
    pins.push(SelectedBlobManifestPin {
        record,
        store,
        attempt,
        basis_digest,
    });
    Ok(())
}

fn is_manifest_kind(kind: Option<u8>) -> bool {
    matches!(kind, Some(value) if value == BlobRecordKind::DropSetManifest as u8
        || value == BlobRecordKind::DropSetManifestV2 as u8
        || value == BlobRecordKind::DropSetManifestV3 as u8)
}

const MANIFEST_COUNT_OFFSET: usize = BLOB_RECORD_HEADER_BYTES + 160;
const MINIMUM_MANIFEST_FRAME_BYTES: usize = BLOB_RECORD_HEADER_BYTES + 194 + 24;

fn manifest_count_offset(bytes: &[u8]) -> Result<usize, SelectedBlobManifestPinDenial> {
    if bytes.get(8).copied() != Some(BlobRecordKind::DropSetManifestV3 as u8) {
        return Ok(MANIFEST_COUNT_OFFSET);
    }
    // V3 carries a length-tagged source basis; unlike V1/V2 its count has no
    // fixed offset. Inspect only the fixed prefix before charging decoder
    // storage for the untrusted count.
    let source_length_offset = BLOB_RECORD_HEADER_BYTES + 33;
    let source_length = bytes
        .get(source_length_offset..source_length_offset + 2)
        .map(|raw| u16::from_le_bytes([raw[0], raw[1]]) as usize)
        .ok_or(SelectedBlobManifestPinDenial::MalformedManifest)?;
    (BLOB_RECORD_HEADER_BYTES + 35)
        .checked_add(source_length)
        .ok_or(SelectedBlobManifestPinDenial::MalformedManifest)
}

fn is_oversized_manifest(
    reader: &PhysicalRecordReader,
    record: PersistedRecordIdentity,
) -> Result<bool, SelectedBlobManifestPinDenial> {
    let mut stream = reader
        .open(
            PhysicalRecordId::from_persisted(record),
            RecordReadLimits::new(RecordByteLimit::new(u32::MAX).expect("nonzero byte limit")),
        )
        .map_err(|_| SelectedBlobManifestPinDenial::ReadUnavailable)?;
    let mut prefix = [0_u8; 9];
    let mut used = 0;
    while used < prefix.len() {
        let read = stream
            .read_next(&mut prefix[used..])
            .map_err(|_| SelectedBlobManifestPinDenial::ReadUnavailable)?;
        if read == 0 {
            return Err(SelectedBlobManifestPinDenial::ReadUnavailable);
        }
        used += read;
    }
    Ok(&prefix[..8] == b"WRC11BLB" && is_manifest_kind(Some(prefix[8])))
}

#[cfg(test)]
#[path = "selected_manifest_pins/tests.rs"]
mod tests;

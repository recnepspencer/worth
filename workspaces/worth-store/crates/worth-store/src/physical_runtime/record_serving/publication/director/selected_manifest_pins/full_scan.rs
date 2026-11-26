//! Full rebuild of a selected root's manifest roster over every C.5 route.
//!
//! The cold path: a director with no carried roster, another routing tree, or
//! an older root reads every selected route once.

use std::collections::HashSet;

use worth_store_physical_format::PersistedRecordIdentity;

use super::deferred_read::read_deferred;
use super::{
    is_manifest_kind, RosterLimits, SelectedBlobManifestPin, SelectedBlobManifestPinDenial,
};
use crate::physical_runtime::record_serving::{
    PhysicalRecordId, PhysicalRecordReader, RecordByteLimit, RecordCountLimit, RecordReadLimits,
    RecordScanOutcome, RecordScanRequest,
};

pub(super) fn full_scan(
    reader: PhysicalRecordReader,
    limits: RosterLimits,
    scratch: &mut [u8],
) -> Result<Vec<SelectedBlobManifestPin>, SelectedBlobManifestPinDenial> {
    let scratch_bytes = scratch.len();
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
            .read_next_into(scratch)
            .map_err(|_| SelectedBlobManifestPinDenial::ScanUnavailable)?
        {
            RecordScanOutcome::Completed(_) => break,
            RecordScanOutcome::Batch(batch) => {
                let row = &batch.records()[0];
                let record = row.record_id().persisted();
                let deferred = if let Some(bytes) = batch.payload(0) {
                    limits.observe(bytes, record, &mut pins, &mut attempts)?;
                    None
                } else {
                    Some((record, row.declared_payload_bytes()))
                };
                (batch.is_complete(), deferred)
            }
        };
        if let Some((record, bytes)) = deferred {
            if bytes <= scratch_bytes as u64 {
                let used = read_deferred(scan.protected_reader(), record, bytes, scratch)?;
                limits.observe(&scratch[..used], record, &mut pins, &mut attempts)?;
            } else if is_oversized_manifest(scan.protected_reader(), record)? {
                return Err(SelectedBlobManifestPinDenial::BudgetExceeded);
            }
        }
        if complete {
            break;
        }
    }
    Ok(pins)
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

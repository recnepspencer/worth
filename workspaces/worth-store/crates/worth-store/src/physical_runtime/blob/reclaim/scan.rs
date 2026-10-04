use worth_store_physical_format::{
    decode_blob_record, BlobRecordDenial, PersistedRecordIdentity, PhysicalTierClass,
    SelectedRecordContentClass,
};

use crate::physical_runtime::{
    PhysicalRecordId, PhysicalRecordReader, RecordByteLimit, RecordCountLimit, RecordReadLimits,
    RecordScanOutcome, RecordScanRequest,
};

use super::{BlobReclaimFailure, BlobReclaimLimits};

// This canonical frame ceiling covers the current 256KiB chunk and the
// version-one maximum tree. Decoded tree storage is separately in the grant.
pub(super) const FRAME_WINDOW_BYTES: usize =
    worth_store_physical_format::BLOB_TREE_NODE_FRAME_MAX_BYTES;
const BLOB_MAGIC: &[u8; 8] = b"WRC11BLB";

#[derive(Clone, Copy, Default)]
pub(super) struct ReclaimInspectionWork {
    pub(super) records: u64,
    pub(super) bytes: u64,
}

impl ReclaimInspectionWork {
    pub(super) fn checked_add(self, other: Self) -> Result<Self, BlobReclaimFailure> {
        Ok(Self {
            records: self
                .records
                .checked_add(other.records)
                .ok_or(BlobReclaimFailure::ScanBoundExhausted)?,
            bytes: self
                .bytes
                .checked_add(other.bytes)
                .ok_or(BlobReclaimFailure::InspectedByteBoundExhausted)?,
        })
    }
}

/// The one frame window every reclaim pass reads through. It lives inside
/// the operation's admitted blob allocation.
pub(super) fn frame_window() -> Result<Vec<u8>, BlobReclaimFailure> {
    let mut scratch = Vec::new();
    scratch
        .try_reserve_exact(FRAME_WINDOW_BYTES)
        .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
    scratch.resize(FRAME_WINDOW_BYTES, 0);
    if scratch.capacity() != FRAME_WINDOW_BYTES {
        return Err(BlobReclaimFailure::ScratchUnavailable);
    }
    Ok(scratch)
}

pub(super) fn walk(
    reader: PhysicalRecordReader,
    limits: BlobReclaimLimits,
    scratch: &mut [u8],
    work: &mut ReclaimInspectionWork,
    mut visit: impl FnMut(PersistedRecordIdentity, &[u8]) -> Result<(), BlobReclaimFailure>,
) -> Result<PhysicalRecordReader, BlobReclaimFailure> {
    walk_classified(reader, limits, scratch, work, |record, _, _, _, bytes| {
        visit(record, bytes)
    })
}

/// Walks every selected route under one protected root. The class is taken
/// from the durable C.5 route, not inferred from a payload prefix.
pub(super) fn walk_classified(
    reader: PhysicalRecordReader,
    limits: BlobReclaimLimits,
    scratch: &mut [u8],
    work: &mut ReclaimInspectionWork,
    mut visit: impl FnMut(
        PersistedRecordIdentity,
        SelectedRecordContentClass,
        PhysicalTierClass,
        u64,
        &[u8],
    ) -> Result<(), BlobReclaimFailure>,
) -> Result<PhysicalRecordReader, BlobReclaimFailure> {
    let mut scan = reader
        .scan_rebuild(
            RecordScanRequest::from_start()
                .with_batch_limit(RecordCountLimit::new(1).expect("one scan row"))
                .with_payload_limit(RecordByteLimit::new(FRAME_WINDOW_BYTES as u32).unwrap()),
        )
        .map_err(BlobReclaimFailure::Scan)?;
    let mut examined = 0;
    let mut previous_record = None;
    loop {
        if examined == limits.maximum_selected_records() {
            return Err(BlobReclaimFailure::ScanBoundExhausted);
        }
        let mut deferred = None;
        let complete = match scan
            .read_next_into(scratch)
            .map_err(BlobReclaimFailure::Scan)?
        {
            RecordScanOutcome::Completed(_) => break,
            RecordScanOutcome::Batch(batch) => {
                for (index, row) in batch.records().iter().enumerate() {
                    examined += 1;
                    work.records = work
                        .records
                        .checked_add(1)
                        .ok_or(BlobReclaimFailure::ScanBoundExhausted)?;
                    let id = PersistedRecordIdentity::new(
                        row.record_id().allocation_epoch(),
                        row.record_id().ordinal(),
                    )
                    .ok_or(BlobReclaimFailure::Format(
                        BlobRecordDenial::InvalidIdentity,
                    ))?;
                    if previous_record.is_some_and(|previous| previous >= id) {
                        return Err(BlobReclaimFailure::ConflictingSelectedFate);
                    }
                    previous_record = Some(id);
                    if let Some(payload) = batch.payload(index) {
                        admit_inspected_bytes(work, limits, payload.len())?;
                        validate_classification(row.content_class(), payload)?;
                        visit(
                            id,
                            row.content_class(),
                            row.tier_class(),
                            row.declared_payload_bytes(),
                            payload,
                        )?;
                    } else {
                        deferred = Some((
                            id,
                            row.content_class(),
                            row.tier_class(),
                            row.declared_payload_bytes(),
                        ));
                    }
                }
                batch.is_complete()
            }
        };
        if let Some((id, class, tier, declared)) = deferred {
            let used = read_selected(scan.protected_reader(), id, declared, scratch, limits, work)?;
            if declared <= scratch.len() as u64 {
                validate_classification(class, &scratch[..used])?;
                visit(id, class, tier, declared, &scratch[..used])?;
            } else if matches!(class, SelectedRecordContentClass::Blob(_))
                || scratch[..used].starts_with(BLOB_MAGIC)
            {
                return Err(BlobReclaimFailure::InspectionWindowExhausted);
            } else {
                validate_classification(class, &scratch[..used])?;
                visit(id, class, tier, declared, &scratch[..used])?;
            }
        }
        if complete {
            break;
        }
    }
    Ok(scan.into_protected_reader())
}

fn validate_classification(
    class: SelectedRecordContentClass,
    bytes: &[u8],
) -> Result<(), BlobReclaimFailure> {
    match class {
        SelectedRecordContentClass::Blob(kind) => {
            let decoded = decode_blob_record(bytes).map_err(BlobReclaimFailure::Format)?;
            if decoded.kind() != kind {
                return Err(BlobReclaimFailure::ConflictingSelectedFate);
            }
        }
        SelectedRecordContentClass::UnknownLegacy => {}
        _ if bytes.starts_with(BLOB_MAGIC) => {
            return Err(BlobReclaimFailure::ConflictingSelectedFate);
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn read_selected(
    reader: &PhysicalRecordReader,
    id: PersistedRecordIdentity,
    declared: u64,
    scratch: &mut [u8],
    limits: BlobReclaimLimits,
    work: &mut ReclaimInspectionWork,
) -> Result<usize, BlobReclaimFailure> {
    let limit = u32::try_from(declared)
        .ok()
        .and_then(RecordByteLimit::new)
        .ok_or(BlobReclaimFailure::Format(BlobRecordDenial::FrameTooLarge))?;
    let mut stream = reader
        .open(
            PhysicalRecordId::from_persisted(id),
            RecordReadLimits::new(limit),
        )
        .map_err(BlobReclaimFailure::Read)?;
    let wanted = if declared > scratch.len() as u64 {
        BLOB_MAGIC.len()
    } else {
        declared as usize
    };
    admit_inspected_bytes(work, limits, wanted)?;
    let mut used = 0;
    while used < wanted {
        let count = stream
            .read_next(&mut scratch[used..wanted])
            .map_err(BlobReclaimFailure::Stream)?;
        if count == 0 {
            return Err(BlobReclaimFailure::Format(BlobRecordDenial::Truncated));
        }
        used += count;
    }
    Ok(used)
}

pub(super) fn admit_inspected_bytes(
    work: &mut ReclaimInspectionWork,
    limits: BlobReclaimLimits,
    bytes: usize,
) -> Result<(), BlobReclaimFailure> {
    let total = work
        .bytes
        .checked_add(bytes as u64)
        .filter(|total| *total <= limits.maximum_inspected_bytes())
        .ok_or(BlobReclaimFailure::InspectedByteBoundExhausted)?;
    work.bytes = total;
    Ok(())
}

#[cfg(test)]
mod tests {
    use worth_store_physical_format::{
        BlobChunkFrameV1, BlobChunkOccurrenceV1, BlobRecordKind, SelectedRecordContentClass,
    };

    use super::validate_classification;

    fn chunk() -> Vec<u8> {
        BlobChunkFrameV1::encode(
            BlobChunkOccurrenceV1::new([1; 16], [2; 16], 0).unwrap(),
            64 * 1024,
            b"payload",
        )
        .unwrap()
    }

    #[test]
    fn selected_class_rejects_resealed_blob_under_opaque_route() {
        assert!(validate_classification(SelectedRecordContentClass::Opaque, &chunk()).is_err());
    }

    #[test]
    fn selected_class_rejects_wrong_blob_kind() {
        assert!(validate_classification(
            SelectedRecordContentClass::Blob(BlobRecordKind::TreeNode),
            &chunk(),
        )
        .is_err());
    }

    #[test]
    fn selected_class_accepts_exact_authenticated_chunk() {
        assert!(validate_classification(
            SelectedRecordContentClass::Blob(BlobRecordKind::Chunk),
            &chunk(),
        )
        .is_ok());
    }
}

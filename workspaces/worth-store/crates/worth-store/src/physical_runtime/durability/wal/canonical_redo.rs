use sha2::{Digest, Sha256};
use worth_proof::{CanonicalVec, NonEmpty};
use worth_store_physical_format::PersistedPhysicalRecoveryProjection;
use worth_store_wal::{LogSequenceNumber, WalLsnRange};

use crate::physical_runtime::durability::PhysicalRedoTargetClaim;

const REDO_DOMAIN: &[u8] = b"store.physical.wal.canonical-redo.v3";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedoRecord {
    ordinal: u32,
    lsn: LogSequenceNumber,
    targets: CanonicalVec<PhysicalRedoTargetClaim>,
    bytes: Vec<u8>,
    source_copy: Option<worth_store_physical_format::PersistedExtentCopyRecipe>,
}

impl Ord for RedoRecord {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ordinal
            .cmp(&other.ordinal)
            .then_with(|| self.lsn.cmp(&other.lsn))
            .then_with(|| self.targets.as_slice().cmp(other.targets.as_slice()))
            .then_with(|| self.bytes.cmp(&other.bytes))
            .then_with(|| {
                self.source_copy
                    .map(|recipe| (recipe.intent_digest(), recipe.intent_lsn()))
                    .cmp(
                        &other
                            .source_copy
                            .map(|recipe| (recipe.intent_digest(), recipe.intent_lsn())),
                    )
            })
    }
}

impl PartialOrd for RedoRecord {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalRedoRecords {
    records: CanonicalVec<RedoRecord>,
    encoded: Vec<u8>,
    digest: [u8; 32],
}

impl CanonicalRedoRecords {
    pub(in crate::physical_runtime) fn from_prepared_records(
        records: Vec<Vec<u8>>,
        range: WalLsnRange,
        targets: &[CanonicalVec<PhysicalRedoTargetClaim>],
        projection: &PersistedPhysicalRecoveryProjection,
    ) -> Self {
        let nonempty = NonEmpty::try_from_vec(records)
            .expect("durable mutation preparation rejects an empty record batch");
        assert_eq!(
            nonempty.len(),
            targets.len(),
            "the data plan binds every and only every canonical redo record"
        );
        let ordered = nonempty
            .into_vec()
            .into_iter()
            .enumerate()
            .map(|(ordinal, bytes)| {
                let ordinal = u32::try_from(ordinal)
                    .expect("record preparation bounds the batch by u16::MAX");
                let lsn = LogSequenceNumber::new(
                    range
                        .start()
                        .get()
                        .checked_add(u64::from(ordinal))
                        .expect("reserved nonempty redo ranges cannot overflow internally"),
                );
                assert!(
                    range.contains(lsn),
                    "the reserved range has one exact LSN per canonical redo record"
                );
                RedoRecord {
                    ordinal,
                    lsn,
                    targets: CanonicalVec::try_from_sorted(
                        targets[ordinal as usize].as_slice().to_vec(),
                    )
                    .expect("the bound data plan supplies canonical nonempty targets"),
                    bytes,
                    source_copy: None,
                }
            })
            .collect::<Vec<_>>();
        let records = CanonicalVec::try_from_sorted(ordered)
            .expect("monotonic ordinals establish canonical owner order");
        let encoded = encode(records.as_slice(), projection);
        let digest = Sha256::digest(&encoded).into();
        Self {
            records,
            encoded,
            digest,
        }
    }

    pub fn records(&self) -> &[RedoRecord] {
        self.records.as_slice()
    }

    /// Root publication adopts the distinct source-copy recipe. The record's
    /// frame target observation is empty because data belongs to intent LSN;
    /// recovery is governed by this explicitly tagged recipe, not that list.
    pub(in crate::physical_runtime) fn from_source_copy(
        range: WalLsnRange,
        projection: &PersistedPhysicalRecoveryProjection,
    ) -> Self {
        let worth_store_physical_format::PersistedPhysicalRecoveryPayload::SourceCopy(recipe) =
            projection.payload()
        else {
            unreachable!("copy redo requires a typed source-copy projection")
        };
        let record = RedoRecord {
            ordinal: 0,
            lsn: range.start(),
            targets: CanonicalVec::try_from_sorted(Vec::new())
                .expect("empty frame observation is canonical"),
            bytes: vec![0],
            source_copy: Some(*recipe),
        };
        let mut encoded = Vec::new();
        write_field(&mut encoded, b"store.physical.extent-copy-publication.v1");
        encoded.extend_from_slice(&range.start().get().to_le_bytes());
        write_field(&mut encoded, &projection.encode());
        Self {
            records: CanonicalVec::try_from_sorted(vec![record])
                .expect("one copy record is canonical"),
            digest: Sha256::digest(&encoded).into(),
            encoded,
        }
    }

    pub fn encoded(&self) -> &[u8] {
        &self.encoded
    }

    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    pub(in crate::physical_runtime) fn with_encoded_payload(mut self, encoded: Vec<u8>) -> Self {
        self.digest = Sha256::digest(&encoded).into();
        self.encoded = encoded;
        self
    }

    pub(in crate::physical_runtime) fn into_prepared_record_bytes(self) -> Vec<Vec<u8>> {
        self.records
            .into_parts()
            .0
            .into_iter()
            .map(|record| record.bytes)
            .collect()
    }
}

impl RedoRecord {
    pub const fn source_copy_recipe(
        &self,
    ) -> Option<worth_store_physical_format::PersistedExtentCopyRecipe> {
        self.source_copy
    }
    pub const fn ordinal(&self) -> u32 {
        self.ordinal
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub const fn lsn(&self) -> LogSequenceNumber {
        self.lsn
    }

    pub fn targets(&self) -> &[PhysicalRedoTargetClaim] {
        self.targets.as_slice()
    }
}

fn encode(records: &[RedoRecord], projection: &PersistedPhysicalRecoveryProjection) -> Vec<u8> {
    let projection_bytes = projection.encode();
    let mut capacity = field_len(REDO_DOMAIN.len())
        .and_then(|length| length.checked_add(8))
        .expect("admitted redo domain and record count fit an addressable buffer");
    for record in records {
        capacity = capacity
            .checked_add(4 + 8 + 8)
            .and_then(|length| length.checked_add(field_len(record.bytes.len())?))
            .expect("admitted redo record bytes fit an addressable buffer");
        for claim in record.targets.as_slice() {
            let mut target = Vec::with_capacity(96);
            claim.target().write_canonical(&mut target);
            capacity = capacity
                .checked_add(field_len(target.len()).expect("canonical target length fits"))
                .and_then(|length| length.checked_add(32))
                .expect("admitted redo target bytes fit an addressable buffer");
        }
    }
    capacity = capacity
        .checked_add(field_len(projection_bytes.len()).expect("projection length fits"))
        .expect("admitted recovery projection fits an addressable buffer");
    let mut encoded = Vec::with_capacity(capacity);
    write_field(&mut encoded, REDO_DOMAIN);
    encoded.extend_from_slice(&(records.len() as u64).to_le_bytes());
    for record in records {
        encoded.extend_from_slice(&record.ordinal.to_le_bytes());
        encoded.extend_from_slice(&record.lsn.get().to_le_bytes());
        encoded.extend_from_slice(&(record.targets.as_slice().len() as u64).to_le_bytes());
        for claim in record.targets.as_slice() {
            let mut target = Vec::with_capacity(32);
            claim.target().write_canonical(&mut target);
            write_field(&mut encoded, &target);
            encoded.extend_from_slice(&claim.resulting_payload_digest());
        }
        write_field(&mut encoded, &record.bytes);
    }
    write_field(&mut encoded, &projection_bytes);
    debug_assert_eq!(encoded.len(), capacity);
    encoded
}

fn field_len(payload_len: usize) -> Option<usize> {
    8_usize.checked_add(payload_len)
}

fn write_field(target: &mut Vec<u8>, bytes: &[u8]) {
    target.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    target.extend_from_slice(bytes);
}

#[cfg(test)]
mod tests {
    use super::{encode, field_len, write_field, RedoRecord, REDO_DOMAIN};
    use crate::physical_runtime::durability::{PhysicalDataFrameIdentity, PhysicalRedoTargetClaim};
    use worth_proof::CanonicalVec;
    use worth_store_physical_format::{
        CurrentPhysicalRecordPlacement, DurableExtentRecordPlacement, ExtentArenaId,
        ExtentArenaRange, ExtentChunkCoordinate, PersistedPhysicalDataFrameSubject,
        PersistedPhysicalRecoveryFrame, PersistedPhysicalRecoveryProjection,
        PersistedPhysicalRecoveryRootState, PersistedRecordIdentity, PhysicalExtentId,
        PhysicalGeneration, PhysicalGenerationAuthority, RecordArtifactFile, RecordFrameCoordinate,
    };
    use worth_store_wal::LogSequenceNumber;

    #[test]
    fn exact_reserved_redo_matches_previous_wire_bytes_with_target_and_large_frame() {
        let record = PersistedRecordIdentity::new([7; 16], 9).unwrap();
        let extent = PhysicalGenerationAuthority::for_canonical_physical_format()
            .record_extent_cell(PhysicalExtentId::from_raw(3).unwrap())
            .with_extent_generation(PhysicalGeneration::from_raw(4).unwrap());
        let range = ExtentArenaRange::new(ExtentArenaId::new(2).unwrap(), 0, 53_248).unwrap();
        let placement =
            DurableExtentRecordPlacement::legacy_unknown(record, extent, 1, range).unwrap();
        let coordinate =
            RecordFrameCoordinate::new(RecordArtifactFile::ExtentArena { arena: 2 }, 0, 1).unwrap();
        let chunk = ExtentChunkCoordinate::new(record, extent, 1, 0, 1).unwrap();
        let subject = PersistedPhysicalDataFrameSubject::ExtentChunk(chunk);
        let root =
            PersistedPhysicalRecoveryRootState::new(4096, 1, 32, vec![], None, None).unwrap();
        let projection = PersistedPhysicalRecoveryProjection::new(
            11,
            root,
            vec![record],
            vec![PersistedPhysicalRecoveryFrame::new(subject, coordinate, &[3]).unwrap()],
            vec![CurrentPhysicalRecordPlacement::Extent(placement)],
            vec![],
            vec![],
        )
        .unwrap();
        let target = PhysicalDataFrameIdentity::extent_chunk(
            chunk,
            RecordArtifactFile::ExtentArena { arena: 2 },
            0,
            1,
            range,
        )
        .unwrap();
        let claim = PhysicalRedoTargetClaim::new(target, [5; 32]);
        let records = [
            RedoRecord {
                ordinal: 0,
                lsn: LogSequenceNumber::new(41),
                targets: CanonicalVec::try_from_sorted(vec![claim]).unwrap(),
                bytes: vec![0xA5; 256 * 1024],
                source_copy: None,
            },
            RedoRecord {
                ordinal: 1,
                lsn: LogSequenceNumber::new(42),
                targets: CanonicalVec::try_from_sorted(Vec::new()).unwrap(),
                bytes: vec![0x5A; 256 * 1024],
                source_copy: None,
            },
        ];

        let encoded = encode(&records, &projection);
        let mut previous = Vec::new();
        write_field(&mut previous, REDO_DOMAIN);
        previous.extend_from_slice(&(records.len() as u64).to_le_bytes());
        for record in &records {
            previous.extend_from_slice(&record.ordinal.to_le_bytes());
            previous.extend_from_slice(&record.lsn.get().to_le_bytes());
            previous.extend_from_slice(&(record.targets.as_slice().len() as u64).to_le_bytes());
            for claim in record.targets.as_slice() {
                let mut canonical_target = Vec::new();
                claim.target().write_canonical(&mut canonical_target);
                write_field(&mut previous, &canonical_target);
                previous.extend_from_slice(&claim.resulting_payload_digest());
            }
            write_field(&mut previous, &record.bytes);
        }
        write_field(&mut previous, &projection.encode());
        assert_eq!(encoded, previous);
        assert_eq!(encoded.capacity(), encoded.len());
    }

    #[test]
    fn field_length_rejects_address_space_overflow() {
        assert_eq!(field_len(0), Some(8));
        assert_eq!(field_len(usize::MAX), None);
    }
}

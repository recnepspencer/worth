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
}

impl Ord for RedoRecord {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ordinal
            .cmp(&other.ordinal)
            .then_with(|| self.lsn.cmp(&other.lsn))
            .then_with(|| self.targets.as_slice().cmp(other.targets.as_slice()))
            .then_with(|| self.bytes.cmp(&other.bytes))
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
    let payload_bytes = records.iter().fold(0_usize, |total, record| {
        total
            .saturating_add(4)
            .saturating_add(8)
            .saturating_add(8)
            .saturating_add(record.targets.as_slice().len().saturating_mul(64))
            .saturating_add(8)
            .saturating_add(record.bytes.len())
    });
    let mut encoded = Vec::with_capacity(REDO_DOMAIN.len() + 16 + payload_bytes);
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
    write_field(&mut encoded, &projection.encode());
    encoded
}

fn write_field(target: &mut Vec<u8>, bytes: &[u8]) {
    target.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    target.extend_from_slice(bytes);
}

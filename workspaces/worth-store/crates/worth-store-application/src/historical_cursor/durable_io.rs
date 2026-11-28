//! Anchored-record I/O, single-basis pinned staging anchor (`"home.historical-basis"`),
//! O(1) direct pinned-root member reads (ZERO basis-wide member-table scans or
//! secondary maps on cold admission), and self-describing `WHPIN1` payloads.

use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use worth_relational::facade::history::CommitId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::storage::RelationalStorageSubject;
use worth_runtime_world::facade::HistoricalReadDenial;
use worth_store::physical_runtime::stability::durable_pins::PhysicalRootPinWrite;
use worth_store::physical_runtime::{
    AnchoredRecordMember, AnchoredRecordMemberLocator, AnchoredRecordName,
    AnchoredRecordPublication, PhysicalMutationDeadline, PhysicalRecordReader, RecordByteLimit,
    RecordReadLimits, ServingPhysicalRuntime,
};

use super::catalog::{HistoricalBasisCatalog, RetainedBranchIndex};
use super::chunk_format::{verify_member_digest, ChunkMemberId};
use super::stream::LoanedMemberReader;

pub(super) use super::durable_manifest_io::{
    consolidate_durable_chunks, load_basis_manifest_on_demand, publish_basis_with_manifest,
};

pub(super) const HISTORICAL_BASIS_ANCHOR: &str = "home.historical-bases";
pub(super) const HISTORICAL_BASIS_STAGE_ANCHOR: &str = "home.historical-basis";
const WRITE_DEADLINE_MS: u64 = 30_000;
pub(super) const MAX_CATALOG_BYTES: u64 = 16 * 1024 * 1024;
const MAX_BASIS_HEADER_BYTES: u64 = 1024;
const MAX_MEMBER_BYTES: u32 = 16 * 1024 * 1024;
const PIN_PAYLOAD_MAGIC: &[u8; 6] = b"WHPIN1";
const BASIS_HEADER_MAGIC: &[u8; 5] = b"WHBA1";
const BASIS_TOMBSTONE_HEADER: &[u8; 5] = b"WHBA0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct DecodedPinPayload {
    pub(super) commit_id: CommitId,
    pub(super) root_semantic_commitment: [u8; 32],
    pub(super) consolidation_epoch: u64,
}

pub(super) fn encode_pin_payload(cid: CommitId, comm: &[u8; 32], epoch: u64) -> Vec<u8> {
    [
        PIN_PAYLOAD_MAGIC.as_slice(),
        &cid.0.to_le_bytes(),
        comm,
        &epoch.to_le_bytes(),
    ]
    .concat()
}

pub(super) fn decode_pin_payload(bytes: &[u8]) -> Option<DecodedPinPayload> {
    if bytes.len() != 54 || &bytes[..6] != PIN_PAYLOAD_MAGIC {
        return None;
    }
    Some(DecodedPinPayload {
        commit_id: CommitId(u64::from_le_bytes(bytes[6..14].try_into().ok()?)),
        root_semantic_commitment: bytes[14..46].try_into().ok()?,
        consolidation_epoch: u64::from_le_bytes(bytes[46..54].try_into().ok()?),
    })
}

pub(super) fn encode_basis_header(cid: CommitId, epoch: u64, comm: &[u8; 32]) -> Vec<u8> {
    [
        BASIS_HEADER_MAGIC.as_slice(),
        &cid.0.to_le_bytes(),
        &epoch.to_le_bytes(),
        comm,
    ]
    .concat()
}

pub(super) struct DurableCatalogState {
    pub(super) anchor_generation: u64,
    pub(super) basis_stage_generation: u64,
    pub(super) catalog: HistoricalBasisCatalog,
    pub(super) retained_branches: RetainedBranchIndex,
    pub(super) needs_reconcile: bool,
}

impl DurableCatalogState {
    pub(super) fn hydrate_stage_generation(
        &mut self,
        serving: &ServingPhysicalRuntime,
    ) -> Result<(), HistoricalReadDenial> {
        let records = serving.anchored_records().map_err(|_| corrupt())?;
        self.basis_stage_generation = records
            .read(basis_stage_anchor_name(), MAX_BASIS_HEADER_BYTES)
            .map_err(|_| corrupt())?
            .map_or(0, |rec| rec.version().generation());
        Ok(())
    }
}

pub(super) struct DurableMemberReader {
    pinned_reader: PhysicalRecordReader,
    corrupted_overrides: Arc<RwLock<BTreeMap<ChunkMemberId, Vec<u8>>>>,
}

impl DurableMemberReader {
    pub(super) fn new(
        pinned_reader: PhysicalRecordReader,
        corrupted_overrides: Arc<RwLock<BTreeMap<ChunkMemberId, Vec<u8>>>>,
    ) -> Self {
        Self {
            pinned_reader,
            corrupted_overrides,
        }
    }
}

impl LoanedMemberReader for DurableMemberReader {
    fn read_member(
        &self,
        id: ChunkMemberId,
        locator_bytes: Option<&[u8; 64]>,
    ) -> Result<Vec<u8>, HistoricalReadDenial> {
        if let Some(bytes) = self
            .corrupted_overrides
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .get(&id)
        {
            return Ok(bytes.clone());
        }
        let raw_locator = locator_bytes.ok_or_else(corrupt)?;
        let locator = AnchoredRecordMemberLocator::from_bytes(raw_locator).ok_or_else(corrupt)?;
        if locator.sha256() != id.sha256
            || locator.length() != id.length
            || id.length > u64::from(MAX_MEMBER_BYTES)
        {
            return Err(corrupt());
        }
        let limit = RecordReadLimits::new(RecordByteLimit::new(MAX_MEMBER_BYTES).expect("nonzero"));
        let mut session = self
            .pinned_reader
            .open(locator.record(), limit)
            .map_err(|_| corrupt())?;
        let expected_len = usize::try_from(id.length).map_err(|_| corrupt())?;
        if session.observation().bytes_requested() != id.length {
            return Err(corrupt());
        }
        let mut out = vec![0u8; expected_len];
        let mut filled = 0usize;
        while filled < expected_len {
            let n = session
                .read_next(&mut out[filled..])
                .map_err(|_| corrupt())?;
            if n == 0 {
                return Err(corrupt());
            }
            filled += n;
        }
        if session.read_next(&mut [0u8; 1]).map_err(|_| corrupt())? != 0 {
            return Err(corrupt());
        }
        verify_member_digest(id, &out)?;
        Ok(out)
    }
}

fn write_deadline() -> Result<PhysicalMutationDeadline, HistoricalReadDenial> {
    PhysicalMutationDeadline::after_milliseconds(WRITE_DEADLINE_MS).ok_or_else(corrupt)
}

pub(super) fn publish_anchor(
    serving: &ServingPhysicalRuntime,
    name: AnchoredRecordName,
    generation: &mut u64,
    header: Vec<u8>,
    members: Vec<AnchoredRecordMember>,
) -> Result<Vec<AnchoredRecordMemberLocator>, HistoricalReadDenial> {
    let records = serving.anchored_records().map_err(|_| corrupt())?;
    let mut pub_req = AnchoredRecordPublication::new(
        name,
        *generation,
        header,
        crate::open::physical_admission::placement(),
        write_deadline()?,
    );
    if !members.is_empty() {
        pub_req = pub_req.with_members(members);
    }
    let published = records.publish(pub_req).map_err(|_| corrupt())?;
    *generation = published.version().generation();
    Ok(published.members().to_vec())
}

pub(super) fn publish_catalog(
    serving: &ServingPhysicalRuntime,
    state: &mut DurableCatalogState,
    next_catalog: &HistoricalBasisCatalog,
) -> Result<(), HistoricalReadDenial> {
    let _ = publish_anchor(
        serving,
        anchor_name(),
        &mut state.anchor_generation,
        next_catalog.encode(),
        Vec::new(),
    )?;
    Ok(())
}

pub(super) fn clear_basis_stage(
    serving: &ServingPhysicalRuntime,
    state: &mut DurableCatalogState,
) -> Result<(), HistoricalReadDenial> {
    if state.basis_stage_generation == 0 {
        return Ok(());
    }
    let _ = publish_anchor(
        serving,
        basis_stage_anchor_name(),
        &mut state.basis_stage_generation,
        BASIS_TOMBSTONE_HEADER.to_vec(),
        Vec::new(),
    )?;
    Ok(())
}

pub(super) fn anchor_name() -> AnchoredRecordName {
    AnchoredRecordName::new(HISTORICAL_BASIS_ANCHOR).expect("static valid anchor name")
}
pub(super) fn basis_stage_anchor_name() -> AnchoredRecordName {
    AnchoredRecordName::new(HISTORICAL_BASIS_STAGE_ANCHOR).expect("static valid stage anchor name")
}
pub(super) fn pin_write() -> Result<PhysicalRootPinWrite, HistoricalReadDenial> {
    Ok(PhysicalRootPinWrite {
        placement: crate::open::physical_admission::placement(),
        deadline: write_deadline()?,
    })
}
pub(super) fn corrupt() -> HistoricalReadDenial {
    HistoricalReadDenial::CorruptStorage {
        subject: RelationalStorageSubject::Region(PartitionId(0)),
    }
}

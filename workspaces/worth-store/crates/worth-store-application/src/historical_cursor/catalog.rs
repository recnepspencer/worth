//! Fixed-size durable retained-basis root directory (`CatalogDirectory`)
//! persisted in Store's `"home.historical-bases"` anchored record (Directory)
//! or held in memory (Memory).
//!
//! Holds ONLY fixed-size per-basis summary entries (including each basis's
//! `basis_manifest_member` + `basis_manifest_locator` pointing to its
//! on-demand `BasisManifest` member) plus crash-recoverable pending pin
//! retirements, ready-to-retire branches (`pending_retire_branches`), and
//! active pre-fork retain intents (`inflight_retain_branches`).
//!
//! ZERO chunk vectors and ZERO continuation locator vectors live here, so
//! restoring the directory on cold reopen stays O(1) in chunk/continuation count.

use std::collections::BTreeMap;
use std::sync::Arc;

use sha2::{Digest, Sha256};
use worth_relational::facade::history::{BranchId, CommitId};
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::schema::{SchemaId, SchemaVersionId};
use worth_relational::facade::storage::RelationalStorageSubject;
use worth_runtime_world::facade::{
    BranchOriginIdentity, HistoricalBasisRetentionKind, HistoricalReadDenial, SchemaVersionIdentity,
};

use super::chunk_format::ChunkMemberId;

const CATALOG_MAGIC: &[u8; 6] = b"WHCAT5";
const CATALOG_SEAL_DOMAIN: &[u8] = b"worth.historical.basis-catalog.v5";

const MIN_PIN_BYTES: usize = 8;
const MIN_STR_BYTES: usize = 4;
const MIN_BASIS_BYTES: usize = 8 + 32 + 4 + 32 + 4 + 4 + 32 + 1 + 8 + 1 + 8 + 1 + 1 + 1 + 1;

/// Fixed-size summary entry for one independently retained historical basis.
/// Chunk and continuation descriptors live in the basis's on-demand
/// `BasisManifest` member referenced by `basis_manifest_member` /
/// `basis_manifest_locator`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetainedBasisRecord {
    pub commit_id: CommitId,
    pub root_semantic_commitment: [u8; 32],
    pub branch_origin: BranchOriginIdentity,
    pub schema_version: SchemaVersionIdentity,
    pub retention_kind: HistoricalBasisRetentionKind,
    /// `0` iff staged in step R1 before Store pin acquisition (step R2).
    pub pin_ordinal: u64,
    /// `Some(old_pin)` during two-phase consolidation until `new_pin` binds.
    pub previous_pin_ordinal: Option<u64>,
    pub consolidation_epoch: u64,
    /// Real durable Relational/Query branch name when backed by a live/archived branch.
    pub retained_branch_name: Option<String>,
    /// Exact pinned Relational truth snapshot version for Query branch bases.
    pub retained_snapshot_version: Option<u64>,
    /// Content-addressed identity of this basis's on-demand `BasisManifest` member.
    pub basis_manifest_member: Option<ChunkMemberId>,
    /// Direct physical locator of this basis's on-demand `BasisManifest` member.
    pub basis_manifest_locator: Option<[u8; 64]>,
}

/// Root directory of all independently retained bases in one Home.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoricalBasisCatalog {
    pub mac_key: [u8; 32],
    pub pending_retire_pins: Vec<u64>,
    /// Branches explicitly retired/abandoned and eligible for `close_branch`.
    pub pending_retire_branches: Vec<String>,
    /// Pre-fork retain intents currently in flight in this incarnation; NEVER
    /// reaped by live reconciliation, promoted to `pending_retire_branches`
    /// ONLY on cold reopen after a crash.
    pub inflight_retain_branches: Vec<String>,
    pub bases: BTreeMap<CommitId, Arc<RetainedBasisRecord>>,
}

/// O(log N) branch-name reference-count index maintained alongside `bases`
/// under the catalog mutex so retirement reconciliation never scans `bases`.
#[derive(Debug, Default, Clone)]
pub(super) struct RetainedBranchIndex {
    counts: BTreeMap<String, usize>,
}

impl RetainedBranchIndex {
    pub(super) fn from_catalog(catalog: &HistoricalBasisCatalog) -> Self {
        let mut index = Self::default();
        index.rebuild(catalog);
        index
    }

    pub(super) fn rebuild(&mut self, catalog: &HistoricalBasisCatalog) {
        self.counts.clear();
        for entry in catalog.bases.values() {
            self.increment(entry.retained_branch_name.as_deref());
        }
    }

    pub(super) fn increment(&mut self, branch_name: Option<&str>) {
        if let Some(name) = branch_name {
            *self.counts.entry(name.to_owned()).or_insert(0) += 1;
        }
    }

    pub(super) fn decrement(&mut self, branch_name: Option<&str>) {
        if let Some(name) = branch_name
            && let Some(count) = self.counts.get_mut(name)
        {
            if *count <= 1 {
                self.counts.remove(name);
            } else {
                *count -= 1;
            }
        }
    }

    pub(super) fn contains(&self, branch_name: &str) -> bool {
        self.counts.get(branch_name).copied().unwrap_or(0) > 0
    }
}

impl HistoricalBasisCatalog {
    pub fn fresh(mac_key: [u8; 32]) -> Self {
        Self {
            mac_key,
            pending_retire_pins: Vec::new(),
            pending_retire_branches: Vec::new(),
            inflight_retain_branches: Vec::new(),
            bases: BTreeMap::new(),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut body = Vec::new();
        body.extend_from_slice(CATALOG_MAGIC);
        body.extend_from_slice(&self.mac_key);
        body.extend_from_slice(&(self.pending_retire_pins.len() as u32).to_le_bytes());
        for pin in &self.pending_retire_pins {
            body.extend_from_slice(&pin.to_le_bytes());
        }
        body.extend_from_slice(&(self.pending_retire_branches.len() as u32).to_le_bytes());
        for branch in &self.pending_retire_branches {
            write_str(&mut body, branch);
        }
        body.extend_from_slice(&(self.inflight_retain_branches.len() as u32).to_le_bytes());
        for branch in &self.inflight_retain_branches {
            write_str(&mut body, branch);
        }
        body.extend_from_slice(&(self.bases.len() as u32).to_le_bytes());
        for record in self.bases.values() {
            encode_basis_record(&mut body, record);
        }
        let seal = catalog_seal(&body);
        body.extend_from_slice(&seal);
        body
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, HistoricalReadDenial> {
        if bytes.len() < CATALOG_MAGIC.len() + 32 + 32 {
            return Err(corrupt());
        }
        let (body, tag) = bytes.split_at(bytes.len() - 32);
        if tag != catalog_seal(body) {
            return Err(corrupt());
        }
        let mut r = Reader { bytes: body, at: 0 };
        if r.take(6)? != CATALOG_MAGIC {
            return Err(corrupt());
        }
        let mac_key = r.array::<32>()?;
        let pending_count = r.bounded_count(MIN_PIN_BYTES)?;
        let mut pending_retire_pins = Vec::with_capacity(pending_count);
        for _ in 0..pending_count {
            pending_retire_pins.push(r.u64()?);
        }
        let branch_count = r.bounded_count(MIN_STR_BYTES)?;
        let mut pending_retire_branches = Vec::with_capacity(branch_count);
        for _ in 0..branch_count {
            pending_retire_branches.push(r.string()?);
        }
        let inflight_count = r.bounded_count(MIN_STR_BYTES)?;
        let mut inflight_retain_branches = Vec::with_capacity(inflight_count);
        for _ in 0..inflight_count {
            inflight_retain_branches.push(r.string()?);
        }
        let count = r.bounded_count(MIN_BASIS_BYTES)?;
        let mut bases = BTreeMap::new();
        for _ in 0..count {
            let record = decode_basis_record(&mut r)?;
            bases.insert(record.commit_id, Arc::new(record));
        }
        if r.at != body.len() {
            return Err(corrupt());
        }
        Ok(Self {
            mac_key,
            pending_retire_pins,
            pending_retire_branches,
            inflight_retain_branches,
            bases,
        })
    }
}

fn encode_basis_record(body: &mut Vec<u8>, record: &RetainedBasisRecord) {
    body.extend_from_slice(&record.commit_id.0.to_le_bytes());
    body.extend_from_slice(&record.root_semantic_commitment);
    write_str(body, &record.branch_origin.branch_id().0);
    body.extend_from_slice(&record.branch_origin.origin_digest());
    write_str(body, &record.schema_version.schema_id().0);
    body.extend_from_slice(&record.schema_version.schema_version_id().0.to_le_bytes());
    body.extend_from_slice(&record.schema_version.schema_commitment());
    body.push(retention_tag(record.retention_kind));
    body.extend_from_slice(&record.pin_ordinal.to_le_bytes());
    write_opt_bytes(
        body,
        record
            .previous_pin_ordinal
            .map(u64::to_le_bytes)
            .as_ref()
            .map(|b| b.as_slice()),
    );
    body.extend_from_slice(&record.consolidation_epoch.to_le_bytes());
    match &record.retained_branch_name {
        Some(name) => {
            body.push(1);
            write_str(body, name);
        }
        None => body.push(0),
    }
    write_opt_bytes(
        body,
        record
            .retained_snapshot_version
            .map(u64::to_le_bytes)
            .as_ref()
            .map(|b| b.as_slice()),
    );
    match record.basis_manifest_member {
        Some(member) => {
            body.push(1);
            body.extend_from_slice(&member.sha256);
            body.extend_from_slice(&member.length.to_le_bytes());
        }
        None => body.push(0),
    }
    write_opt_bytes(
        body,
        record.basis_manifest_locator.as_ref().map(|l| l.as_slice()),
    );
}

fn decode_basis_record(r: &mut Reader<'_>) -> Result<RetainedBasisRecord, HistoricalReadDenial> {
    let commit_id = CommitId(r.u64()?);
    let root_semantic_commitment = r.array::<32>()?;
    let branch_id = BranchId(r.string()?);
    let branch_origin_digest = r.array::<32>()?;
    let branch_origin =
        BranchOriginIdentity::from_owner(branch_id, branch_origin_digest).ok_or_else(corrupt)?;
    let schema_id = SchemaId(r.string()?);
    let schema_version_id = SchemaVersionId(r.u32()?);
    let schema_commitment = r.array::<32>()?;
    let schema_version =
        SchemaVersionIdentity::new(schema_id, schema_version_id, schema_commitment)
            .ok_or_else(corrupt)?;
    let retention_kind = parse_retention_tag(r.u8()?).ok_or_else(corrupt)?;
    let pin_ordinal = r.u64()?;
    let previous_pin_ordinal = read_opt_u64(r)?;
    let consolidation_epoch = r.u64()?;
    let retained_branch_name = match r.u8()? {
        0 => None,
        1 => Some(r.string()?),
        _ => return Err(corrupt()),
    };
    let retained_snapshot_version = read_opt_u64(r)?;
    let basis_manifest_member = match r.u8()? {
        0 => None,
        1 => Some(ChunkMemberId {
            sha256: r.array::<32>()?,
            length: r.u64()?,
        }),
        _ => return Err(corrupt()),
    };
    let basis_manifest_locator = match r.u8()? {
        0 => None,
        1 => Some(r.array::<64>()?),
        _ => return Err(corrupt()),
    };
    Ok(RetainedBasisRecord {
        commit_id,
        root_semantic_commitment,
        branch_origin,
        schema_version,
        retention_kind,
        pin_ordinal,
        previous_pin_ordinal,
        consolidation_epoch,
        retained_branch_name,
        retained_snapshot_version,
        basis_manifest_member,
        basis_manifest_locator,
    })
}

fn catalog_seal(body: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(CATALOG_SEAL_DOMAIN);
    h.update(body);
    h.finalize().into()
}

fn corrupt() -> HistoricalReadDenial {
    HistoricalReadDenial::CorruptStorage {
        subject: RelationalStorageSubject::Region(PartitionId(0)),
    }
}

const fn retention_tag(kind: HistoricalBasisRetentionKind) -> u8 {
    match kind {
        HistoricalBasisRetentionKind::LiveBranch => 0,
        HistoricalBasisRetentionKind::ArchivedBranch => 1,
        HistoricalBasisRetentionKind::PinLease => 2,
        HistoricalBasisRetentionKind::RetentionHorizon => 3,
    }
}

const fn parse_retention_tag(tag: u8) -> Option<HistoricalBasisRetentionKind> {
    Some(match tag {
        0 => HistoricalBasisRetentionKind::LiveBranch,
        1 => HistoricalBasisRetentionKind::ArchivedBranch,
        2 => HistoricalBasisRetentionKind::PinLease,
        3 => HistoricalBasisRetentionKind::RetentionHorizon,
        _ => return None,
    })
}

fn write_opt_bytes(out: &mut Vec<u8>, bytes: Option<&[u8]>) {
    match bytes {
        Some(b) => {
            out.push(1);
            out.extend_from_slice(b);
        }
        None => out.push(0),
    }
}

fn read_opt_u64(r: &mut Reader<'_>) -> Result<Option<u64>, HistoricalReadDenial> {
    match r.u8()? {
        0 => Ok(None),
        1 => Ok(Some(r.u64()?)),
        _ => Err(corrupt()),
    }
}

fn write_str(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(&(s.len() as u32).to_le_bytes());
    out.extend_from_slice(s.as_bytes());
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn bounded_count(&mut self, min_elem: usize) -> Result<usize, HistoricalReadDenial> {
        let count = self.u32()? as usize;
        if min_elem > 0 && count > self.bytes.len().saturating_sub(self.at) / min_elem {
            return Err(corrupt());
        }
        Ok(count)
    }
    fn take(&mut self, len: usize) -> Result<&'a [u8], HistoricalReadDenial> {
        let end = self.at.checked_add(len).ok_or_else(corrupt)?;
        let slice = self.bytes.get(self.at..end).ok_or_else(corrupt)?;
        self.at = end;
        Ok(slice)
    }
    fn u8(&mut self) -> Result<u8, HistoricalReadDenial> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> Result<u32, HistoricalReadDenial> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().expect("4")))
    }
    fn u64(&mut self) -> Result<u64, HistoricalReadDenial> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().expect("8")))
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N], HistoricalReadDenial> {
        Ok(self.take(N)?.try_into().expect("N"))
    }
    fn string(&mut self) -> Result<String, HistoricalReadDenial> {
        let len = self.u32()? as usize;
        std::str::from_utf8(self.take(len)?)
            .map(str::to_owned)
            .map_err(|_| corrupt())
    }
}

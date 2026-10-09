use std::collections::BTreeMap;

use worth_store_physical_format::{
    BlobSessionDeclarationV1, PersistedRecordIdentity, SelectedRecordContentClass,
};

use crate::physical_runtime::blob::ingest::SelectedResumeClaim;
use crate::physical_runtime::{
    PhysicalRecordId, PhysicalRecordReader, RecordByteLimit, RecordCountLimit, RecordReadLimits,
    RecordScanOutcome, RecordScanRequest,
};

use super::{BlobReachabilityFailure as Failure, BlobReachabilityLimits};

pub(super) mod closure;
pub(super) mod control;
mod decode;
pub(super) mod reuse;

use decode::decode_selected;

const FRAME_WINDOW: usize = 1024 * 1024;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Role {
    Opaque,
    Declaration([u8; 16]),
    Chunk {
        session: [u8; 16],
        ordinal: u64,
    },
    Tree([u8; 16]),
    Frontier {
        session: [u8; 16],
        next_ordinal: u64,
    },
    ReuseClaim {
        session: [u8; 16],
        ordinal: u64,
    },
    Abandonment([u8; 16]),
    Publication([u8; 16]),
    ReleasedControl([u8; 16]),
    Control,
    Derived,
}

pub(super) struct Fact {
    pub(super) current: bool,
    pub(super) held: bool,
    pub(super) role: Role,
    pub(super) edges: Vec<PersistedRecordIdentity>,
    pub(super) declaration: Option<BlobSessionDeclarationV1>,
    pub(super) claim: Option<SelectedResumeClaim>,
    pub(super) closure: Option<closure::ClosureFact>,
    pub(super) control: Option<control::ControlFact>,
    pub(super) reuse: Option<reuse::ReuseSource>,
}

pub(super) struct SelectedInventory {
    pub(super) facts: BTreeMap<PersistedRecordIdentity, Fact>,
    pub(super) current_derived_directory: Option<PersistedRecordIdentity>,
    pub(super) selected_records_scanned: u64,
    pub(super) bytes_inspected: u64,
    pub(super) edge_count: u64,
    limits: BlobReachabilityLimits,
}

impl SelectedInventory {
    pub(super) const fn maximum_edges(&self) -> u64 {
        self.limits.maximum_edges.get()
    }
    pub(super) fn new(limits: BlobReachabilityLimits) -> Result<Self, Failure> {
        Ok(Self {
            facts: BTreeMap::new(),
            current_derived_directory: None,
            selected_records_scanned: 0,
            bytes_inspected: 0,
            edge_count: 0,
            limits,
        })
    }

    pub(super) fn scan_root(
        &mut self,
        reader: PhysicalRecordReader,
        current: bool,
    ) -> Result<(), Failure> {
        if current {
            self.current_derived_directory = reader
                .selected_derived_family_directory()
                .map(|binding| binding.directory_record());
        }
        let scratch_len = usize::try_from(self.limits.maximum_inspected_bytes.get())
            .unwrap_or(usize::MAX)
            .min(FRAME_WINDOW)
            .max(8);
        let mut scratch = Vec::new();
        scratch
            .try_reserve_exact(scratch_len)
            .map_err(|_| Failure::MetadataUnavailable)?;
        scratch.resize(scratch_len, 0);
        let mut scan = reader
            .into_rebuild()
            .scan_rebuild(
                RecordScanRequest::from_start()
                    .with_batch_limit(RecordCountLimit::new(1).expect("one selected route"))
                    .with_payload_limit(
                        RecordByteLimit::new(scratch_len as u32).expect("frame window"),
                    ),
            )
            .map_err(Failure::Scan)?;
        loop {
            if self.selected_records_scanned == self.limits.maximum_selected_records.get() {
                return Err(Failure::SelectedBoundExhausted);
            }
            let (record, class, declared, mut payload, complete) = {
                let batch = match scan.read_next_into(&mut scratch).map_err(Failure::Scan)? {
                    RecordScanOutcome::Completed(_) => break,
                    RecordScanOutcome::Batch(batch) => batch,
                };
                let row = batch
                    .records()
                    .first()
                    .ok_or(Failure::ConflictingSelectedFate)?;
                let record = PersistedRecordIdentity::new(
                    row.record_id().allocation_epoch(),
                    row.record_id().ordinal(),
                )
                .ok_or(Failure::ConflictingSelectedFate)?;
                let payload = if let Some(bytes) = batch.payload(0) {
                    let mut copy = Vec::new();
                    copy.try_reserve_exact(bytes.len())
                        .map_err(|_| Failure::MetadataUnavailable)?;
                    copy.extend_from_slice(bytes);
                    Some(copy)
                } else {
                    None
                };
                (
                    record,
                    row.content_class(),
                    row.declared_payload_bytes(),
                    payload,
                    batch.is_complete(),
                )
            };
            self.selected_records_scanned = self
                .selected_records_scanned
                .checked_add(1)
                .ok_or(Failure::SelectedBoundExhausted)?;
            if payload.is_none()
                && matches!(
                    class,
                    SelectedRecordContentClass::Blob(_)
                        | SelectedRecordContentClass::DerivedDirectory
                        | SelectedRecordContentClass::BTreeNode { .. }
                )
            {
                if declared > scratch_len as u64 {
                    return Err(Failure::InspectionWindowExhausted);
                }
                payload = Some(read_selected(scan.protected_reader(), record, declared)?);
            }
            if payload.is_none() && class == SelectedRecordContentClass::Opaque && declared >= 8 {
                payload = Some(read_prefix(scan.protected_reader(), record, declared, 8)?);
            }
            let payload = payload.as_deref().unwrap_or(&[]);
            self.bytes_inspected = self
                .bytes_inspected
                .checked_add(payload.len() as u64)
                .filter(|bytes| *bytes <= self.limits.maximum_inspected_bytes.get())
                .ok_or(Failure::InspectedByteBoundExhausted)?;
            let (role, edges, declaration, claim, closure, control, reuse) =
                decode_selected(class, payload, record)?;
            self.edge_count = self
                .edge_count
                .checked_add(edges.len() as u64)
                .filter(|count| *count <= self.limits.maximum_edges.get())
                .ok_or(Failure::EdgeBoundExhausted)?;
            let inventory_full =
                self.facts.len() as u64 == self.limits.maximum_selected_records.get();
            match self.facts.entry(record) {
                std::collections::btree_map::Entry::Occupied(mut occupied) => {
                    let fact = occupied.get_mut();
                    if fact.role != role
                        || fact.edges != edges
                        || fact.declaration != declaration
                        || fact.claim != claim
                        || fact.closure != closure
                        || fact.control != control
                        || fact.reuse != reuse
                    {
                        return Err(Failure::ConflictingSelectedFate);
                    }
                    fact.current |= current;
                    fact.held |= !current;
                }
                std::collections::btree_map::Entry::Vacant(vacant) => {
                    if inventory_full {
                        return Err(Failure::SelectedBoundExhausted);
                    }
                    vacant.insert(Fact {
                        current,
                        held: !current,
                        role,
                        edges,
                        declaration,
                        claim,
                        closure,
                        control,
                        reuse,
                    });
                }
            }
            if complete {
                break;
            }
        }
        Ok(())
    }
}

fn read_selected(
    reader: &PhysicalRecordReader,
    record: PersistedRecordIdentity,
    declared: u64,
) -> Result<Vec<u8>, Failure> {
    let len = usize::try_from(declared).map_err(|_| Failure::InspectionWindowExhausted)?;
    let limit = RecordByteLimit::new(
        u32::try_from(declared).map_err(|_| Failure::InspectionWindowExhausted)?,
    )
    .ok_or(Failure::InspectionWindowExhausted)?;
    let mut stream = reader
        .open(
            PhysicalRecordId::from_persisted(record),
            RecordReadLimits::new(limit),
        )
        .map_err(Failure::Read)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(len)
        .map_err(|_| Failure::MetadataUnavailable)?;
    bytes.resize(len, 0);
    let mut used = 0;
    while used < len {
        let count = stream
            .read_next(&mut bytes[used..])
            .map_err(Failure::Stream)?;
        if count == 0 {
            return Err(Failure::ConflictingSelectedFate);
        }
        used += count;
    }
    let mut extra = [0; 1];
    if stream.read_next(&mut extra).map_err(Failure::Stream)? != 0 {
        return Err(Failure::ConflictingSelectedFate);
    }
    Ok(bytes)
}

fn read_prefix(
    reader: &PhysicalRecordReader,
    record: PersistedRecordIdentity,
    declared: u64,
    prefix: usize,
) -> Result<Vec<u8>, Failure> {
    let limit = RecordByteLimit::new(
        u32::try_from(declared).map_err(|_| Failure::InspectionWindowExhausted)?,
    )
    .ok_or(Failure::InspectionWindowExhausted)?;
    let mut stream = reader
        .open(
            PhysicalRecordId::from_persisted(record),
            RecordReadLimits::new(limit),
        )
        .map_err(Failure::Read)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(prefix)
        .map_err(|_| Failure::MetadataUnavailable)?;
    bytes.resize(prefix, 0);
    let mut used = 0;
    while used < prefix {
        let count = stream
            .read_next(&mut bytes[used..])
            .map_err(Failure::Stream)?;
        if count == 0 {
            return Err(Failure::ConflictingSelectedFate);
        }
        used += count;
    }
    Ok(bytes)
}

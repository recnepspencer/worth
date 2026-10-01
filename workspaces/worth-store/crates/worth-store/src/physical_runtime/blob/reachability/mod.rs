//! Read-only classification from Store-owned protected roots and selected routes.

mod graph;
mod scan;

use std::num::NonZeroU64;

use worth_store_physical_format::PersistedRecordIdentity;

use crate::physical_runtime::{PhysicalReadProtectionDenial, ServingPhysicalRuntime};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobRecordReachability {
    Reachable,
    HeldOnly,
    FailedOperationResidue,
    DerivedResidue,
    Unreferenced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobReachabilityRecord {
    record: PersistedRecordIdentity,
    class: BlobRecordReachability,
}

impl BlobReachabilityRecord {
    pub const fn record(self) -> PersistedRecordIdentity {
        self.record
    }
    pub const fn class(self) -> BlobRecordReachability {
        self.class
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobReachabilityLimits {
    maximum_selected_records: NonZeroU64,
    maximum_inspected_bytes: NonZeroU64,
    maximum_retained_roots: NonZeroU64,
    maximum_edges: NonZeroU64,
}

impl BlobReachabilityLimits {
    pub const fn new(
        maximum_selected_records: NonZeroU64,
        maximum_inspected_bytes: NonZeroU64,
        maximum_retained_roots: NonZeroU64,
        maximum_edges: NonZeroU64,
    ) -> Self {
        Self {
            maximum_selected_records,
            maximum_inspected_bytes,
            maximum_retained_roots,
            maximum_edges,
        }
    }
}

#[derive(Debug)]
pub enum BlobReachabilityFailure {
    RootProtection(PhysicalReadProtectionDenial),
    Scan(crate::physical_runtime::RecordScanError),
    Read(crate::physical_runtime::RecordReadError),
    Stream(crate::physical_runtime::RecordStreamFailure),
    Format(worth_store_physical_format::BlobRecordDenial),
    DerivedDirectory(worth_store_physical_format::DerivedFamilyDirectoryDenial),
    BTree(worth_store_physical_format::BTreeNodeDenial),
    SelectedBoundExhausted,
    InspectedByteBoundExhausted,
    EdgeBoundExhausted,
    InspectionWindowExhausted,
    ConflictingSelectedFate,
    MetadataUnavailable,
}

pub struct BlobReachabilityObservation {
    records: Vec<BlobReachabilityRecord>,
    selected_records_scanned: u64,
    bytes_inspected: u64,
    retained_roots_scanned: u64,
    edges_traversed: u64,
}

impl BlobReachabilityObservation {
    pub fn records(&self) -> &[BlobReachabilityRecord] {
        &self.records
    }
    pub const fn selected_records_scanned(&self) -> u64 {
        self.selected_records_scanned
    }
    pub const fn bytes_inspected(&self) -> u64 {
        self.bytes_inspected
    }
    pub const fn retained_roots_scanned(&self) -> u64 {
        self.retained_roots_scanned
    }
    pub const fn edges_traversed(&self) -> u64 {
        self.edges_traversed
    }
}

pub(super) fn classify(
    runtime: &ServingPhysicalRuntime,
    limits: BlobReachabilityLimits,
) -> Result<BlobReachabilityObservation, BlobReachabilityFailure> {
    let current = runtime
        .records()
        .map_err(BlobReachabilityFailure::RootProtection)?;
    let roots = usize::try_from(limits.maximum_retained_roots.get())
        .map_err(|_| BlobReachabilityFailure::MetadataUnavailable)?;
    let retained = runtime
        .retained_record_readers(
            roots
                .checked_add(1)
                .ok_or(BlobReachabilityFailure::MetadataUnavailable)?,
        )
        .map_err(BlobReachabilityFailure::RootProtection)?;
    let recovery = runtime
        .recovery_retained_record_reader()
        .map_err(BlobReachabilityFailure::RootProtection)?;
    let current_root = current.protected_root().root();
    let retained_cells = retained
        .iter()
        .map(|reader| reader.protected_root().root())
        .collect::<Vec<_>>();
    let recovery_is_distinct = recovery.as_ref().is_some_and(|reader| {
        let root = reader.protected_root().root();
        root != current_root && !retained_cells.contains(&root)
    });
    let distinct_retained = retained_cells
        .iter()
        .filter(|root| **root != current_root)
        .count();
    if distinct_retained + usize::from(recovery_is_distinct) > roots {
        return Err(BlobReachabilityFailure::RootProtection(
            PhysicalReadProtectionDenial::RetainedRootLimit,
        ));
    }
    let mut inventory = scan::SelectedInventory::new(limits)?;
    inventory.scan_root(current, true)?;
    let mut retained_roots_scanned = 0;
    for reader in retained {
        if reader.protected_root().root() == current_root {
            continue;
        }
        inventory.scan_root(reader, false)?;
        retained_roots_scanned += 1;
    }
    if let Some(reader) = recovery {
        if recovery_is_distinct {
            inventory.scan_root(reader, false)?;
            retained_roots_scanned += 1;
        }
    }
    let (records, edges_traversed) = graph::classify(&inventory)?;
    Ok(BlobReachabilityObservation {
        records,
        selected_records_scanned: inventory.selected_records_scanned,
        bytes_inspected: inventory.bytes_inspected,
        retained_roots_scanned,
        edges_traversed,
    })
}

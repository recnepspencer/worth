//! Advances a carried manifest roster to the selected root by its routing
//! delta: `O(touched)` routing blocks and records instead of every route.
//!
//! Equivalence contract (arch law 17): identity basis is the carried root's
//! cell and routing tree identity; dependency order is ascending record
//! identity; the comparator is the full rebuild scan's roster, pin for pin and
//! in order; invalidation is any routing block written after the carried
//! root's generation, which this walk re-reads.

use std::collections::HashSet;

use worth_store_physical_format::{
    ManifestBlockReference, PersistedRecordIdentity, PhysicalRootRoutingBlock,
};

use super::carried::{
    carried_in_range, walk_changed_since, CarriedManifestRoster, RoutingDeltaVisitor, RoutingNode,
    RoutingSource,
};
use super::{
    is_manifest_kind, RosterLimits, SelectedBlobManifestPin, SelectedBlobManifestPinDenial,
};
use crate::physical_runtime::record_serving::access::manifest_routing::{
    ManifestDiscoveryCounterSnapshot, ManifestReader,
};
use crate::physical_runtime::record_serving::{
    PhysicalRecordId, PhysicalRecordReader, RecordByteLimit, RecordReadLimits,
};

const MANIFEST_PREFIX_BYTES: usize = 9;

pub(super) fn delta_scan(
    reader: PhysicalRecordReader,
    carried: &CarriedManifestRoster,
    limits: RosterLimits,
    scratch: &mut [u8],
) -> Result<Vec<SelectedBlobManifestPin>, SelectedBlobManifestPinDenial> {
    if carried.root == reader.current_root.root_cell() {
        return carried_roster_unchanged(carried, limits);
    }
    let manifest = ManifestReader::serving(
        reader.residency.clone(),
        reader.format,
        reader.access,
        reader.current_root.clone(),
    );
    let page_bytes = u64::from(reader.format.declaration().page_size().bytes());
    let allocation = reader
        .residency
        .begin_operation(
            reader.residency.read_allocation_scope(),
            std::num::NonZeroU64::new(page_bytes).expect("an admitted page is nonzero"),
        )
        .map_err(|_| SelectedBlobManifestPinDenial::ScanUnavailable)?;
    let mut source = SelectedRoutingSource {
        manifest: &manifest,
        allocation: &allocation,
        counters: ManifestDiscoveryCounterSnapshot::default(),
    };
    let mut roster = DeltaRoster {
        carried: &carried.pins,
        reader: &reader,
        limits,
        scratch,
        pins: Vec::new(),
        attempts: HashSet::new(),
    };
    walk_changed_since(
        &mut source,
        reader.current_root.routing_root(),
        carried.root.generation().get(),
        &mut roster,
    )?;
    Ok(roster.pins)
}

/// The selected root is the carried root itself: its roster is re-admitted
/// under this selection's bound without reading any route.
fn carried_roster_unchanged(
    carried: &CarriedManifestRoster,
    limits: RosterLimits,
) -> Result<Vec<SelectedBlobManifestPin>, SelectedBlobManifestPinDenial> {
    let mut pins = Vec::new();
    let mut attempts = HashSet::new();
    for pin in &carried.pins {
        limits.admit_carried(*pin, &mut pins, &mut attempts)?;
    }
    Ok(pins)
}

struct SelectedRoutingSource<'walk> {
    manifest: &'walk ManifestReader<'static>,
    allocation: &'walk worth_store_buffer_pool::OperationAllocationGrant,
    counters: ManifestDiscoveryCounterSnapshot,
}

impl RoutingSource for SelectedRoutingSource<'_> {
    type Ref = ManifestBlockReference;
    type Key = PersistedRecordIdentity;
    type Error = SelectedBlobManifestPinDenial;

    fn generation(&self, reference: ManifestBlockReference) -> u64 {
        reference.generation()
    }

    fn range(
        &self,
        reference: ManifestBlockReference,
    ) -> (PersistedRecordIdentity, PersistedRecordIdentity) {
        (reference.first(), reference.last())
    }

    fn read(
        &mut self,
        reference: ManifestBlockReference,
    ) -> Result<RoutingNode<ManifestBlockReference, PersistedRecordIdentity>, Self::Error> {
        let block = self
            .manifest
            .read_block(self.allocation, reference, &mut self.counters)
            .map_err(|_| SelectedBlobManifestPinDenial::ScanUnavailable)?;
        Ok(match block {
            PhysicalRootRoutingBlock::Leaf { entries, .. } => {
                RoutingNode::Leaf(entries.iter().map(|entry| entry.record()).collect())
            }
            PhysicalRootRoutingBlock::Branch { children, .. } => RoutingNode::Branch(children),
        })
    }
}

/// Merges carried pins of unchanged subtrees with manifests re-observed in
/// rewritten leaves, in ascending record order.
struct DeltaRoster<'walk> {
    carried: &'walk [SelectedBlobManifestPin],
    reader: &'walk PhysicalRecordReader,
    limits: RosterLimits,
    scratch: &'walk mut [u8],
    pins: Vec<SelectedBlobManifestPin>,
    attempts: HashSet<[u8; 16]>,
}

impl RoutingDeltaVisitor<PersistedRecordIdentity> for DeltaRoster<'_> {
    type Error = SelectedBlobManifestPinDenial;

    fn retained(
        &mut self,
        first: PersistedRecordIdentity,
        last: PersistedRecordIdentity,
    ) -> Result<(), Self::Error> {
        for pin in carried_in_range(self.carried, first, last, |pin| pin.record) {
            self.limits
                .admit_carried(*pin, &mut self.pins, &mut self.attempts)?;
        }
        Ok(())
    }

    fn changed(&mut self, record: PersistedRecordIdentity) -> Result<(), Self::Error> {
        let Some(used) = read_changed_manifest(self.reader, record, self.scratch)? else {
            return Ok(());
        };
        self.limits.observe(
            &self.scratch[..used],
            record,
            &mut self.pins,
            &mut self.attempts,
        )
    }
}

/// Reads a rewritten record's payload when its prefix names a drop-set
/// manifest. Other records stop after the prefix; a manifest larger than the
/// scratch is the same budget denial the full rebuild scan returns.
fn read_changed_manifest(
    reader: &PhysicalRecordReader,
    record: PersistedRecordIdentity,
    scratch: &mut [u8],
) -> Result<Option<usize>, SelectedBlobManifestPinDenial> {
    let mut stream = reader
        .open(
            PhysicalRecordId::from_persisted(record),
            RecordReadLimits::new(RecordByteLimit::new(u32::MAX).expect("nonzero byte limit")),
        )
        .map_err(|_| SelectedBlobManifestPinDenial::ReadUnavailable)?;
    let mut read_into = |target: &mut [u8]| {
        stream
            .read_next(target)
            .map_err(|_| SelectedBlobManifestPinDenial::ReadUnavailable)
    };
    let mut used = 0;
    while used < MANIFEST_PREFIX_BYTES {
        match read_into(&mut scratch[used..MANIFEST_PREFIX_BYTES])? {
            0 => return Ok(None),
            read => used += read,
        }
    }
    if &scratch[..8] != b"WRC11BLB" || !is_manifest_kind(Some(scratch[8])) {
        return Ok(None);
    }
    loop {
        if used == scratch.len() {
            return match read_into(&mut [0_u8; 1])? {
                0 => Ok(Some(used)),
                _ => Err(SelectedBlobManifestPinDenial::BudgetExceeded),
            };
        }
        match read_into(&mut scratch[used..])? {
            0 => return Ok(Some(used)),
            read => used += read,
        }
    }
}

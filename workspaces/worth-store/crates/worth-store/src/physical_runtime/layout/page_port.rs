use std::num::NonZeroU64;

use worth_store_physical_format::PersistedRecordIdentity;

use crate::physical_runtime::{
    LayoutPhysicalAllocation, PhysicalRecordId, PhysicalRecordReader,
    PhysicalScopedAllocationFailure, RecordByteLimit, RecordReadError, RecordReadLimits,
    RecordReadObservation, RecordStreamFailure, ServingPhysicalRuntime,
};

/// The protected page port owns one selected C.5 root and one charged scratch
/// envelope. Every node read routes through `PhysicalRecordReader::open`.
pub(in crate::physical_runtime) struct PhysicalLayoutPagePort<'runtime> {
    runtime: &'runtime ServingPhysicalRuntime,
    reader: PhysicalRecordReader,
    maximum_node_bytes: u32,
    _allocation: LayoutPhysicalAllocation<'runtime>,
}

#[derive(Debug)]
pub enum PhysicalLayoutPageReadFailure {
    Allocation(PhysicalScopedAllocationFailure),
    Read(RecordReadError),
    Stream(RecordStreamFailure),
    ScratchUnavailable,
    NodeTooWide,
}

pub(in crate::physical_runtime) struct PhysicalLayoutNodeRead {
    bytes: Vec<u8>,
    observation: RecordReadObservation,
}

impl PhysicalLayoutNodeRead {
    pub(in crate::physical_runtime) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(in crate::physical_runtime) const fn observation(&self) -> RecordReadObservation {
        self.observation
    }
}

impl<'runtime> PhysicalLayoutPagePort<'runtime> {
    pub(in crate::physical_runtime) fn read_node_from_protected_reader(
        runtime: &ServingPhysicalRuntime,
        reader: &PhysicalRecordReader,
        maximum_node_bytes: u32,
        record: PersistedRecordIdentity,
    ) -> Result<PhysicalLayoutNodeRead, PhysicalLayoutPageReadFailure> {
        let charge = NonZeroU64::new(u64::from(maximum_node_bytes) * 6)
            .ok_or(PhysicalLayoutPageReadFailure::NodeTooWide)?;
        let _allocation = runtime
            .physical_allocations()
            .admit_layout_read(charge)
            .map_err(PhysicalLayoutPageReadFailure::Allocation)?;
        read_node_from_reader(reader, maximum_node_bytes, record)
    }
    pub(in crate::physical_runtime) fn from_protected_reader(
        runtime: &'runtime ServingPhysicalRuntime,
        reader: PhysicalRecordReader,
        maximum_node_bytes: u32,
    ) -> Result<Self, PhysicalLayoutPageReadFailure> {
        // Payload bytes and two independently allocated decoded-node images
        // may overlap; cell Vec metadata makes each image wider than media.
        let charge = NonZeroU64::new(u64::from(maximum_node_bytes) * 6)
            .ok_or(PhysicalLayoutPageReadFailure::NodeTooWide)?;
        let allocation = runtime
            .physical_allocations()
            .admit_layout_read(charge)
            .map_err(PhysicalLayoutPageReadFailure::Allocation)?;
        Ok(Self {
            runtime,
            reader,
            maximum_node_bytes,
            _allocation: allocation,
        })
    }

    pub(in crate::physical_runtime) fn reader(&self) -> &PhysicalRecordReader {
        &self.reader
    }

    pub(in crate::physical_runtime) fn into_reader(self) -> PhysicalRecordReader {
        self.reader
    }

    pub(in crate::physical_runtime) fn admit_scan_stack(
        &self,
    ) -> Result<LayoutPhysicalAllocation<'runtime>, PhysicalScopedAllocationFailure> {
        let charge = NonZeroU64::new(u64::from(self.maximum_node_bytes) * 8)
            .expect("admitted inline page size is nonzero");
        self.runtime
            .physical_allocations()
            .admit_layout_read(charge)
    }

    pub(in crate::physical_runtime) fn read_node(
        &self,
        record: PersistedRecordIdentity,
    ) -> Result<PhysicalLayoutNodeRead, PhysicalLayoutPageReadFailure> {
        read_node_from_reader(&self.reader, self.maximum_node_bytes, record)
    }
}

fn read_node_from_reader(
    reader: &PhysicalRecordReader,
    maximum_node_bytes: u32,
    record: PersistedRecordIdentity,
) -> Result<PhysicalLayoutNodeRead, PhysicalLayoutPageReadFailure> {
    let limit =
        RecordByteLimit::new(maximum_node_bytes).expect("an admitted node byte maximum is nonzero");
    let mut read = reader
        .open(
            PhysicalRecordId::from_persisted(record),
            RecordReadLimits::new(limit),
        )
        .map_err(PhysicalLayoutPageReadFailure::Read)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(maximum_node_bytes as usize)
        .map_err(|_| PhysicalLayoutPageReadFailure::ScratchUnavailable)?;
    let mut scratch = [0_u8; 8192];
    loop {
        let count = read
            .read_next(&mut scratch)
            .map_err(PhysicalLayoutPageReadFailure::Stream)?;
        if count == 0 {
            return Ok(PhysicalLayoutNodeRead {
                bytes,
                observation: read.observation(),
            });
        }
        if bytes.len() + count > maximum_node_bytes as usize {
            return Err(PhysicalLayoutPageReadFailure::NodeTooWide);
        }
        bytes.extend_from_slice(&scratch[..count]);
    }
}

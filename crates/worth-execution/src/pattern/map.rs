use worth_foundational::{ExecutionPosture, ExecutionReport, PartitionIdentity};
use worth_proof::{CanonicalUniqueVec, DisjointKeySetFamily, DisjointKeySetViolation};

use crate::{
    authority::ExecutionResourceLease,
    backend::{AdmittedBatch, BackendKind, BatchDenial, BatchOutcome},
    report::ChargedBytes,
};

mod access;
mod borrowed;
mod declarations;
mod keyless;
mod owned;
mod prepared;
use access::access_memory_bytes;
use declarations::{check_read_key_order, check_read_write_conflicts};
pub use keyless::{KeylessPartition, MapMemoryOverflow};
pub use prepared::PreparedExecutionMap;

pub use crate::backend::{
    BatchStop as MapStop, KernelContext as MapKernelContext, KernelFailure as MapKernelFailure,
    KernelStop as MapKernelStop,
};
pub use crate::oracle::OracleMismatch;

/// A leased run under an automatic posture runs native; every other run is
/// serial.
fn backend_for(lease: Option<&ExecutionResourceLease<'_>>) -> BackendKind {
    if lease.is_some_and(|value| value.resolved_posture() == ExecutionPosture::Automatic) {
        BackendKind::Native
    } else {
        BackendKind::Serial
    }
}

/// One declared access set and memory ceiling for the value dispatched under
/// `identity`. Keys within each set must be in strictly increasing order.
pub struct MapPartition<T, K> {
    pub identity: PartitionIdentity,
    pub value: T,
    pub read_keys: Vec<K>,
    pub write_keys: Vec<K>,
    pub kernel_scratch_bytes: u64,
    pub max_result_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapDenial {
    ExpectedIdentitiesNotCanonical,
    CoverageMismatch,
    ReadKeysNotCanonical {
        partition: usize,
        earlier: usize,
        later: usize,
    },
    WriteKeysNotCanonical {
        partition: PartitionIdentity,
        earlier: usize,
        later: usize,
    },
    WriteSetOverlap {
        earlier: PartitionIdentity,
        later: PartitionIdentity,
        later_key: usize,
    },
    ReadWriteConflict {
        reader: PartitionIdentity,
        writer: PartitionIdentity,
        read_key: usize,
    },
    MemoryOverflow,
}

/// A map whose dispatched values carry checked identity and access declarations.
/// The checked write-set family is retained with the exact admitted batch.
pub struct ExecutionMap<T, K> {
    batch: AdmittedBatch<T>,
    _read_sets: Vec<Vec<K>>,
    _write_sets: DisjointKeySetFamily<PartitionIdentity, K>,
}

pub enum MapOutcome<R, E> {
    Complete {
        values: Vec<R>,
        report: ExecutionReport,
    },
    Stopped {
        completed_prefix: Vec<R>,
        boundary: Option<PartitionIdentity>,
        reason: MapStop<E>,
        report: ExecutionReport,
    },
}

impl<R, E> From<BatchOutcome<R, E>> for MapOutcome<R, E> {
    fn from(outcome: BatchOutcome<R, E>) -> Self {
        match outcome.stop {
            Some(reason) => Self::Stopped {
                completed_prefix: outcome.values,
                boundary: outcome.prefix_boundary,
                reason,
                report: outcome.report,
            },
            None => Self::Complete {
                values: outcome.values,
                report: outcome.report,
            },
        }
    }
}

impl<T, K: Ord + ChargedBytes> ExecutionMap<T, K> {
    /// Query the exact generic execution reservation for a declared shape.
    /// The checked run performs the authoritative reservation before dispatch.
    pub fn declared_memory_requirement<R, E>(
        count: usize,
        input_heap: u64,
        kernel_scratch_bytes: u64,
        declared_result_bytes: u64,
        access_memory_bytes: u64,
    ) -> Option<u64> {
        crate::backend::execution_memory_requirement::<T, R, E>(
            count,
            input_heap,
            kernel_scratch_bytes,
            declared_result_bytes,
            access_memory_bytes,
        )
    }

    /// Full leased reservation, including the actual request's checkpoint
    /// lineage and worker contexts. The later run still reserves it itself.
    pub fn declared_memory_requirement_for_lease<R, E>(
        lease: &ExecutionResourceLease<'_>,
        count: usize,
        input_heap: u64,
        kernel_scratch_bytes: u64,
        declared_result_bytes: u64,
        access_memory_bytes: u64,
    ) -> Option<u64> {
        crate::backend::execution_memory_requirement_for_lease::<T, R, E>(
            lease,
            count,
            input_heap,
            kernel_scratch_bytes,
            declared_result_bytes,
            access_memory_bytes,
        )
    }

    pub fn try_from_declared_partitions(
        expected_identities: Vec<PartitionIdentity>,
        mut partitions: Vec<MapPartition<T, K>>,
    ) -> Result<Self, MapDenial> {
        let expected = CanonicalUniqueVec::try_from_sorted_unique(expected_identities)
            .map_err(|_| MapDenial::ExpectedIdentitiesNotCanonical)?;
        if !expected
            .as_slice()
            .iter()
            .copied()
            .eq(partitions.iter().map(|partition| partition.identity))
        {
            return Err(MapDenial::CoverageMismatch);
        }
        check_read_key_order(&partitions)?;
        let write_sets: Vec<_> = partitions
            .iter_mut()
            .map(|partition| {
                (
                    partition.identity,
                    std::mem::take(&mut partition.write_keys),
                )
            })
            .collect();
        let write_member_capacity = write_sets.capacity();
        let write_sets = DisjointKeySetFamily::try_from_sorted_sets(write_sets).map_err(
            |denial| match denial.violation() {
                DisjointKeySetViolation::MemberIdentityOrder { .. } => MapDenial::CoverageMismatch,
                DisjointKeySetViolation::KeyOrder {
                    member,
                    earlier,
                    later,
                } => MapDenial::WriteKeysNotCanonical {
                    partition: expected.as_slice()[member],
                    earlier,
                    later,
                },
                DisjointKeySetViolation::IntersectingKey {
                    earlier_member,
                    later_member,
                    later_key,
                } => MapDenial::WriteSetOverlap {
                    earlier: expected.as_slice()[earlier_member],
                    later: expected.as_slice()[later_member],
                    later_key,
                },
            },
        )?;
        check_read_write_conflicts(&partitions, &write_sets)?;
        let read_sets: Vec<Vec<K>> = partitions
            .iter_mut()
            .map(|partition| std::mem::take(&mut partition.read_keys))
            .collect();
        let access_memory_bytes = access_memory_bytes(
            &read_sets,
            read_sets.capacity(),
            &write_sets,
            write_member_capacity,
        )
        .ok_or(MapDenial::MemoryOverflow)?;
        let entries = partitions
            .into_iter()
            .map(|partition| {
                (
                    partition.identity,
                    partition.value,
                    partition.kernel_scratch_bytes,
                    partition.max_result_bytes,
                )
            })
            .collect();
        let batch =
            AdmittedBatch::try_admit(entries, access_memory_bytes).map_err(
                |denial| match denial {
                    BatchDenial::Identities => MapDenial::CoverageMismatch,
                    BatchDenial::MemoryOverflow => MapDenial::MemoryOverflow,
                },
            )?;
        Ok(Self {
            batch,
            _read_sets: read_sets,
            _write_sets: write_sets,
        })
    }
}

impl<T, K> ExecutionMap<T, K> {
    pub fn partition_count(&self) -> usize {
        self.batch.identities().len()
    }

    pub fn identities(&self) -> &[PartitionIdentity] {
        self.batch.identities()
    }

    pub(crate) fn retained_result_bytes<R>(&self) -> Option<u64> {
        self.batch.retained_result_bytes::<R>()
    }
}

impl<R, E> MapOutcome<R, E> {
    pub fn report(&self) -> ExecutionReport {
        match self {
            Self::Complete { report, .. } | Self::Stopped { report, .. } => *report,
        }
    }
}

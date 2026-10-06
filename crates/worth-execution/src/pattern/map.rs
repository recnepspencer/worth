use std::collections::BTreeMap;

use worth_foundational::{ExecutionPosture, ExecutionReport, PartitionIdentity};
use worth_proof::{CanonicalUniqueVec, DisjointKeySetFamily, DisjointKeySetViolation};

use crate::{
    authority::{ExecutionMemoryReservation, ExecutionResourceLease},
    backend::{run_checked_batch_taking, AdmittedBatch, BackendKind, BatchDenial, BatchOutcome},
    oracle::{self, CanonicalBits},
    report::ChargedBytes,
};

mod access;
mod keyless;
mod prepared;
use access::access_memory_bytes;
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

impl<T: Sync + ChargedBytes, K> ExecutionMap<T, K> {
    /// Consume the checked map and retain its exact live memory admission
    /// before domain evaluators run. Dispatch later reserves only a worker.
    pub fn prepare_run<'authority, R, E>(
        self,
        lease: ExecutionResourceLease<'authority>,
    ) -> Result<PreparedExecutionMap<'authority, T, K, R, E>, crate::LeaseDenial>
    where
        R: Send + ChargedBytes,
        E: Send + ChargedBytes,
    {
        prepared::prepare_map(self, lease)
    }

    pub fn run<R, E, F>(
        &self,
        lease: Option<&ExecutionResourceLease<'_>>,
        kernel: F,
    ) -> MapOutcome<R, E>
    where
        R: Send + ChargedBytes,
        E: Send + ChargedBytes,
        F: Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Sync,
    {
        self.run_with_backend(lease, backend_for(lease), kernel)
    }

    /// [`Self::run`], with the run's memory admission taking over `inputs`,
    /// the caller's reservation for what the partitions hold. It becomes the
    /// run's own, at the run's exact bytes on the run's lease or serial
    /// budget, in one ledger step, so the inputs are never unreserved between
    /// the caller's hold and the run's. The run releases it when it settles.
    pub fn run_taking<R, E, F>(
        &self,
        lease: Option<&ExecutionResourceLease<'_>>,
        inputs: ExecutionMemoryReservation,
        kernel: F,
    ) -> MapOutcome<R, E>
    where
        R: Send + ChargedBytes,
        E: Send + ChargedBytes,
        F: Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Sync,
    {
        self.run_with_backend_taking(lease, backend_for(lease), Some(inputs), kernel)
    }

    pub(crate) fn run_with_backend<R, E, F>(
        &self,
        lease: Option<&ExecutionResourceLease<'_>>,
        backend: BackendKind,
        kernel: F,
    ) -> MapOutcome<R, E>
    where
        R: Send + ChargedBytes,
        E: Send + ChargedBytes,
        F: Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Sync,
    {
        self.run_with_backend_taking(lease, backend, None, kernel)
    }

    pub(crate) fn run_with_backend_taking<R, E, F>(
        &self,
        lease: Option<&ExecutionResourceLease<'_>>,
        backend: BackendKind,
        taken: Option<ExecutionMemoryReservation>,
        kernel: F,
    ) -> MapOutcome<R, E>
    where
        R: Send + ChargedBytes,
        E: Send + ChargedBytes,
        F: Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Sync,
    {
        run_checked_batch_taking(lease, &self.batch, backend, taken, &kernel).into()
    }

    /// Certification runs the exact admitted inputs and kernel on the serial
    /// oracle and a schedule-perturbed backend before comparing canonical cost.
    pub fn certify<R, E, F>(
        &self,
        lease: &ExecutionResourceLease<'_>,
        seed: u64,
        kernel: F,
    ) -> Result<MapOutcome<R, E>, OracleMismatch>
    where
        R: Send + ChargedBytes + CanonicalBits,
        E: Send + ChargedBytes + CanonicalBits,
        F: Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Sync,
    {
        oracle::certify(lease, &self.batch, BackendKind::Perturbation(seed), &kernel)
            .map(MapOutcome::from)
    }
}

impl<R, E> MapOutcome<R, E> {
    pub fn report(&self) -> ExecutionReport {
        match self {
            Self::Complete { report, .. } | Self::Stopped { report, .. } => *report,
        }
    }
}

fn check_read_key_order<T, K: Ord>(partitions: &[MapPartition<T, K>]) -> Result<(), MapDenial> {
    for (partition, entry) in partitions.iter().enumerate() {
        for (earlier, pair) in entry.read_keys.windows(2).enumerate() {
            if pair[0] >= pair[1] {
                return Err(MapDenial::ReadKeysNotCanonical {
                    partition,
                    earlier,
                    later: earlier + 1,
                });
            }
        }
    }
    Ok(())
}

fn check_read_write_conflicts<T, K: Ord>(
    partitions: &[MapPartition<T, K>],
    write_sets: &DisjointKeySetFamily<PartitionIdentity, K>,
) -> Result<(), MapDenial> {
    let writers: BTreeMap<&K, usize> = write_sets
        .members()
        .iter()
        .enumerate()
        .flat_map(|(writer, (_, keys))| keys.iter().map(move |key| (key, writer)))
        .collect();
    for (reader, partition) in partitions.iter().enumerate() {
        for (read_key, key) in partition.read_keys.iter().enumerate() {
            if let Some(&writer) = writers.get(key) {
                if reader != writer {
                    return Err(MapDenial::ReadWriteConflict {
                        reader: partition.identity,
                        writer: write_sets.members()[writer].0,
                        read_key,
                    });
                }
            }
        }
    }
    Ok(())
}

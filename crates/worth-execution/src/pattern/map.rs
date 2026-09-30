use std::{collections::BTreeMap, mem::size_of};

use worth_foundational::{ExecutionPosture, ExecutionReport, PartitionIdentity};
use worth_proof::{CanonicalUniqueVec, DisjointKeySetFamily, DisjointKeySetViolation};

use crate::{
    authority::ExecutionResourceLease,
    backend::{run_checked_batch, AdmittedBatch, BackendKind, BatchDenial, BatchOutcome},
    oracle::{self, CanonicalBits},
    report::ChargedBytes,
};

pub use crate::backend::{
    BatchStop as MapStop, KernelContext as MapKernelContext, KernelFailure as MapKernelFailure,
    KernelStop as MapKernelStop,
};
pub use crate::oracle::OracleMismatch;

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
        )?;
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

    pub fn partition_count(&self) -> usize {
        self.batch.identities().len()
    }
}

impl<T: Sync + ChargedBytes, K> ExecutionMap<T, K> {
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
        let backend =
            if lease.is_some_and(|value| value.resolved_posture() == ExecutionPosture::Automatic) {
                BackendKind::Native
            } else {
                BackendKind::Serial
            };
        run_checked_batch(lease, &self.batch, backend, &kernel).into()
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

fn access_memory_bytes<K: ChargedBytes>(
    read_sets: &[Vec<K>],
    read_set_capacity: usize,
    write_sets: &DisjointKeySetFamily<PartitionIdentity, K>,
    write_member_capacity: usize,
) -> Result<u64, MapDenial> {
    let keys = read_sets
        .iter()
        .chain(write_sets.members().iter().map(|(_, keys)| keys));
    let mut total = 0_u64;
    for set in keys {
        let inline = set
            .capacity()
            .checked_mul(size_of::<K>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(MapDenial::MemoryOverflow)?;
        total = total.checked_add(inline).ok_or(MapDenial::MemoryOverflow)?;
        for key in set {
            total = total
                .checked_add(key.additional_charged_bytes())
                .ok_or(MapDenial::MemoryOverflow)?;
        }
    }
    let read_members = read_set_capacity
        .checked_mul(size_of::<Vec<K>>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(MapDenial::MemoryOverflow)?;
    let write_members = write_member_capacity
        .checked_mul(size_of::<(PartitionIdentity, Vec<K>)>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(MapDenial::MemoryOverflow)?;
    total
        .checked_add(read_members)
        .and_then(|bytes| bytes.checked_add(write_members))
        .ok_or(MapDenial::MemoryOverflow)
}

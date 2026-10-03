use std::mem::size_of;

use worth_execution::{
    ChargedBytes, ExecutionResourceLease, ExecutionScan, MapKernelContext, MapKernelFailure,
    ScanDenial, ScanOutcome,
};
use worth_foundational::{ExecutionReport, PartitionIdentity};

use super::{PacketExecutionStop, ReadOnlyPacket};

const PREPARATION_MEMORY_DIVISOR: u64 = 8;

pub(crate) struct PacketPreparationBudget<'a, 'b, 'c> {
    context: &'a mut MapKernelContext<'b, 'c>,
    ceiling: u64,
    claimed: u64,
}

impl PacketPreparationBudget<'_, '_, '_> {
    pub(crate) const fn claimed_bytes(&self) -> u64 {
        self.claimed
    }
    pub(crate) fn checkpoint(
        &mut self,
        units: u64,
    ) -> Result<(), MapKernelFailure<super::PacketBudgetDenial>> {
        self.context
            .checkpoint(units)
            .map_err(MapKernelFailure::Stop)
    }

    pub(crate) fn claim(
        &mut self,
        bytes: u64,
    ) -> Result<(), MapKernelFailure<super::PacketBudgetDenial>> {
        self.claimed = self
            .claimed
            .checked_add(bytes)
            .filter(|claimed| *claimed <= self.ceiling)
            .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
        Ok(())
    }

    pub(crate) fn claim_packet<T>(
        &mut self,
    ) -> Result<(), MapKernelFailure<super::PacketBudgetDenial>> {
        self.checkpoint(1)?;
        // The dispatcher subsequently holds this descriptor, an identity,
        // and a MapPartition wrapper concurrently until map admission.
        self.claim(
            (size_of::<ReadOnlyPacket<T>>() as u64)
                .saturating_mul(4)
                .saturating_add(size_of::<PartitionIdentity>() as u64),
        )
    }
}

struct PreparedPackets<T>(Vec<ReadOnlyPacket<T>>);

struct PreparedArtifact<T> {
    value: T,
    owned_bytes: u64,
}

impl<T> ChargedBytes for PreparedArtifact<T> {
    fn additional_charged_bytes(&self) -> u64 {
        self.owned_bytes
    }
}

/// Seal one borrowed-source artifact under the same request preparation account
/// used by packet construction. The caller claims each allocation before growth.
pub(crate) fn prepare_borrowed_artifact<T>(
    lease: &ExecutionResourceLease<'_>,
    work_budget: Option<&super::RequestWorkBudget>,
    build: impl FnOnce(
        &mut PacketPreparationBudget<'_, '_, '_>,
    ) -> Result<(T, u64), MapKernelFailure<super::PacketBudgetDenial>>,
) -> Result<T, PacketExecutionStop> {
    let identity = PartitionIdentity::new(1);
    let scan = ExecutionScan::try_from_ordered(vec![identity], vec![(identity, ())])
        .map_err(|_| PacketExecutionStop::Admission(worth_execution::MapDenial::MemoryOverflow))?;
    let ceiling = lease.policy().budget().charged_memory_bytes() / PREPARATION_MEMORY_DIVISOR;
    let mut build = Some(build);
    let outcome = super::run_with_remaining_request_work(
        lease,
        work_budget,
        |child_lease| {
            scan.run(
                Some(child_lease),
                (),
                0,
                ceiling,
                0,
                ceiling,
                |_, _, context| {
                    let mut budget = PacketPreparationBudget {
                        context,
                        ceiling,
                        claimed: 0,
                    };
                    budget.checkpoint(0)?;
                    let (value, owned_bytes) =
                        build.take().expect("one artifact preparation")(&mut budget)?;
                    Ok(((), PreparedArtifact { value, owned_bytes }))
                },
            )
        },
        ScanOutcome::report,
    );
    match outcome {
        ScanOutcome::Complete { mut prefixes, .. } => {
            Ok(prefixes.pop().expect("one artifact output").value)
        }
        ScanOutcome::Stopped {
            boundary, reason, ..
        } => Err(PacketExecutionStop::Execution { boundary, reason }),
    }
}

impl<T> ChargedBytes for PreparedPackets<T> {
    fn additional_charged_bytes(&self) -> u64 {
        (self.0.capacity() as u64).saturating_mul(size_of::<ReadOnlyPacket<T>>() as u64)
    }
}

/// Admit packet descriptors before allocating them. Borrowed source rows stay
/// in the caller's immutable intent and are cloned only inside a checked map.
pub(crate) fn prepare_borrowed_packets<T>(
    lease: &ExecutionResourceLease<'_>,
    work_budget: Option<&super::RequestWorkBudget>,
    build: impl FnOnce(
        &mut PacketPreparationBudget<'_, '_, '_>,
    )
        -> Result<Vec<ReadOnlyPacket<T>>, MapKernelFailure<super::PacketBudgetDenial>>,
) -> Result<(Vec<ReadOnlyPacket<T>>, ExecutionReport), PacketExecutionStop> {
    let identity = PartitionIdentity::new(1);
    let scan = ExecutionScan::try_from_ordered(vec![identity], vec![(identity, ())]).map_err(
        |denial| {
            PacketExecutionStop::Admission(match denial {
                ScanDenial::IdentitiesNotCanonical => {
                    worth_execution::MapDenial::ExpectedIdentitiesNotCanonical
                }
                ScanDenial::CoverageMismatch => worth_execution::MapDenial::CoverageMismatch,
                ScanDenial::MemoryOverflow => worth_execution::MapDenial::MemoryOverflow,
            })
        },
    )?;
    let ceiling = lease.policy().budget().charged_memory_bytes() / PREPARATION_MEMORY_DIVISOR;
    let mut build = Some(build);
    let outcome = super::run_with_remaining_request_work(
        lease,
        work_budget,
        |child_lease| {
            scan.run(
                Some(child_lease),
                (),
                0,
                ceiling,
                0,
                ceiling,
                |_, _, context| {
                    let mut budget = PacketPreparationBudget {
                        context,
                        ceiling,
                        claimed: 0,
                    };
                    budget.checkpoint(0)?;
                    let packets = build.take().expect("one packet preparation step")(&mut budget)?;
                    Ok(((), PreparedPackets(packets)))
                },
            )
        },
        ScanOutcome::report,
    );
    match outcome {
        ScanOutcome::Complete {
            mut prefixes,
            report,
            ..
        } => Ok((
            prefixes.pop().expect("one packet preparation output").0,
            report,
        )),
        ScanOutcome::Stopped {
            boundary, reason, ..
        } => Err(PacketExecutionStop::Execution { boundary, reason }),
    }
}

use worth_execution::{
    ChargedBytes, ExecutionMap, ExecutionResourceLease, MapDenial, MapKernelContext,
    MapKernelFailure, MapOutcome, MapPartition, MapStop, ScanDenial,
};
use worth_foundational::{ExecutionReport, PartitionIdentity};

use super::super::query_packetization::PacketizedQueryWork;
use crate::query::data::{QueryExecutionOutcome, QueryWorkerFragment};

const RESULT_BUDGET_DIVISOR: u64 = 4;

struct QueryPacketMapInput {
    ordinal: usize,
    work: PacketizedQueryWork,
}

impl ChargedBytes for QueryPacketMapInput {
    fn additional_charged_bytes(&self) -> u64 {
        self.work.additional_charged_bytes()
    }
}

#[derive(Debug)]
pub enum QueryReadExecutionStop {
    PreparationAdmission(ScanDenial),
    PreparationStopped {
        reason: MapStop<()>,
        report: ExecutionReport,
    },
    PreparationLease(worth_execution::LeaseDenial),
    PacketAdmission(MapDenial),
    PacketStopped {
        boundary: Option<PartitionIdentity>,
        reason: MapStop<QueryReadPacketDenial>,
        report: ExecutionReport,
    },
    CompletionAdmission(ScanDenial),
    CompletionStopped {
        reason: MapStop<()>,
        report: ExecutionReport,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum QueryReadPacketDenial {
    FragmentUnavailable,
}

impl ChargedBytes for QueryReadPacketDenial {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

#[derive(Debug)]
pub struct QueryLeasedReadOutcome {
    pub outcome: QueryExecutionOutcome,
    pub report: ExecutionReport,
}

/// Charges packet-owned buffers before growing them. The map also checks the
/// completed fragment's actual charge; this guard prevents an unbounded
/// intermediate allocation before that settlement point.
pub(super) struct QueryPacketBudget<'context, 'meter, 'lease> {
    context: &'context mut MapKernelContext<'meter, 'lease>,
    ceiling: u64,
    result_bytes: u64,
    scratch_bytes: u64,
}

impl<'context, 'meter, 'lease> QueryPacketBudget<'context, 'meter, 'lease> {
    pub(super) fn new(
        context: &'context mut MapKernelContext<'meter, 'lease>,
        ceiling: u64,
    ) -> Result<Self, MapKernelFailure<QueryReadPacketDenial>> {
        let result_bytes = std::mem::size_of::<QueryWorkerFragment>() as u64;
        if result_bytes > ceiling {
            return Err(MapKernelFailure::ResultCapacityExceeded);
        }
        Ok(Self {
            context,
            ceiling,
            result_bytes,
            scratch_bytes: 0,
        })
    }

    pub(super) fn checkpoint(
        &mut self,
        units: u64,
    ) -> Result<(), MapKernelFailure<QueryReadPacketDenial>> {
        self.context
            .checkpoint(units)
            .map_err(MapKernelFailure::Stop)
    }

    pub(super) fn claim_scratch(
        &mut self,
        additional_bytes: u64,
    ) -> Result<(), MapKernelFailure<QueryReadPacketDenial>> {
        self.scratch_bytes = self
            .scratch_bytes
            .checked_add(additional_bytes)
            .filter(|total| *total <= self.ceiling)
            .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
        Ok(())
    }

    pub(super) fn check_temporary_record(
        &self,
        bytes: u64,
    ) -> Result<(), MapKernelFailure<QueryReadPacketDenial>> {
        self.scratch_bytes
            .checked_add(bytes)
            .filter(|total| *total <= self.ceiling)
            .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
        Ok(())
    }

    pub(super) fn push_result<T: ChargedBytes>(
        &mut self,
        records: &mut Vec<T>,
        record: T,
    ) -> Result<(), MapKernelFailure<QueryReadPacketDenial>> {
        self.reserve_result_record(records, &record)?;
        records.push(record);
        Ok(())
    }

    pub(super) fn push_cloned_result<T: ChargedBytes + Clone>(
        &mut self,
        records: &mut Vec<T>,
        record: &T,
    ) -> Result<(), MapKernelFailure<QueryReadPacketDenial>> {
        self.reserve_result_record(records, record)?;
        records.push(record.clone());
        Ok(())
    }

    fn reserve_result_record<T: ChargedBytes>(
        &mut self,
        records: &mut Vec<T>,
        record: &T,
    ) -> Result<(), MapKernelFailure<QueryReadPacketDenial>> {
        let old_capacity = records.capacity();
        let required_capacity = old_capacity.max(records.len().saturating_add(1));
        let inline_growth = required_capacity
            .saturating_sub(old_capacity)
            .saturating_mul(std::mem::size_of::<T>()) as u64;
        let projected = self
            .result_bytes
            .checked_add(inline_growth)
            .and_then(|total| total.checked_add(record.additional_charged_bytes()))
            .filter(|total| *total <= self.ceiling)
            .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
        records
            .try_reserve_exact(1)
            .map_err(|_| MapKernelFailure::ResultCapacityExceeded)?;
        let actual_growth = records
            .capacity()
            .saturating_sub(old_capacity)
            .saturating_mul(std::mem::size_of::<T>()) as u64;
        self.result_bytes = self
            .result_bytes
            .checked_add(actual_growth)
            .and_then(|total| total.checked_add(record.additional_charged_bytes()))
            .filter(|total| *total <= self.ceiling)
            .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
        debug_assert!(self.result_bytes >= projected);
        Ok(())
    }
}

pub(in crate::visibility::materialization::read_records::reader) fn execute_leased_query_packets(
    packets: Vec<PacketizedQueryWork>,
    lease: &ExecutionResourceLease<'_>,
    execute: impl Fn(
            &PacketizedQueryWork,
            usize,
            u64,
            &mut MapKernelContext<'_, '_>,
        ) -> Result<QueryWorkerFragment, MapKernelFailure<QueryReadPacketDenial>>
        + Sync,
) -> Result<(Vec<QueryWorkerFragment>, ExecutionReport), QueryReadExecutionStop> {
    let packet_count = packets.len();
    let packet_count_u64 = u64::try_from(packet_count).unwrap_or(u64::MAX);
    let per_packet_ceiling = lease.policy().budget().charged_memory_bytes()
        / RESULT_BUDGET_DIVISOR
            .saturating_mul(packet_count_u64)
            .max(1);
    let identities: Vec<_> = (0..packet_count)
        .map(|ordinal| PartitionIdentity::new(ordinal as u64 + 1))
        .collect();
    let partitions = identities
        .iter()
        .copied()
        .zip(packets.into_iter().enumerate())
        .map(|(identity, (ordinal, work))| MapPartition {
            identity,
            value: QueryPacketMapInput { ordinal, work },
            read_keys: Vec::<u64>::new(),
            write_keys: Vec::new(),
            kernel_scratch_bytes: per_packet_ceiling,
            max_result_bytes: per_packet_ceiling,
        })
        .collect();
    let map = ExecutionMap::try_from_declared_partitions(identities, partitions)
        .map_err(QueryReadExecutionStop::PacketAdmission)?;
    match map.run(Some(lease), |input, context| {
        context.checkpoint(0)?;
        execute(&input.work, input.ordinal, per_packet_ceiling, context)
    }) {
        MapOutcome::Complete { values, report } => Ok((values, report)),
        MapOutcome::Stopped {
            boundary,
            reason,
            report,
            ..
        } => Err(QueryReadExecutionStop::PacketStopped {
            boundary,
            reason,
            report,
        }),
    }
}

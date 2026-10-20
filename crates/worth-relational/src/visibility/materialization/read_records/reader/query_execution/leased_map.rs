use worth_execution::{
    ChargedBytes, ExecutionResourceLease, MapDenial, MapKernelContext, MapKernelFailure,
    MapOutcome, MapStop, ScanDenial,
};
use worth_foundational::{ExecutionReport, PartitionIdentity};

use super::super::query_packetization::PacketizedQueryWork;
use crate::query::data::{QueryExecutionOutcome, QueryWorkerFragment};

const RESULT_BUDGET_DIVISOR: u64 = 4;

mod denial_memory;
mod memory;
mod packet_identity;
mod result_growth;
mod tracked;
pub(super) use memory::QueryPacketMemory;

#[cfg(test)]
mod caller_lease_tests;

struct QueryPacketMapInput {
    ordinal: usize,
    work: PacketizedQueryWork,
}

impl ChargedBytes for QueryPacketMapInput {
    fn additional_charged_bytes(&self) -> u64 {
        self.work.additional_charged_bytes()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum QueryReadExecutionStop {
    ReadRequest(crate::execution::RelationalExecutionDenialCause),
    Request(worth_execution::WorkCeilingDenial),
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

impl std::fmt::Display for QueryReadExecutionStop {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "relational query read stopped: {self:?}")
    }
}

impl std::error::Error for QueryReadExecutionStop {}

#[derive(Debug, PartialEq, Eq)]
pub enum QueryReadPacketDenial {
    ReadRequest(crate::execution::RelationalExecutionDenialCause),
    FragmentUnavailable,
}

impl ChargedBytes for QueryReadPacketDenial {
    fn additional_charged_bytes(&self) -> u64 {
        match self {
            Self::ReadRequest(cause) => denial_memory::inline(cause),
            Self::FragmentUnavailable => 0,
        }
    }
}

#[derive(Debug)]
pub struct QueryLeasedReadOutcome {
    pub outcome: QueryExecutionOutcome,
    pub report: Option<ExecutionReport>,
}

/// Charges packet-owned buffers before growing them. The map also checks the
/// completed fragment's actual charge; this guard prevents an unbounded
/// intermediate allocation before that settlement point.
pub(super) struct QueryPacketBudget<'context, 'meter, 'lease> {
    context: &'context mut MapKernelContext<'meter, 'lease>,
    memory: &'context QueryPacketMemory,
    result_bytes: u64,
    scratch_bytes: u64,
    temporary_bytes: std::cell::Cell<u64>,
}

impl<'context, 'meter, 'lease> QueryPacketBudget<'context, 'meter, 'lease> {
    pub(super) fn new(
        context: &'context mut MapKernelContext<'meter, 'lease>,
        memory: &'context QueryPacketMemory,
    ) -> Result<Self, MapKernelFailure<QueryReadPacketDenial>> {
        let result_bytes = std::mem::size_of::<QueryWorkerFragment>() as u64;
        if memory.ceiling.is_some_and(|ceiling| result_bytes > ceiling) {
            return Err(MapKernelFailure::ResultCapacityExceeded);
        }
        memory.result(result_bytes)?;
        Ok(Self {
            context,
            memory,
            result_bytes,
            scratch_bytes: 0,
            temporary_bytes: std::cell::Cell::new(0),
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
        let next = self.memory.add(self.scratch_bytes, additional_bytes)?;
        self.memory
            .scratch(self.memory.add(next, self.temporary_bytes.get())?)?;
        self.scratch_bytes = next;
        Ok(())
    }

    pub(super) fn settle_temporary_record(
        &self,
        bytes: u64,
    ) -> Result<(), MapKernelFailure<QueryReadPacketDenial>> {
        if self.memory.ceiling.is_some() {
            return self.check_temporary_record(bytes);
        }
        self.memory
            .settle_scratch(self.memory.add(self.scratch_bytes, bytes)?)?;
        self.temporary_bytes.set(bytes);
        Ok(())
    }

    pub(super) fn check_temporary_record(
        &self,
        bytes: u64,
    ) -> Result<(), MapKernelFailure<QueryReadPacketDenial>> {
        let peak = self.memory.add(self.scratch_bytes, bytes)?;
        self.memory.scratch(peak)?;
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
        result_growth::reserve_record(self.memory, &mut self.result_bytes, records, record)
    }
}

pub(in crate::visibility::materialization::read_records::reader) fn execute_leased_query_packets(
    packets: Vec<PacketizedQueryWork>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
    execute: impl Fn(
            &PacketizedQueryWork,
            usize,
            &QueryPacketMemory,
            &mut MapKernelContext<'_, '_>,
        ) -> Result<QueryWorkerFragment, MapKernelFailure<QueryReadPacketDenial>>
        + Sync,
) -> Result<(Vec<QueryWorkerFragment>, ExecutionReport), QueryReadExecutionStop> {
    let identities = packet_identity::identities(packets);
    schedule_query_packets(identities, execution, lease, execute)
}

fn schedule_query_packets(
    packets: Vec<packet_identity::NamedQueryPacket>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
    execute: impl Fn(
            &PacketizedQueryWork,
            usize,
            &QueryPacketMemory,
            &mut MapKernelContext<'_, '_>,
        ) -> Result<QueryWorkerFragment, MapKernelFailure<QueryReadPacketDenial>>
        + Sync,
) -> Result<(Vec<QueryWorkerFragment>, ExecutionReport), QueryReadExecutionStop> {
    let packet_count = packets.len();
    let packet_count_u64 = u64::try_from(packet_count).unwrap_or(u64::MAX);
    let Some(limit) = execution.memory_limit() else {
        return tracked::schedule(packets, lease, execute);
    };
    let per_packet_ceiling = limit
        / RESULT_BUDGET_DIVISOR
            .saturating_mul(packet_count_u64)
            .max(1);
    let memory = QueryPacketMemory::declared(per_packet_ceiling);
    let map = packet_identity::declared_map(packets, per_packet_ceiling)?;
    match map.run(lease, |input, context| {
        context.checkpoint(0)?;
        execute(&input.work, input.ordinal, &memory, context)
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

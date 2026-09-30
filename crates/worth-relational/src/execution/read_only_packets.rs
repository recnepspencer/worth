use worth_execution::{
    ChargedBytes, ExecutionMap, ExecutionResourceLease, MapDenial, MapKernelContext,
    MapKernelFailure, MapOutcome, MapPartition, MapStop,
};
use worth_foundational::PartitionIdentity;

/// A read-only preparation packet may inspect an immutable snapshot but must
/// publish its result only after the whole canonical batch settles.
#[derive(Debug)]
pub(crate) enum PacketExecutionStop {
    Admission(MapDenial),
    Execution {
        boundary: Option<PartitionIdentity>,
        reason: MapStop<PacketBudgetDenial>,
    },
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum PacketBudgetDenial {
    ScratchCapacityExceeded,
    UncheckedCustomKernel,
}

impl ChargedBytes for PacketBudgetDenial {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

pub(crate) struct PacketKernelContext<'a, 'b, 'c> {
    meter: Option<&'a mut MapKernelContext<'b, 'c>>,
    scratch_limit: Option<u64>,
    result_limit: Option<u64>,
    scratch_bytes: u64,
    result_bytes: u64,
}

impl PacketKernelContext<'_, '_, '_> {
    pub(crate) fn remaining_work(&self) -> u64 {
        self.meter
            .as_ref()
            .map_or(u64::MAX, |meter| meter.remaining_work())
    }

    pub(crate) fn remaining_scratch(&self) -> u64 {
        self.scratch_limit
            .map_or(u64::MAX, |limit| limit.saturating_sub(self.scratch_bytes))
    }

    pub(crate) fn account_completed_work(
        &mut self,
        units: u64,
    ) -> Result<(), MapKernelFailure<PacketBudgetDenial>> {
        self.meter.as_mut().map_or(Ok(()), |meter| {
            meter
                .account_completed_work(units)
                .map_err(MapKernelFailure::Stop)
        })
    }
    pub(crate) fn checkpoint(
        &mut self,
        units: u64,
    ) -> Result<(), MapKernelFailure<PacketBudgetDenial>> {
        self.meter.as_mut().map_or(Ok(()), |meter| {
            meter.checkpoint(units).map_err(MapKernelFailure::Stop)
        })
    }

    pub(crate) fn claim_scratch(
        &mut self,
        bytes: u64,
    ) -> Result<(), MapKernelFailure<PacketBudgetDenial>> {
        self.scratch_bytes =
            self.scratch_bytes
                .checked_add(bytes)
                .ok_or(MapKernelFailure::Domain(
                    PacketBudgetDenial::ScratchCapacityExceeded,
                ))?;
        if self
            .scratch_limit
            .is_some_and(|limit| self.scratch_bytes > limit)
        {
            return Err(MapKernelFailure::Domain(
                PacketBudgetDenial::ScratchCapacityExceeded,
            ));
        }
        Ok(())
    }

    pub(crate) fn check_scratch_peak(
        &self,
        bytes: u64,
    ) -> Result<(), MapKernelFailure<PacketBudgetDenial>> {
        let peak = self
            .scratch_bytes
            .checked_add(bytes)
            .ok_or(MapKernelFailure::Domain(
                PacketBudgetDenial::ScratchCapacityExceeded,
            ))?;
        if self.scratch_limit.is_some_and(|limit| peak > limit) {
            return Err(MapKernelFailure::Domain(
                PacketBudgetDenial::ScratchCapacityExceeded,
            ));
        }
        Ok(())
    }

    pub(crate) fn claim_result(
        &mut self,
        bytes: u64,
    ) -> Result<(), MapKernelFailure<PacketBudgetDenial>> {
        self.result_bytes = self
            .result_bytes
            .checked_add(bytes)
            .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
        if self
            .result_limit
            .is_some_and(|limit| self.result_bytes > limit)
        {
            return Err(MapKernelFailure::ResultCapacityExceeded);
        }
        Ok(())
    }

    pub(crate) fn check_result_peak(
        &self,
        bytes: u64,
    ) -> Result<(), MapKernelFailure<PacketBudgetDenial>> {
        let peak = self
            .result_bytes
            .checked_add(bytes)
            .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
        if self.result_limit.is_some_and(|limit| peak > limit) {
            return Err(MapKernelFailure::ResultCapacityExceeded);
        }
        Ok(())
    }

    pub(crate) fn ensure_result_total(
        &mut self,
        bytes: u64,
    ) -> Result<(), MapKernelFailure<PacketBudgetDenial>> {
        if bytes > self.result_bytes {
            self.claim_result(bytes - self.result_bytes)?;
        }
        Ok(())
    }
}

impl From<PacketExecutionStop> for crate::transactions::data::CommitExecutionDenial {
    fn from(stop: PacketExecutionStop) -> Self {
        use crate::transactions::data::CommitExecutionDenialKind as Kind;
        use worth_execution::{MapKernelFailure, MapKernelStop};
        let (kind, partition_identity) = match stop {
            PacketExecutionStop::Admission(denial) => (
                if denial == MapDenial::MemoryOverflow {
                    Kind::ResourceExhausted
                } else {
                    Kind::Admission
                },
                None,
            ),
            PacketExecutionStop::Execution { boundary, reason } => {
                let kind = match reason {
                    MapStop::Admission(_) => Kind::ResourceExhausted,
                    MapStop::WorkExhausted { .. } => Kind::WorkExhausted,
                    MapStop::Failure { cause, .. } => match cause {
                        MapKernelFailure::Stop(MapKernelStop::Cancelled) => Kind::Cancelled,
                        MapKernelFailure::Stop(MapKernelStop::DeadlineElapsed) => {
                            Kind::DeadlineElapsed
                        }
                        MapKernelFailure::Stop(_) => Kind::WorkExhausted,
                        MapKernelFailure::ResultCapacityExceeded => Kind::ResultCapacityExceeded,
                        MapKernelFailure::Domain(PacketBudgetDenial::ScratchCapacityExceeded) => {
                            Kind::ResourceExhausted
                        }
                        MapKernelFailure::Domain(PacketBudgetDenial::UncheckedCustomKernel) => {
                            Kind::Admission
                        }
                        MapKernelFailure::Panic => Kind::WorkerFailed,
                    },
                };
                (kind, boundary.map(PartitionIdentity::value))
            }
        };
        Self {
            kind,
            partition_identity,
        }
    }
}

impl From<PacketExecutionStop> for crate::transactions::data::TransactionCommitError {
    fn from(stop: PacketExecutionStop) -> Self {
        Self::execution(stop.into())
    }
}

struct ChargedPacketResult<T> {
    value: T,
    owned_bytes: u64,
}

pub(crate) struct ReadOnlyPacket<T> {
    pub identity: PartitionIdentity,
    pub value: T,
    pub input_bytes: u64,
    pub kernel_scratch_bytes: u64,
    pub max_result_bytes: u64,
}

struct ChargedPacketInput<T> {
    value: T,
    owned_bytes: u64,
    scratch_limit: u64,
    result_limit: u64,
}

impl<T> ChargedBytes for ChargedPacketInput<T> {
    fn additional_charged_bytes(&self) -> u64 {
        self.owned_bytes
    }
}

impl<T> ChargedBytes for ChargedPacketResult<T> {
    fn additional_charged_bytes(&self) -> u64 {
        self.owned_bytes
    }
}

/// Input retention, kernel scratch, and result capacity are declared for each
/// stable packet before dispatch. Shared immutable authority is only borrowed.
pub(crate) fn execute_read_only_packets<P, T, F, C>(
    packets: Vec<ReadOnlyPacket<P>>,
    lease: Option<&ExecutionResourceLease<'_>>,
    kernel: F,
    charged_bytes: C,
) -> Result<Vec<T>, PacketExecutionStop>
where
    P: Sync,
    T: Send,
    F: Fn(
            &P,
            &mut PacketKernelContext<'_, '_, '_>,
        ) -> Result<T, MapKernelFailure<PacketBudgetDenial>>
        + Sync,
    C: Fn(&T) -> u64 + Sync,
{
    execute_read_only_packets_with_budget(packets, lease, None, kernel, charged_bytes)
}

pub(crate) fn execute_read_only_packets_with_budget<P, T, F, C>(
    packets: Vec<ReadOnlyPacket<P>>,
    lease: Option<&ExecutionResourceLease<'_>>,
    budget: Option<&super::RequestWorkBudget>,
    kernel: F,
    charged_bytes: C,
) -> Result<Vec<T>, PacketExecutionStop>
where
    P: Sync,
    T: Send,
    F: Fn(
            &P,
            &mut PacketKernelContext<'_, '_, '_>,
        ) -> Result<T, MapKernelFailure<PacketBudgetDenial>>
        + Sync,
    C: Fn(&T) -> u64 + Sync,
{
    let identities = packets
        .iter()
        .map(|packet| packet.identity)
        .collect::<Vec<_>>();
    if lease.is_none() {
        if identities.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(PacketExecutionStop::Admission(
                MapDenial::ExpectedIdentitiesNotCanonical,
            ));
        }
        return Ok(packets
            .iter()
            .map(|packet| {
                let mut context = PacketKernelContext {
                    meter: None,
                    scratch_limit: None,
                    result_limit: None,
                    scratch_bytes: 0,
                    result_bytes: 0,
                };
                kernel(&packet.value, &mut context)
                    .unwrap_or_else(|_| panic!("unleased packet kernel has no execution ceiling"))
            })
            .collect());
    }
    let partitions = packets
        .into_iter()
        .map(|packet| MapPartition {
            identity: packet.identity,
            value: ChargedPacketInput {
                value: packet.value,
                owned_bytes: packet.input_bytes,
                scratch_limit: packet.kernel_scratch_bytes,
                result_limit: packet.max_result_bytes,
            },
            read_keys: Vec::<u64>::new(),
            write_keys: Vec::<u64>::new(),
            kernel_scratch_bytes: packet.kernel_scratch_bytes,
            max_result_bytes: packet.max_result_bytes,
        })
        .collect();
    let batch = ExecutionMap::try_from_declared_partitions(identities, partitions)
        .map_err(PacketExecutionStop::Admission)?;
    let outcome = super::run_with_remaining_request_work(
        lease.expect("checked above"),
        budget,
        |lease| {
            batch.run(Some(lease), |packet, context| {
                context.checkpoint(1)?;
                let mut packet_context = PacketKernelContext {
                    meter: Some(context),
                    scratch_limit: Some(packet.scratch_limit),
                    result_limit: Some(packet.result_limit),
                    scratch_bytes: 0,
                    result_bytes: 0,
                };
                let value = kernel(&packet.value, &mut packet_context)?;
                Ok(ChargedPacketResult {
                    owned_bytes: charged_bytes(&value),
                    value,
                })
            })
        },
        |outcome| match outcome {
            MapOutcome::Complete { report, .. } | MapOutcome::Stopped { report, .. } => *report,
        },
    );
    match outcome {
        MapOutcome::Complete { values, .. } => {
            Ok(values.into_iter().map(|result| result.value).collect())
        }
        MapOutcome::Stopped {
            boundary, reason, ..
        } => Err(PacketExecutionStop::Execution { boundary, reason }),
    }
}

#[cfg(test)]
mod tests;

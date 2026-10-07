use std::mem::size_of;

use worth_execution::{ChargedBytes, ExecutionResourceLease, MapKernelFailure, ScanOutcome};
use worth_foundational::PartitionIdentity;

use super::packet_for_definition;
use crate::authority::commit::preparation::packets::index::IndexPreparationPacket;
use crate::execution::{
    PacketBudgetDenial, PacketExecutionStop, ReadOnlyPacket, RequestWorkBudget,
};
use crate::indexes::data::DerivedIndexId;
use crate::runtime::RelationalRuntime;

pub(in crate::indexes::authority) struct PlannedIndexPackets {
    pub inputs: Vec<ReadOnlyPacket<IndexPreparationPacket>>,
    pub present: Vec<DerivedIndexId>,
    pub missing: Vec<DerivedIndexId>,
}

impl ChargedBytes for PlannedIndexPackets {
    fn additional_charged_bytes(&self) -> u64 {
        (self.inputs.capacity() as u64)
            .saturating_mul(size_of::<ReadOnlyPacket<IndexPreparationPacket>>() as u64)
            .saturating_add(
                self.inputs
                    .iter()
                    .map(|input| input.input_bytes)
                    .fold(0_u64, u64::saturating_add),
            )
            .saturating_add(
                ((self.present.capacity() + self.missing.capacity()) as u64)
                    .saturating_mul(size_of::<DerivedIndexId>() as u64),
            )
    }
}

pub(in crate::indexes::authority) fn plan_index_packets_checked(
    runtime: &RelationalRuntime,
    index_ids: &[DerivedIndexId],
    lease: &ExecutionResourceLease<'_>,
    work_budget: &RequestWorkBudget,
) -> Result<PlannedIndexPackets, PacketExecutionStop> {
    let identity = PartitionIdentity::new(1);
    let scan = crate::execution::admit_ordered_scan(vec![(identity, ())])?;
    let ceiling = lease.policy().budget().charged_memory_bytes() / 8;
    let outcome = crate::execution::run_with_remaining_request_work(
        lease,
        Some(work_budget),
        |child| {
            scan.run(Some(child), (), 0, ceiling, 0, ceiling, |_, _, context| {
                context.checkpoint(0)?;
                let descriptor_bytes = (index_ids.len() as u64).saturating_mul(
                    (size_of::<ReadOnlyPacket<IndexPreparationPacket>>()
                        + 2 * size_of::<DerivedIndexId>()
                        + 3 * size_of::<PartitionIdentity>()) as u64,
                );
                if descriptor_bytes > ceiling {
                    return Err(MapKernelFailure::<PacketBudgetDenial>::ResultCapacityExceeded);
                }
                let mut claimed = descriptor_bytes;
                let mut inputs = Vec::new();
                let mut present = Vec::new();
                let mut missing = Vec::new();
                inputs
                    .try_reserve_exact(index_ids.len())
                    .map_err(|_| MapKernelFailure::<PacketBudgetDenial>::ResultCapacityExceeded)?;
                present
                    .try_reserve_exact(index_ids.len())
                    .map_err(|_| MapKernelFailure::<PacketBudgetDenial>::ResultCapacityExceeded)?;
                missing
                    .try_reserve_exact(index_ids.len())
                    .map_err(|_| MapKernelFailure::<PacketBudgetDenial>::ResultCapacityExceeded)?;
                for index_id in index_ids.iter().copied() {
                    context.checkpoint(1).map_err(MapKernelFailure::Stop)?;
                    if let Some(definition) = runtime.indexes.definition(index_id) {
                        let owned = definition.owned_allocation_capacity_bytes();
                        claimed = claimed
                            .checked_add(owned)
                            .filter(|total| *total <= ceiling)
                            .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
                        let packet_index = present.len();
                        present.push(index_id);
                        inputs.push(ReadOnlyPacket {
                            identity: PartitionIdentity::new(packet_index as u64),
                            input_bytes: owned,
                            kernel_scratch_bytes: 0,
                            max_result_bytes: 0,
                            value: packet_for_definition(packet_index, definition.as_ref().clone()),
                        });
                    } else {
                        missing.push(index_id);
                    }
                }
                let sort_work = (inputs.len() as u64).saturating_mul(
                    (usize::BITS - inputs.len().saturating_sub(1).leading_zeros()) as u64,
                );
                context
                    .checkpoint(sort_work)
                    .map_err(MapKernelFailure::Stop)?;
                inputs.sort_by_key(|input| {
                    (
                        input.value.definition.index_id.0,
                        input.value.header.packet_index,
                    )
                });
                let packet_ceiling = lease.policy().budget().charged_memory_bytes()
                    / (inputs.len().max(1) as u64).saturating_mul(8);
                for (ordinal, input) in inputs.iter_mut().enumerate() {
                    input.identity = PartitionIdentity::new(ordinal as u64);
                    input.kernel_scratch_bytes = packet_ceiling;
                    input.max_result_bytes = packet_ceiling;
                }
                Ok((
                    (),
                    PlannedIndexPackets {
                        inputs,
                        present,
                        missing,
                    },
                ))
            })
        },
        ScanOutcome::report,
    );
    match outcome {
        ScanOutcome::Complete { mut prefixes, .. } => {
            Ok(prefixes.pop().expect("one index planning output"))
        }
        ScanOutcome::Stopped {
            boundary, reason, ..
        } => Err(PacketExecutionStop::Execution { boundary, reason }),
    }
}

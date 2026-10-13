use std::collections::BTreeSet;
use std::mem::size_of;

use worth_execution::{ExecutionResourceLease, MapKernelFailure};
use worth_foundational::PartitionIdentity;

use crate::authority::commit::preparation::facade::PreparedInvariantExecution;
use crate::authority::commit::preparation::proofs::locality::{
    PreparationPartitionScope, PreparationReadSetApproximation,
};
use crate::execution::{
    prepare_borrowed_packets, PacketBudgetDenial, PacketExecutionStop, PacketPreparationBudget,
    ReadOnlyPacket, RequestWorkBudget,
};
use crate::identity::data::PartitionId;
use crate::validation::engine::{
    InvariantPlanScopeClass, InvariantProofBoundarySummary, InvariantScopeWideningCause,
};

pub(crate) struct CheckedPacketPlan<'state> {
    pub(crate) packets:
        Vec<ReadOnlyPacket<crate::authority::commit::preparation::InvariantWorkPacket<'state>>>,
    pub(crate) proof_boundary: InvariantProofBoundarySummary,
    pub(crate) scope_units: usize,
}

pub(crate) fn prepare_checked_invariant_packets<'state>(
    planned: PreparedInvariantExecution<'state>,
    lease: &ExecutionResourceLease<'_>,
    work_budget: Option<&RequestWorkBudget>,
) -> Result<CheckedPacketPlan<'state>, PacketExecutionStop> {
    let mut metadata = None;
    let (packets, _) = prepare_borrowed_packets(lease, work_budget, |budget| {
        let (proof_boundary, scope_units) = fold_scope(&planned, budget)?;
        metadata = Some((proof_boundary, scope_units));
        let mut packets = Vec::new();
        for packet in planned.packets {
            budget
                .claim_packet::<crate::authority::commit::preparation::InvariantWorkPacket<'_>>()?;
            packets
                .try_reserve(1)
                .map_err(|_| MapKernelFailure::ResultCapacityExceeded)?;
            packets.push(ReadOnlyPacket {
                identity: PartitionIdentity::new(packet.packet_index as u64),
                // Nested inputs retain the live planning owner's custody.
                input_bytes: 0,
                // Fixed input/framework storage is admitted by the map.
                // Invariant allocations reserve on that map's child lease.
                kernel_scratch_bytes: 0,
                max_result_bytes: 0,
                value: packet,
            });
        }
        Ok(packets)
    })?;
    let (proof_boundary, scope_units) = metadata.expect("checked scope fold completed");
    Ok(CheckedPacketPlan {
        packets,
        proof_boundary,
        scope_units,
    })
}

fn fold_scope(
    planned: &PreparedInvariantExecution<'_>,
    budget: &mut PacketPreparationBudget<'_, '_, '_>,
) -> Result<(InvariantProofBoundarySummary, usize), MapKernelFailure<PacketBudgetDenial>> {
    let mut touched = BTreeSet::new();
    let mut widened = Vec::new();
    let mut touched_only = false;
    let mut all_observed = false;
    budget.claim((2 * size_of::<InvariantScopeWideningCause>()) as u64)?;
    widened
        .try_reserve(2)
        .map_err(|_| MapKernelFailure::ResultCapacityExceeded)?;
    for packet in &planned.packets {
        budget.checkpoint(1)?;
        match &packet.locality.partition_scope {
            PreparationPartitionScope::AllObserved => {
                all_observed = true;
                push_cause(
                    &mut widened,
                    InvariantScopeWideningCause::AllObservedPartitionScope,
                );
            }
            PreparationPartitionScope::TouchedPartitions(partitions) => {
                for partition in partitions.iter() {
                    budget.checkpoint(1)?;
                    if !touched.contains(partition) {
                        // Declared BTreeSet node payload and links; allocator
                        // metadata is outside the repository's byte model.
                        budget.claim((size_of::<PartitionId>() + 4 * size_of::<usize>()) as u64)?;
                        touched.insert(*partition);
                    }
                }
            }
        }
        match packet.locality.read_set_approximation {
            PreparationReadSetApproximation::FullObservedScan => {
                push_cause(
                    &mut widened,
                    InvariantScopeWideningCause::FullObservedReadSet,
                );
            }
            PreparationReadSetApproximation::TouchedOnly => touched_only = true,
            PreparationReadSetApproximation::SharedCommittedRead => {}
        }
    }
    let scope_class = if !widened.is_empty() {
        InvariantPlanScopeClass::BroaderScope
    } else if touched_only {
        InvariantPlanScopeClass::TouchedScope
    } else {
        InvariantPlanScopeClass::PartitionScope
    };
    let scope_units = if all_observed { 1 } else { touched.len() };
    Ok((
        InvariantProofBoundarySummary::new(
            scope_class,
            widened,
            planned.packets.len(),
            touched.len(),
        ),
        scope_units,
    ))
}

fn push_cause(causes: &mut Vec<InvariantScopeWideningCause>, cause: InvariantScopeWideningCause) {
    if !causes.contains(&cause) {
        causes.push(cause);
    }
}

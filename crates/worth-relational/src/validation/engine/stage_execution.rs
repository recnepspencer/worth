//! One selected stage through the existing planners and packet runners.
use super::{
    engine::check_live, metrics::InvariantMetrics, InvariantExecutionRequest, InvariantRuntimeView,
};
use crate::validation::execution::{
    evaluate_invariant_packet, plan_invariant_execution, planned_proof_boundary_summary,
    InvariantStage, InvariantStageOutput,
};
use worth_execution::ExecutionResourceLease;
use worth_foundational::PartitionIdentity;

pub(super) fn execute_stage<'state>(
    runtime: &InvariantRuntimeView<'state>,
    execution: worth_execution::ExecutionRequest<'state, 'state>,
    request: &'state InvariantExecutionRequest<'state>,
    lease: Option<&ExecutionResourceLease<'_>>,
    stage: InvariantStage,
) -> Result<Option<InvariantStageOutput>, crate::transactions::data::TransactionCommitError> {
    let metrics = InvariantMetrics::new(runtime.performance_access());
    let (planned, retained_bytes) = if let Some(lease) = lease {
        let preparation = crate::validation::execution::plan_checked_invariant_preparation(
            runtime, execution, request, lease, stage,
        );
        check_live(execution)?;
        let (planned, retained_bytes, _) = preparation?;
        (planned, retained_bytes)
    } else {
        (
            plan_invariant_execution(runtime, execution, request, None, stage),
            0,
        )
    };
    check_live(execution)?;
    if planned.packets.is_empty() {
        return Ok(None);
    }
    let strategy = planned.strategy;
    let (proof_boundary, packets) = if let Some(lease) = lease {
        let counters = crate::validation::execution::planned_packet_counters(&planned);
        let checked = crate::validation::execution::prepare_checked_invariant_packets(
            planned,
            lease,
            runtime.commit_work_budget.as_ref(),
            retained_bytes,
        );
        check_live(execution)?;
        let checked = checked?;
        metrics.record_preparation_shape(counters.packet_count, checked.scope_units, strategy);
        (checked.proof_boundary, checked.packets)
    } else {
        metrics.record_preparation_plan(&planned);
        let proof_boundary = planned_proof_boundary_summary(&planned);
        let packets = planned.packets.into_iter().map(|packet| {
                let scope_bytes = match &packet.locality.partition_scope {
                    crate::authority::commit::preparation::proofs::locality::PreparationPartitionScope::AllObserved => 0,
                    crate::authority::commit::preparation::proofs::locality::PreparationPartitionScope::TouchedPartitions(partitions) => {
                        (partitions.len() as u64).saturating_mul(std::mem::size_of::<crate::identity::data::PartitionId>() as u64)
                    }
                };
                crate::execution::ReadOnlyPacket {
                    identity: PartitionIdentity::new(packet.packet_index as u64),
                    input_bytes: scope_bytes
                        .saturating_add((packet.reduction_key.partition_scope.len() as u64)
                            .saturating_mul(std::mem::size_of::<crate::identity::data::PartitionId>() as u64)),
                    memory: crate::execution::PacketMemory::Declared { scratch: 0, result: 0 },
                    value: packet,
                }
            }).collect();
        (proof_boundary, packets)
    };
    crate::execution::admit_read_only_packet_order(&packets)
        .map_err(crate::execution::PacketExecutionStop::Admission)?;
    let envelopes = if let Some(lease) = lease {
        crate::execution::execute_read_only_packets_with_budget(
            packets,
            lease,
            runtime.commit_work_budget.as_ref(),
            |packet, context| {
                crate::validation::execution::evaluate_invariant_packet_checked(
                    runtime,
                    execution,
                    packet,
                    context,
                    lease.status(),
                )
            },
            crate::validation::execution::InvariantWorkerEnvelope::owned_allocation_capacity_bytes,
        )
    } else {
        crate::execution::execute_unleased_read_only_packets(&packets, |packet| {
            execution
                .check_live()
                .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
            let envelope = evaluate_invariant_packet(runtime, execution, packet);
            execution
                .check_live()
                .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
            Ok(envelope)
        })
        .map_err(crate::execution::PacketExecutionStop::from)
    };
    check_live(execution)?;
    let envelopes = envelopes?;
    Ok(Some(InvariantStageOutput {
        strategy,
        proof_boundary,
        envelopes,
    }))
}

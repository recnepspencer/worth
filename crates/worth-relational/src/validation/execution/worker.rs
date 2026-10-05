use crate::authority::commit::preparation::diagnostics::counters::ValidationPreparationCounters;
use crate::authority::commit::preparation::diagnostics::failures::PreparationFailureClass;
use crate::authority::commit::preparation::proofs::kinds::PreparationProofKind;
use crate::authority::commit::preparation::proofs::locality::{
    PreparationPartitionScope, PreparationReadSetApproximation, PreparationRecordDomain,
    PreparationWriteExclusionClass,
};
use crate::validation::engine::budget::{
    InvariantBudget, InvariantBudgetStop, LeasedInvariantBudget,
};
use crate::validation::engine::InvariantRuntimeView;

use super::envelope::InvariantWorkerEnvelope;
use super::packets::InvariantWorkPacket;

mod registered_rule;
mod verdict_reduction;

pub(crate) fn evaluate_invariant_packet<'state>(
    runtime: &InvariantRuntimeView<'state>,
    packet: &InvariantWorkPacket<'state>,
) -> InvariantWorkerEnvelope {
    evaluate_invariant_packet_inner(runtime, packet, None)
}

pub(crate) fn evaluate_invariant_packet_checked<'state>(
    runtime: &InvariantRuntimeView<'state>,
    packet: &InvariantWorkPacket<'state>,
    context: &mut crate::execution::PacketKernelContext<'_, '_, '_>,
    status: worth_execution::ExecutionLeaseStatus,
) -> Result<InvariantWorkerEnvelope, InvariantBudgetStop> {
    let budget = LeasedInvariantBudget::new(context, status);
    let envelope = evaluate_invariant_packet_inner(runtime, packet, Some(&budget));
    if let Some(failure) = budget.take_failure() {
        return Err(failure);
    }
    if !budget.ensure_result_total(envelope.owned_allocation_capacity_bytes()) {
        return Err(budget.take_failure().expect("result claim recorded stop"));
    }
    Ok(envelope)
}

fn evaluate_invariant_packet_inner<'state>(
    runtime: &InvariantRuntimeView<'state>,
    packet: &InvariantWorkPacket<'state>,
    budget: Option<&dyn InvariantBudget>,
) -> InvariantWorkerEnvelope {
    let preparation_failures = invariant_packet_failures(packet);
    debug_assert!(
        preparation_failures.is_empty(),
        "planned invariant packet violated preparation proof contract: {:?}",
        preparation_failures
    );
    let evaluation = registered_rule::evaluate_registered_rule(runtime, packet, budget);
    let reduced = verdict_reduction::reduce_invariant_verdicts(packet, evaluation);

    let worker_result_count = reduced.results.len();
    InvariantWorkerEnvelope {
        packet_index: packet.packet_index,
        reduction_key: packet.reduction_key.clone(),
        results: reduced.results,
        diagnostic_observations: reduced.diagnostic_observations,
        preparation_failures: preparation_failures.clone(),
        counters: ValidationPreparationCounters {
            packet_count: 1,
            worker_result_count,
            reducer_input_count: 0,
            reducer_conflict_count: 0,
            failure_count: preparation_failures.len(),
        },
    }
}

fn invariant_packet_failures(packet: &InvariantWorkPacket<'_>) -> Vec<PreparationFailureClass> {
    let mut failures = Vec::new();
    if packet.validity.context != packet.planning_context {
        failures.push(PreparationFailureClass::PlanningProofInsufficient);
    }
    if packet.planning_context.execution_point != packet.registration.execution_point()
        || packet.planning_context.observation_kind != packet.observation.kind()
        || packet.locality.observation_scope != packet.observation.kind()
        || packet.locality.invariant_group_scope != packet.registration.groups()
    {
        failures.push(PreparationFailureClass::PlanningProofInsufficient);
    }
    match packet.locality.partition_scope {
        PreparationPartitionScope::AllObserved
        | PreparationPartitionScope::TouchedPartitions(_) => {}
    }
    match packet.locality.read_set_approximation {
        PreparationReadSetApproximation::SharedCommittedRead
        | PreparationReadSetApproximation::TouchedOnly
        | PreparationReadSetApproximation::FullObservedScan => {}
    }
    match packet.locality.record_domain {
        PreparationRecordDomain::Mixed | PreparationRecordDomain::None => {}
        PreparationRecordDomain::Entity | PreparationRecordDomain::Relation => {}
    }
    match packet.proof_kind {
        PreparationProofKind::RequiresSerial => {
            if packet.locality.write_exclusion
                != PreparationWriteExclusionClass::RequiresSingleLaneExecution
            {
                failures.push(PreparationFailureClass::PublicationIsolationViolation);
            }
        }
        _ => {
            if packet.locality.write_exclusion != PreparationWriteExclusionClass::ReadOnly {
                failures.push(PreparationFailureClass::PublicationIsolationViolation);
            }
        }
    }
    failures
}

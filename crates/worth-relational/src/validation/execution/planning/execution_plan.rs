use std::sync::Arc;

use crate::authority::commit::preparation::facade::PreparedInvariantExecution;
use crate::authority::commit::preparation::packets::invariant::InvariantPacketRegistration;
use crate::authority::commit::preparation::planning::context::PreparationPlanningContext;
use crate::authority::commit::preparation::planning::strategy::{
    packet_width_is_profitable, ParallelLegality, ParallelProfitability, PreparationStrategy,
    PreparationStrategySelection, SerialPreparationReason, MIN_PARALLEL_PACKET_WIDTH,
};
use crate::authority::commit::preparation::proofs::kinds::PreparationProofKind;
use crate::authority::commit::preparation::proofs::locality::{
    PreparationLocalityProof, PreparationPartitionScope, PreparationReadSetApproximation,
    PreparationRecordDomain, PreparationWriteExclusionClass,
};
use crate::authority::commit::preparation::proofs::validity::PreparationProofValidity;
use crate::authority::commit::preparation::reduction::keys::ValidationReductionKey;
use crate::validation::engine::InvariantExecutionRequest;
use crate::validation::engine::InvariantRuntimeView;
use worth_execution::ExecutionResourceLease;

use super::packet_scope::{packet_partition_scope, packet_partition_scope_checked};
use super::packet_selection::eligible_registrations;

pub(crate) fn plan_invariant_execution<'state>(
    runtime: &InvariantRuntimeView<'state>,
    request: &'state InvariantExecutionRequest<'state>,
    lease: Option<&ExecutionResourceLease>,
) -> PreparedInvariantExecution<'state> {
    plan_invariant_execution_inner(runtime, request, lease, None)
}

pub(crate) fn plan_invariant_execution_checked<'state>(
    runtime: &InvariantRuntimeView<'state>,
    request: &'state InvariantExecutionRequest<'state>,
    lease: &ExecutionResourceLease,
    budget: &crate::validation::custom_rule::CustomPreparationBudget,
) -> PreparedInvariantExecution<'state> {
    plan_invariant_execution_inner(runtime, request, Some(lease), Some(budget))
}

fn plan_invariant_execution_inner<'state>(
    runtime: &InvariantRuntimeView<'state>,
    request: &'state InvariantExecutionRequest<'state>,
    lease: Option<&ExecutionResourceLease>,
    budget: Option<&crate::validation::custom_rule::CustomPreparationBudget>,
) -> PreparedInvariantExecution<'state> {
    let registrations = eligible_registrations(runtime, request, budget);
    let context = Arc::new(planning_context(runtime, request));
    if budget.is_some_and(|budget| budget.stop().is_some()) {
        return PreparedInvariantExecution {
            context,
            strategy: PreparationStrategy::serial(SerialPreparationReason::NoLease),
            packets: Vec::new(),
        };
    }
    let partition_scope = if let Some(budget) = budget {
        packet_partition_scope_checked(request.merged_plan(), budget)
    } else {
        packet_partition_scope(request.merged_plan())
    };
    let proof_kind = proof_kind_for_runtime(lease);
    let strategy = preparation_strategy_for_runtime(lease, registrations.len(), proof_kind);
    let packets = invariant_work_packets(
        request,
        registrations,
        context.clone(),
        partition_scope,
        proof_kind,
        budget,
    );

    PreparedInvariantExecution {
        context,
        strategy,
        packets,
    }
}

fn planning_context(
    runtime: &InvariantRuntimeView,
    request: &InvariantExecutionRequest<'_>,
) -> PreparationPlanningContext {
    PreparationPlanningContext {
        transaction_id: request.merged_plan().map(|plan| plan.transaction_id),
        execution_point: request.execution_point(),
        observation_kind: request.observation().kind(),
        version_id: request.version_id(),
        current_version_id: request.current_version_id(),
        structural_summary: None,
        plan_contract: request.plan_contract(),
        schema_registry_entry_count: runtime.config.schema.registry.entity_kinds.len()
            + runtime.config.schema.registry.relation_kinds.len(),
        invariant_registration_count: runtime.config.schema.invariant_catalog.registrations.len()
            + runtime
                .schema_contract_runtime
                .relation_integrity_registrations
                .len()
            + runtime
                .schema_contract_runtime
                .custom_invariant_registries
                .len(),
    }
}

fn proof_kind_for_runtime(lease: Option<&ExecutionResourceLease>) -> PreparationProofKind {
    if lease.is_some_and(|lease| {
        lease.resolved_posture() == worth_foundational::ExecutionPosture::Automatic
    }) {
        PreparationProofKind::ReadOnlyShared
    } else {
        PreparationProofKind::RequiresSerial
    }
}

fn preparation_strategy_for_runtime(
    lease: Option<&ExecutionResourceLease>,
    packet_count: usize,
    proof_kind: PreparationProofKind,
) -> PreparationStrategy {
    if lease.is_none() {
        return PreparationStrategy::serial(SerialPreparationReason::NoLease);
    }
    if lease.is_some_and(|lease| {
        lease.resolved_posture() == worth_foundational::ExecutionPosture::Serial
    }) {
        return PreparationStrategy::serial(SerialPreparationReason::SerialPosture);
    }
    if !packet_width_is_profitable(packet_count, MIN_PARALLEL_PACKET_WIDTH) {
        return PreparationStrategy {
            parallel_legality: ParallelLegality::ProvenParallel,
            parallel_profitability: ParallelProfitability::NotProfitable,
            selected_mode: PreparationStrategySelection::Serial,
            serial_selection_reason: Some(SerialPreparationReason::InsufficientPacketBreadth),
        };
    }
    if proof_kind == PreparationProofKind::RequiresSerial {
        return PreparationStrategy::serial(SerialPreparationReason::ProofRequiresSerial);
    }

    PreparationStrategy {
        parallel_legality: ParallelLegality::ProvenParallel,
        parallel_profitability: ParallelProfitability::Profitable,
        selected_mode: PreparationStrategySelection::StagedParallel,
        serial_selection_reason: None,
    }
}

fn invariant_work_packets<'state>(
    request: &'state InvariantExecutionRequest<'state>,
    registrations: Vec<InvariantPacketRegistration>,
    context: Arc<PreparationPlanningContext>,
    partition_scope: Arc<[crate::identity::data::PartitionId]>,
    proof_kind: PreparationProofKind,
    budget: Option<&crate::validation::custom_rule::CustomPreparationBudget>,
) -> Vec<crate::authority::commit::preparation::InvariantWorkPacket<'state>> {
    let observation = request.observation();
    let relation_integrity_scopes = request.relation_integrity_scopes().cloned();
    let current_version_minimum_index = Arc::new(std::sync::OnceLock::new());

    registrations
        .into_iter()
        .enumerate()
        .take_while(|_| budget.is_none_or(|budget| budget.stop().is_none()))
        .filter_map(|(packet_index, registration)| {
            if budget.is_some_and(|budget| {
                !budget.try_plan_item(
                    2 * std::mem::size_of::<
                        crate::authority::commit::preparation::InvariantWorkPacket<'_>,
                    >() as u64,
                )
            }) {
                return None;
            }
            let invariant_group_scope = registration.groups();
            let record_domain = if request.merged_plan().is_some() {
                PreparationRecordDomain::Mixed
            } else {
                PreparationRecordDomain::None
            };
            let locality = PreparationLocalityProof {
                observation_scope: request.observation().kind(),
                record_domain,
                partition_scope: if partition_scope.is_empty() {
                    PreparationPartitionScope::AllObserved
                } else {
                    PreparationPartitionScope::TouchedPartitions(partition_scope.clone())
                },
                invariant_group_scope,
                read_set_approximation: PreparationReadSetApproximation::SharedCommittedRead,
                write_exclusion: match proof_kind {
                    PreparationProofKind::RequiresSerial => {
                        PreparationWriteExclusionClass::RequiresSingleLaneExecution
                    }
                    _ => PreparationWriteExclusionClass::ReadOnly,
                },
            };
            Some(crate::authority::commit::preparation::InvariantWorkPacket {
                packet_index,
                registration,
                reduction_key: ValidationReductionKey::new(
                    request.execution_point(),
                    observation.kind(),
                    partition_scope.clone(),
                    invariant_group_scope,
                    packet_index,
                ),
                proof_kind,
                locality,
                validity: PreparationProofValidity {
                    context: context.clone(),
                },
                planning_context: context.clone(),
                observation,
                version_id: request.version_id(),
                current_version_id: request.current_version_id(),
                merged_plan: request.merged_plan(),
                relation_integrity_scopes: relation_integrity_scopes.clone(),
                current_version_minimum_index: Arc::clone(&current_version_minimum_index),
            })
        })
        .collect()
}

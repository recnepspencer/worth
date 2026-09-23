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
use crate::config::data::RelationalExecutionModel;
use crate::validation::engine::InvariantExecutionRequest;
use crate::validation::engine::InvariantRuntimeView;

use super::packet_scope::packet_partition_scope;
use super::packet_selection::eligible_registrations;

pub(crate) fn plan_invariant_execution<'runtime, 'state>(
    runtime: &'runtime InvariantRuntimeView,
    request: &'state InvariantExecutionRequest<'state>,
) -> PreparedInvariantExecution<'state>
where
    'runtime: 'state,
{
    let registrations = eligible_registrations(runtime, request);
    let context = Arc::new(planning_context(runtime, request));
    let partition_scope = packet_partition_scope(request.merged_plan());
    let proof_kind = proof_kind_for_runtime(runtime);
    let strategy = preparation_strategy_for_runtime(runtime, registrations.len(), proof_kind);
    let packets = invariant_work_packets(
        request,
        registrations,
        context.clone(),
        partition_scope,
        proof_kind,
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
        planning_contract: runtime.config.execution.planning.clone(),
    }
}

fn proof_kind_for_runtime(runtime: &InvariantRuntimeView) -> PreparationProofKind {
    if matches!(
        runtime.config.execution.execution_model,
        RelationalExecutionModel::ParallelPreparation
    ) {
        PreparationProofKind::ReadOnlyShared
    } else {
        PreparationProofKind::RequiresSerial
    }
}

fn preparation_strategy_for_runtime(
    runtime: &InvariantRuntimeView,
    packet_count: usize,
    proof_kind: PreparationProofKind,
) -> PreparationStrategy {
    if !matches!(
        runtime.config.execution.execution_model,
        RelationalExecutionModel::ParallelPreparation
    ) {
        return PreparationStrategy::serial(SerialPreparationReason::ExecutionModelSerial);
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
) -> Vec<crate::authority::commit::preparation::InvariantWorkPacket<'state>> {
    let observation = request.observation();
    let relation_integrity_scopes = request.relation_integrity_scopes().cloned();
    let current_version_minimum_index = Arc::new(std::sync::OnceLock::new());

    registrations
        .into_iter()
        .enumerate()
        .map(|(packet_index, registration)| {
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
            crate::authority::commit::preparation::InvariantWorkPacket {
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
            }
        })
        .collect()
}

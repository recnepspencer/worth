use crate::validation::engine::InvariantRuntimeView;
use crate::validation::execution::{
    evaluate_invariant_packet, plan_invariant_execution, planned_proof_boundary_summary,
};
use crate::validation::reduction::{
    reduce_invariant_execution, reduce_invariant_execution_checked,
};
use std::collections::BTreeSet;
use worth_execution::ExecutionResourceLease;
use worth_foundational::PartitionIdentity;

use super::request::InvariantExecutionRequest;
use super::result::InvariantExecutionResult;

pub(crate) struct InvariantEngine<'runtime> {
    runtime: InvariantRuntimeView<'runtime>,
}

impl<'runtime> InvariantEngine<'runtime> {
    pub(crate) fn from_view(runtime: &InvariantRuntimeView<'runtime>) -> Self {
        Self {
            runtime: runtime.clone(),
        }
    }

    #[cfg(test)]
    pub(crate) fn new(runtime: &'runtime crate::runtime::RelationalRuntime) -> Self {
        Self {
            runtime: InvariantRuntimeView::from_runtime(runtime),
        }
    }

    pub(crate) fn execute<'state>(
        &self,
        request: InvariantExecutionRequest<'state>,
    ) -> InvariantExecutionResult
    where
        'runtime: 'state,
    {
        self.execute_inner(request, None)
            .expect("serial invariant packet plan must be canonical")
    }

    pub(crate) fn execute_with_lease<'state>(
        &self,
        request: InvariantExecutionRequest<'state>,
        lease: &ExecutionResourceLease<'_>,
    ) -> Result<InvariantExecutionResult, crate::transactions::data::TransactionCommitError>
    where
        'runtime: 'state,
    {
        self.execute_inner(request, Some(lease))
    }

    fn execute_inner<'state>(
        &self,
        request: InvariantExecutionRequest<'state>,
        lease: Option<&ExecutionResourceLease<'_>>,
    ) -> Result<InvariantExecutionResult, crate::transactions::data::TransactionCommitError>
    where
        'runtime: 'state,
    {
        if lease.is_some()
            && self
                .runtime
                .schema_contract_runtime
                .custom_invariant_registries
                .iter()
                .filter(|registration| request.includes_custom_registration(registration))
                .any(|registration| !registration.supports_checked_preparation())
        {
            return Err(
                crate::transactions::data::TransactionCommitError::execution(
                    crate::transactions::data::CommitExecutionDenial {
                        kind: crate::transactions::data::CommitExecutionDenialKind::Cause(
                            crate::execution::RelationalExecutionDenialCause::UncheckedCustomKernel,
                        ),
                        partition_identity: None,
                    },
                ),
            );
        }
        let runtime = self.runtime.with_candidate_input_plan(&request);
        let (planned, retained_bytes) = if let Some(lease) = lease {
            let (planned, retained_bytes, _) =
                crate::validation::execution::plan_checked_invariant_preparation(
                    &runtime, &request, lease,
                )?;
            (planned, retained_bytes)
        } else {
            (plan_invariant_execution(&runtime, &request, None), 0)
        };
        let strategy = planned.strategy;
        let (proof_boundary, packets) = if let Some(lease) = lease {
            let counters = crate::validation::execution::planned_packet_counters(&planned);
            let checked = crate::validation::execution::prepare_checked_invariant_packets(
                planned,
                lease,
                runtime.commit_work_budget.as_ref(),
                retained_bytes,
            )?;
            self.record_preparation_shape(counters.packet_count, checked.scope_units, strategy);
            (checked.proof_boundary, checked.packets)
        } else {
            self.record_preparation_plan(&planned);
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
                    kernel_scratch_bytes: 0,
                    max_result_bytes: 0,
                    value: packet,
                }
            }).collect();
            (proof_boundary, packets)
        };
        let envelopes = crate::execution::execute_read_only_packets_with_budget(
            packets,
            lease,
            runtime.commit_work_budget.as_ref(),
            |packet, context| {
                if let Some(lease) = lease {
                    crate::validation::execution::evaluate_invariant_packet_checked(
                        &runtime,
                        packet,
                        context,
                        lease.status(),
                    )
                } else {
                    Ok(evaluate_invariant_packet(&runtime, packet))
                }
            },
            crate::validation::execution::InvariantWorkerEnvelope::owned_allocation_capacity_bytes,
        )?;
        let (result, _, reducer_conflicts) = if let Some(lease) = lease {
            reduce_invariant_execution_checked(
                &request,
                strategy,
                proof_boundary,
                envelopes,
                lease,
                runtime.commit_work_budget.as_ref(),
            )?
        } else {
            reduce_invariant_execution(&request, strategy, proof_boundary, envelopes)
        };
        if let Some(inputs) = runtime.shared_candidate_inputs() {
            runtime
                .performance_access()
                .count_custom_invariant_candidate_inputs(inputs.counters());
        }
        if !reducer_conflicts.is_empty() {
            self.runtime
                .performance_access()
                .count_preparation_reducer_conflicts(reducer_conflicts.len());
        }
        Ok(result)
    }
}

impl InvariantEngine<'_> {
    fn record_preparation_plan(
        &self,
        planned: &crate::authority::commit::preparation::facade::PreparedInvariantExecution<'_>,
    ) {
        let counters = crate::validation::execution::planned_packet_counters(planned);
        let scope_units = if planned.packets.iter().any(|packet| {
            matches!(
                packet.locality.partition_scope,
                crate::authority::commit::preparation::proofs::locality::PreparationPartitionScope::AllObserved
            )
        }) {
            1
        } else {
            let mut touched = BTreeSet::new();
            for packet in &planned.packets {
                if let crate::authority::commit::preparation::proofs::locality::PreparationPartitionScope::TouchedPartitions(
                    partitions,
                ) = &packet.locality.partition_scope
                {
                    touched.extend(partitions.iter().copied());
                }
            }
            touched.len()
        };
        self.record_preparation_shape(counters.packet_count, scope_units, planned.strategy);
        debug_assert!(planned
            .packets
            .iter()
            .all(|packet| packet.planning_context == planned.context));
    }

    fn record_preparation_shape(
        &self,
        packet_count: usize,
        scope_units: usize,
        strategy: crate::authority::commit::preparation::planning::strategy::PreparationStrategy,
    ) {
        let performance = self.runtime.performance_access();
        performance.count_preparation_packet_shape(
            packet_count,
            packet_count,
            usize::from(packet_count > 0),
            scope_units,
        );
        match strategy.parallel_legality {
            crate::authority::commit::preparation::planning::strategy::ParallelLegality::ProvenParallel => {
                performance.count_preparation_parallel_legal();
            }
            crate::authority::commit::preparation::planning::strategy::ParallelLegality::RequiresSerial => {}
        }
        match strategy.parallel_profitability {
            crate::authority::commit::preparation::planning::strategy::ParallelProfitability::Profitable => {
                performance.count_preparation_parallel_profitable();
            }
            crate::authority::commit::preparation::planning::strategy::ParallelProfitability::NotProfitable => {}
        }
        match strategy.selected_mode {
            crate::authority::commit::preparation::planning::strategy::PreparationStrategySelection::Serial => {
                performance.count_preparation_serial_strategy();
            }
            crate::authority::commit::preparation::planning::strategy::PreparationStrategySelection::StagedParallel => {
                performance.count_preparation_staged_parallel_strategy();
            }
        }
    }
}

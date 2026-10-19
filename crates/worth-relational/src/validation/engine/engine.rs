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
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: InvariantExecutionRequest<'state>,
    ) -> Result<InvariantExecutionResult, crate::transactions::data::TransactionCommitError>
    where
        'runtime: 'state,
    {
        self.execute_inner(*execution, request)
    }

    fn execute_inner<'state>(
        &self,
        execution: worth_execution::ExecutionRequest<'state, 'state>,
        request: InvariantExecutionRequest<'state>,
    ) -> Result<InvariantExecutionResult, crate::transactions::data::TransactionCommitError>
    where
        'runtime: 'state,
    {
        check_live(execution)?;
        let lease = crate::authority::commit::pipeline::request_lease(&execution);
        if lease.is_some_and(|lease| {
            lease.resolved_posture() == worth_foundational::ExecutionPosture::Automatic
        }) && self
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
        let mut runtime = self.runtime.with_candidate_input_plan(&request);
        runtime.custom_kernel_posture = lease.map_or(
            worth_foundational::ExecutionPosture::Serial,
            ExecutionResourceLease::resolved_posture,
        );
        let (planned, retained_bytes) = if let Some(lease) = lease {
            let preparation = crate::validation::execution::plan_checked_invariant_preparation(
                &runtime, execution, &request, lease,
            );
            check_live(execution)?;
            let (planned, retained_bytes, _) = preparation?;
            (planned, retained_bytes)
        } else {
            (
                plan_invariant_execution(&runtime, execution, &request, None),
                0,
            )
        };
        check_live(execution)?;
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
                packets,lease,runtime.commit_work_budget.as_ref(),
                |packet,context|crate::validation::execution::evaluate_invariant_packet_checked(
                    &runtime,execution,packet,context,lease.status(),
                ),
                crate::validation::execution::InvariantWorkerEnvelope::owned_allocation_capacity_bytes,
            )
        } else {
            crate::execution::execute_unleased_read_only_packets(&packets, |packet| {
                execution
                    .check_live()
                    .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
                let envelope = evaluate_invariant_packet(&runtime, execution, packet);
                execution
                    .check_live()
                    .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
                Ok(envelope)
            })
            .map_err(crate::execution::PacketExecutionStop::from)
        };
        check_live(execution)?;
        let envelopes = envelopes?;
        let (result, _, reducer_conflicts) = if let Some(lease) = lease {
            let reduction = reduce_invariant_execution_checked(
                &request,
                strategy,
                proof_boundary,
                envelopes,
                lease,
                runtime.commit_work_budget.as_ref(),
            );
            check_live(execution)?;
            reduction?
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
        check_live(execution)?;
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

fn check_live(
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), crate::transactions::data::TransactionCommitError> {
    execution.check_live().map_err(|stop| {
        crate::transactions::data::TransactionCommitError::execution(
            crate::transactions::data::CommitExecutionDenial {
                kind: crate::transactions::data::CommitExecutionDenialKind::Cause(stop.into()),
                partition_identity: None,
            },
        )
    })
}

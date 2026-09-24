use crate::validation::engine::InvariantRuntimeView;
use crate::validation::execution::{
    evaluate_invariant_packet, plan_invariant_execution, planned_proof_boundary_summary,
};
use crate::validation::reduction::reduce_invariant_execution;
use rayon::prelude::*;
use std::collections::BTreeSet;

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
        let runtime = self.runtime.with_candidate_input_plan(&request);
        let mut work_plan =
            crate::authority::commit::preparation::planning::work_plan::empty_preparation_work_plan(
            );
        work_plan.invariant_execution = Some(plan_invariant_execution(&runtime, &request));
        self.record_preparation_plan(&work_plan);
        let planned = work_plan
            .invariant_execution
            .as_ref()
            .expect("validation work plan must include invariant execution");
        let envelopes = match planned.strategy.selected_mode {
            crate::authority::commit::preparation::planning::strategy::PreparationStrategySelection::Serial => {
                planned
                    .packets
                    .iter()
                    .map(|packet| evaluate_invariant_packet(&runtime, packet))
                    .collect()
            }
            crate::authority::commit::preparation::planning::strategy::PreparationStrategySelection::StagedParallel => {
                planned
                    .packets
                    .par_iter()
                    .map(|packet| evaluate_invariant_packet(&runtime, packet))
                    .collect()
            }
        };
        let proof_boundary = planned_proof_boundary_summary(planned);
        let (result, _, reducer_conflicts) =
            reduce_invariant_execution(&request, planned.strategy, proof_boundary, envelopes);
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
        result
    }
}

impl InvariantEngine<'_> {
    fn record_preparation_plan(
        &self,
        work_plan: &crate::authority::commit::preparation::PreparationWorkPlan<'_>,
    ) {
        let Some(planned) = work_plan.invariant_execution.as_ref() else {
            return;
        };
        let performance = self.runtime.performance_access();
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
        performance.count_preparation_packet_shape(
            counters.packet_count,
            counters.packet_count,
            usize::from(counters.packet_count > 0),
            scope_units,
        );
        debug_assert!(planned
            .packets
            .iter()
            .all(|packet| packet.planning_context == planned.context));
        match planned.strategy.parallel_legality {
            crate::authority::commit::preparation::planning::strategy::ParallelLegality::ProvenParallel => {
                performance.count_preparation_parallel_legal();
            }
            crate::authority::commit::preparation::planning::strategy::ParallelLegality::RequiresSerial => {}
        }
        match planned.strategy.parallel_profitability {
            crate::authority::commit::preparation::planning::strategy::ParallelProfitability::Profitable => {
                performance.count_preparation_parallel_profitable();
            }
            crate::authority::commit::preparation::planning::strategy::ParallelProfitability::NotProfitable => {}
        }
        match planned.strategy.selected_mode {
            crate::authority::commit::preparation::planning::strategy::PreparationStrategySelection::Serial => {
                performance.count_preparation_serial_strategy();
            }
            crate::authority::commit::preparation::planning::strategy::PreparationStrategySelection::StagedParallel => {
                performance.count_preparation_staged_parallel_strategy();
            }
        }
    }
}

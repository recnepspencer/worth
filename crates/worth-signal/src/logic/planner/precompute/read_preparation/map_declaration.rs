//! Declares the admitted graph epoch's checked map and resource bounds.
use super::super::eligibility::PrevalidatedTask;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::proof::invalidation::progression::{DisjointGraphBatch, GraphProposalKey};
use crate::data::request_preparation::{self as preparation_budget, SignalPreparationBudget};
use worth_execution::{ChargedBytes, ExecutionMap, MapKernelContext, MapPartition};
use worth_foundational::PartitionIdentity;

pub(super) struct GraphWorkItem {
    pub(super) task_index: usize,
    pub(super) capacity: super::super::callback::CheckedKernelCapacity,
}
impl ChargedBytes for GraphWorkItem {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

pub(super) fn lower_checked_map(
    graph: &SignalGraph,
    validated: &[PrevalidatedTask],
    batch: &DisjointGraphBatch,
    stage: &crate::logic::planner::execution::CheckedAdmittedEpoch<'_, '_, '_>,
    mut work: Option<&mut MapKernelContext<'_, '_>>,
    mut preparation: Option<&mut SignalPreparationBudget>,
) -> Result<ExecutionMap<GraphWorkItem, GraphProposalKey>, SignalError> {
    let count = validated
        .iter()
        .filter(|task| matches!(task, PrevalidatedTask::NeedsCompute { .. }))
        .count();
    super::super::work::checkpoint(work.as_deref_mut(), validated.len().saturating_add(count))?;
    preparation_budget::claim_vec::<MapPartition<GraphWorkItem, GraphProposalKey>>(
        preparation.as_deref_mut(),
        count,
    )?;
    preparation_budget::claim_vec::<PartitionIdentity>(preparation.as_deref_mut(), count)?;
    let mut partitions = Vec::with_capacity(count);
    for (index, (task, validation)) in stage.tasks().iter().zip(validated).enumerate() {
        if !matches!(validation, PrevalidatedTask::NeedsCompute { .. }) {
            continue;
        }
        let declaration = graph
            .get_contract(task.node)?
            .execution
            .bounded_inputs
            .as_ref()
            .expect("admitted");
        let capture_bytes =
            crate::logic::prepared::PreparedDependencyCapture::checked_capture_heap_bound(
                declaration,
            )
            .ok_or_else(|| SignalError::invalid_input("checked capture memory overflow"))?;
        let per_result = stage
            .result_grant_bytes()
            .checked_add(capture_bytes)
            .ok_or_else(|| SignalError::invalid_input("checked result grant overflow"))?;
        let result_bytes = per_result.checked_sub(capture_bytes).ok_or_else(|| {
            SignalError::invalid_input("checked capture exceeds admitted result capacity")
        })?;
        let scratch_bytes = per_result
            .checked_add(capture_bytes)
            .ok_or_else(|| SignalError::invalid_input("checked scratch memory overflow"))?;
        super::super::work::checkpoint(
            work.as_deref_mut(),
            declaration
                .as_slice()
                .len()
                .saturating_add(crate::data::aspect::MAX_ASPECTS + 8),
        )?;
        preparation_budget::claim_vec::<GraphProposalKey>(
            preparation.as_deref_mut(),
            declaration.as_slice().len(),
        )?;
        let effect_count = batch
            .effects()
            .iter()
            .find(|(node, _)| *node == task.node)
            .ok_or_else(|| SignalError::internal("admitted graph task missing effects"))?
            .1
            .len();
        preparation_budget::claim_vec::<GraphProposalKey>(
            preparation.as_deref_mut(),
            effect_count,
        )?;
        if let Some(budget) = preparation.as_deref_mut() {
            // Completed map values outlive the map's own reservation until the
            // epoch owner consumes them, so the request retains their ceiling.
            budget.claim(per_result)?;
        }
        let mut read_keys = declaration
            .as_slice()
            .iter()
            .map(|input| GraphProposalKey::Aspect(input.source, input.aspect))
            .collect::<Vec<_>>();
        read_keys.sort_unstable();
        read_keys.dedup();
        let write_keys = batch
            .effects()
            .iter()
            .find(|(node, _)| *node == task.node)
            .ok_or_else(|| SignalError::internal("admitted graph task missing effects"))?
            .1
            .clone();
        partitions.push(MapPartition {
            identity: PartitionIdentity::new(stage.task_offset() as u64 + index as u64),
            value: GraphWorkItem {
                task_index: index,
                capacity: super::super::callback::CheckedKernelCapacity {
                    scratch_bytes: per_result,
                    result_bytes,
                },
            },
            read_keys,
            write_keys,
            kernel_scratch_bytes: scratch_bytes,
            max_result_bytes: per_result,
        });
    }
    let identities = partitions
        .iter()
        .map(|partition| partition.identity)
        .collect();
    ExecutionMap::try_from_declared_partitions(identities, partitions)
        .map_err(|_| SignalError::invalid_input("graph epoch access or memory declaration denied"))
}

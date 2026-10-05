mod checked;

pub(crate) use checked::reduce_invariant_execution_checked;

use crate::authority::commit::preparation::diagnostics::counters::ValidationPreparationCounters;
use crate::authority::commit::preparation::diagnostics::failures::PreparationFailureClass;
use crate::authority::commit::preparation::planning::strategy::PreparationStrategy;
use crate::validation::engine::{
    InvariantExecutionMetadata, InvariantExecutionRequest, InvariantExecutionResult,
    InvariantProofBoundarySummary,
};
use crate::validation::execution::{
    check_result_bytes, identity_bytes, InvariantWorkerEnvelope, ValidationReducerConflict,
};

pub(crate) type ReducedInvariants = (
    InvariantExecutionResult,
    ValidationPreparationCounters,
    Vec<ValidationReducerConflict>,
);

pub(crate) fn reduce_invariant_execution(
    request: &InvariantExecutionRequest<'_>,
    strategy: PreparationStrategy,
    proof_boundary: InvariantProofBoundarySummary,
    envelopes: Vec<InvariantWorkerEnvelope>,
) -> ReducedInvariants {
    reduce_inner(request, strategy, proof_boundary, envelopes, |_, _| {
        Ok::<(), ()>(())
    })
    .expect("unleased reduction has no execution ceiling")
}

/// The checked caller supplies an admitted checkpoint and capacity claim.
pub(super) fn reduce_inner<E>(
    request: &InvariantExecutionRequest<'_>,
    strategy: PreparationStrategy,
    proof_boundary: InvariantProofBoundarySummary,
    mut envelopes: Vec<InvariantWorkerEnvelope>,
    mut check: impl FnMut(u64, u64) -> Result<(), E>,
) -> Result<ReducedInvariants, E> {
    let packet_count = envelopes.len();
    let sort_work = (packet_count as u64)
        .saturating_mul((usize::BITS - packet_count.saturating_sub(1).leading_zeros()) as u64);
    check(sort_work, 0)?;
    envelopes.sort_unstable_by(|left, right| {
        let left_identity = &left
            .results
            .first()
            .expect("worker envelope has a result")
            .result_identity;
        let right_identity = &right
            .results
            .first()
            .expect("worker envelope has a result")
            .result_identity;
        (&left.reduction_key, left_identity, left.packet_index).cmp(&(
            &right.reduction_key,
            right_identity,
            right.packet_index,
        ))
    });

    let result_count = envelopes
        .iter()
        .map(|envelope| envelope.results.len())
        .sum::<usize>();
    check(
        0,
        (result_count as u64).saturating_mul(std::mem::size_of::<
            crate::validation::data::InvariantCheckResult,
        >() as u64),
    )?;
    let mut results = Vec::with_capacity(result_count);
    let mut preparation_failures = Vec::new();
    let mut reducer_conflicts = Vec::new();
    let mut last_identity = None;
    for envelope in envelopes {
        for failure in envelope.preparation_failures {
            check(
                1,
                (std::mem::size_of::<PreparationFailureClass>() as u64).saturating_mul(2),
            )?;
            preparation_failures.push(failure);
        }
        for worker_result in envelope.results {
            check(
                1,
                check_result_bytes(&worker_result.result)
                    .saturating_add(identity_bytes(&worker_result.result_identity)),
            )?;
            if last_identity.as_ref() == Some(&worker_result.result_identity) {
                check(
                    1,
                    (std::mem::size_of::<ValidationReducerConflict>() as u64)
                        .saturating_mul(2)
                        .saturating_add(identity_bytes(&worker_result.result_identity)),
                )?;
                reducer_conflicts.push(ValidationReducerConflict {
                    identity: worker_result.result_identity.clone(),
                });
            }
            last_identity = Some(worker_result.result_identity.clone());
            results.push(worker_result.result);
        }
    }
    if strategy.serial_selection_reason.is_some() {
        check(
            1,
            (std::mem::size_of::<PreparationFailureClass>() as u64).saturating_mul(2),
        )?;
        preparation_failures.push(PreparationFailureClass::SerialStrategySelected);
    }
    for _ in &reducer_conflicts {
        check(
            1,
            (std::mem::size_of::<PreparationFailureClass>() as u64).saturating_mul(2),
        )?;
        preparation_failures.push(PreparationFailureClass::ReductionIdentityConflict);
    }

    let worker_result_count = results.len();
    let failure_count = preparation_failures.len();
    check(
        0,
        (worker_result_count as u64).saturating_mul(std::mem::size_of::<
            crate::validation::data::InvariantDecisionRecord,
        >() as u64),
    )?;
    for result in &results {
        check(1, check_result_bytes(result))?;
    }
    if let Some(identity) = request.proposal_identity() {
        let observation = identity.branch_observation();
        let branch_bytes = observation.branch_id().as_str().len() as u64;
        let parent_bytes = observation.target().as_basis().map_or(0, |basis| {
            (basis.parent_commit_ids().len() as u64)
                .saturating_mul(std::mem::size_of::<u64>() as u64)
        });
        check(0, branch_bytes.saturating_add(parent_bytes))?;
    }
    let metadata = InvariantExecutionMetadata::executed_with_strategy(
        request.execution_point(),
        request.observation().kind(),
        request.version_id(),
        request.current_version_id(),
        request.consumed_groups(),
        request.applicable_groups(),
        request.max_cost(),
        request.plan_contract(),
        request.merged_plan().is_some(),
        strategy,
        preparation_failures,
        Some(proof_boundary),
        request.proposal_identity().cloned(),
    );
    let result = InvariantExecutionResult::executed(metadata, results);
    let counters = ValidationPreparationCounters {
        packet_count,
        worker_result_count,
        reducer_input_count: worker_result_count,
        reducer_conflict_count: reducer_conflicts.len(),
        failure_count,
    };
    Ok((result, counters, reducer_conflicts))
}

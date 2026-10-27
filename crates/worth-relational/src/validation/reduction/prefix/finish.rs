//! One admitted output projection after the last actually completed stage.
use super::*;
use crate::authority::commit::preparation::{
    diagnostics::{counters::ValidationPreparationCounters, failures::PreparationFailureClass},
    planning::strategy::PreparationStrategySelection,
};
use crate::validation::{
    data::{InvariantCheckResult, InvariantDecisionRecord},
    engine::{
        InvariantExecutionDisposition, InvariantExecutionMetadata, InvariantExecutionRequest,
        InvariantExecutionResult, InvariantPlanScopeClass, InvariantProofBoundarySummary,
    },
    execution::{composed_proof_boundary_summary, decision_clone_bytes, decision_record_bytes},
};

pub(crate) struct CompletedInvariantReduction {
    reduced: super::super::ReducedInvariants,
    // These cover payloads moved out of the envelopes until engine handoff.
    _stage_memory: [[Option<ExecutionMemoryReservation>; 2]; 2],
    pub(super) projection_memory: Option<ExecutionMemoryReservation>,
    new_owned_bytes: u64,
}

impl CompletedInvariantReduction {
    #[cfg(test)]
    pub(crate) fn result(&self) -> &InvariantExecutionResult {
        &self.reduced.0
    }
    pub(crate) fn conflicts(&self) -> &[ValidationReducerConflict] {
        &self.reduced.2
    }
    pub(crate) fn into_result(self) -> InvariantExecutionResult {
        self.reduced.0
    }
}

impl worth_execution::ChargedBytes for CompletedInvariantReduction {
    fn additional_charged_bytes(&self) -> u64 {
        self.new_owned_bytes
    }
}

impl ReducedInvariantPrefix {
    pub(super) fn project<E>(
        mut self,
        request: &InvariantExecutionRequest<'_>,
        mut check: impl FnMut(u64, u64) -> Result<(), E>,
    ) -> Result<CompletedInvariantReduction, E> {
        let packet_count = self.packet_count();
        let count = self.summary.result_count();
        let conflict_count: usize = self.completed().map(|stage| stage.conflicts.len()).sum();
        let serial_count = self
            .completed()
            .filter(|stage| stage.output.strategy.serial_selection_reason.is_some())
            .count();
        let failure_count = self
            .completed()
            .map(|stage| {
                stage
                    .output
                    .envelopes
                    .iter()
                    .map(|envelope| envelope.preparation_failures.len())
                    .sum::<usize>()
            })
            .sum::<usize>()
            + serial_count
            + conflict_count;
        let backing = (count as u64)
            .saturating_mul(
                (std::mem::size_of::<InvariantCheckResult>()
                    + std::mem::size_of::<InvariantDecisionRecord>()) as u64,
            )
            .saturating_add(
                (failure_count as u64)
                    .saturating_mul(std::mem::size_of::<PreparationFailureClass>() as u64),
            )
            .saturating_add(
                (conflict_count as u64)
                    .saturating_mul(std::mem::size_of::<ValidationReducerConflict>() as u64),
            );
        check(0, backing)?;
        let mut results = Vec::with_capacity(count);
        let mut decisions = Vec::with_capacity(count);
        let mut failures = Vec::with_capacity(failure_count);
        let mut conflicts = Vec::with_capacity(conflict_count);
        // Clone decisions before releasing their original envelope backing.
        for stage in self.completed() {
            for envelope in &stage.output.envelopes {
                for worker in &envelope.results {
                    check(1, decision_clone_bytes(&worker.result))?;
                    decisions.push(worker.result.decision_record());
                }
            }
        }
        let mut new_owned_bytes =
            backing.saturating_add(decisions.iter().fold(0_u64, |total, decision| {
                total.saturating_add(decision_record_bytes(decision))
            }));
        let strategy = self
            .completed()
            .map(|stage| stage.output.strategy)
            .reduce(|first, next| {
                if next.selected_mode == PreparationStrategySelection::StagedParallel {
                    next
                } else {
                    first
                }
            });
        let proof = if self.stage_count == 1 {
            std::mem::replace(
                &mut self.stages[0]
                    .as_mut()
                    .expect("completed first stage")
                    .output
                    .proof_boundary,
                empty_proof(),
            )
        } else {
            let proof = composed_proof_boundary_summary(
                self.completed().map(|stage| &stage.output),
                &mut check,
            )?;
            new_owned_bytes =
                new_owned_bytes.saturating_add(proof.owned_allocation_capacity_bytes());
            proof
        };
        let proposal_bytes = request.proposal_identity().map_or(0, |identity| {
            let observation = identity.branch_observation();
            let parents = observation.target().as_basis().map_or(0, |basis| {
                (basis.parent_commit_ids().len() as u64)
                    .saturating_mul(std::mem::size_of::<u64>() as u64)
            });
            (observation.branch_id().as_str().len() as u64).saturating_add(parents)
        });
        check(0, proposal_bytes)?;
        new_owned_bytes = new_owned_bytes.saturating_add(proposal_bytes);
        // Tickets stay inside self while fallible projection moves payloads out.
        // On a stop the local output vectors drop before self and its tickets.
        for stage in self.stages.iter_mut().flatten() {
            for envelope in std::mem::take(&mut stage.output.envelopes) {
                for failure in envelope.preparation_failures {
                    check(1, 0)?;
                    failures.push(failure);
                }
                for worker in envelope.results {
                    check(1, 0)?;
                    results.push(worker.result);
                }
            }
            if stage.output.strategy.serial_selection_reason.is_some() {
                failures.push(PreparationFailureClass::SerialStrategySelected);
            }
            conflicts.extend(stage.conflicts.drain(..));
        }
        for _ in &conflicts {
            check(1, 0)?;
            failures.push(PreparationFailureClass::ReductionIdentityConflict);
        }
        let metadata = InvariantExecutionMetadata::new(
            request.execution_point(),
            request.observation().kind(),
            request.version_id(),
            request.current_version_id(),
            request.consumed_groups(),
            request.applicable_groups(),
            request.max_cost(),
            InvariantExecutionDisposition::Executed,
            request.plan_contract(),
            request.merged_plan().is_some(),
            strategy,
            failures,
            Some(proof),
            request.proposal_identity().cloned(),
        );
        let mut stage_memory = [[None, None], [None, None]];
        for (index, stage) in self.stages.into_iter().flatten().enumerate() {
            stage_memory[index] = [stage.input_memory, stage.auxiliary_memory];
        }
        let result = InvariantExecutionResult::from_reduced_parts(
            metadata,
            self.summary,
            results,
            decisions,
        );
        let counters = ValidationPreparationCounters {
            packet_count,
            worker_result_count: count,
            reducer_input_count: count,
            reducer_conflict_count: conflict_count,
            failure_count,
        };
        Ok(CompletedInvariantReduction {
            reduced: (result, counters, conflicts),
            _stage_memory: stage_memory,
            projection_memory: None,
            new_owned_bytes,
        })
    }
}

fn empty_proof() -> InvariantProofBoundarySummary {
    InvariantProofBoundarySummary::new(InvariantPlanScopeClass::PartitionScope, Vec::new(), 0, 0)
}

//! One canonical reduction, suspended between native and custom stages.
mod finish;
mod measured;

use crate::execution::PacketAdmissionDenial;
use crate::validation::data::InvariantExecutionPoint;
use crate::validation::engine::InvariantExecutionSummary;
use crate::validation::execution::{
    identity_bytes, InvariantStageOutput, InvariantWorkerEnvelope, ValidationReducerConflict,
};
use worth_execution::ExecutionMemoryReservation;

pub(crate) struct ReducedInvariantPrefix {
    // The summary's cloned failures drop before the tickets that cover them.
    summary: InvariantExecutionSummary,
    stages: [Option<ReducedStage>; 2],
    stage_count: usize,
}

struct ReducedStage {
    output: InvariantStageOutput,
    conflicts: Vec<ValidationReducerConflict>,
    summary_bytes: u64,
    input_memory: Option<ExecutionMemoryReservation>,
    auxiliary_memory: Option<ExecutionMemoryReservation>,
}

struct StageInput {
    output: InvariantStageOutput,
    // Drop backing before releasing its original-request reservation.
    memory: Option<ExecutionMemoryReservation>,
}

pub(super) enum ReductionStop<E> {
    Check(E),
    Order,
}

impl ReducedInvariantPrefix {
    pub(crate) fn new() -> Self {
        Self {
            summary: InvariantExecutionSummary::empty(),
            stages: [None, None],
            stage_count: 0,
        }
    }

    pub(crate) fn packet_count(&self) -> usize {
        self.completed()
            .map(|stage| stage.output.envelopes.len())
            .sum()
    }

    pub(crate) fn prevents_custom(&self, point: InvariantExecutionPoint) -> bool {
        match point {
            InvariantExecutionPoint::SnapshotPublication
            | InvariantExecutionPoint::CertificationBoundary => {
                self.summary.has_publication_violation()
            }
            InvariantExecutionPoint::HarnessAudit => false,
            _ => self.summary.has_blocking_violation(),
        }
    }

    fn completed(&self) -> impl DoubleEndedIterator<Item = &ReducedStage> {
        self.stages.iter().filter_map(Option::as_ref)
    }

    /// The caller owns all input backing; the callback admits newly retained growth.
    fn absorb<E>(
        mut self,
        mut input: StageInput,
        mut check: impl FnMut(u64, u64) -> Result<(), E>,
    ) -> Result<Self, ReductionStop<E>> {
        let output = &mut input.output;
        let mut check = |work, bytes| check(work, bytes).map_err(ReductionStop::Check);
        let count = output.envelopes.len();
        if self.stage_count == self.stages.len()
            || count == 0
            || output.proof_boundary.packet_count() != count
            || output
                .envelopes
                .iter()
                .any(|envelope| envelope.results.is_empty())
        {
            return Err(ReductionStop::Order);
        }
        check(
            (count as u64)
                .saturating_mul((usize::BITS - count.saturating_sub(1).leading_zeros()) as u64),
            0,
        )?;
        output.envelopes.sort_unstable_by(compare_envelopes);
        let offset = self.packet_count();
        if output
            .envelopes
            .iter()
            .enumerate()
            .any(|(index, envelope)| {
                offset.checked_add(index) != Some(envelope.packet_index)
                    || envelope.packet_index != envelope.reduction_key.packet_index
            })
        {
            return Err(ReductionStop::Order);
        }
        if let Some(last) = self
            .completed()
            .next_back()
            .and_then(|stage| stage.output.envelopes.last())
        {
            check(1, 0)?;
            if compare_envelopes(last, &output.envelopes[0]).is_gt() {
                return Err(ReductionStop::Order);
            }
        }
        let mut previous = self
            .stages
            .iter()
            .rev()
            .filter_map(Option::as_ref)
            .find_map(|stage| stage.output.envelopes.last())
            .and_then(|envelope| envelope.results.last())
            .map(|worker| &worker.result_identity);
        let before_summary = self.summary.owned_allocation_capacity_bytes();
        let mut conflicts = Vec::new();
        for envelope in &output.envelopes {
            for worker in &envelope.results {
                // The summary may retain the first original failure of each effect.
                check(
                    1,
                    self.summary
                        .pending_failure_allocation_bytes(&worker.result),
                )?;
                self.summary.observe(&worker.result);
                if previous == Some(&worker.result_identity) {
                    if conflicts.capacity() == 0 {
                        let capacity: usize = output
                            .envelopes
                            .iter()
                            .map(|entry| entry.results.len())
                            .sum();
                        check(
                            output.envelopes.len() as u64,
                            (capacity as u64)
                                .saturating_mul(
                                    std::mem::size_of::<ValidationReducerConflict>() as u64
                                ),
                        )?;
                        conflicts = Vec::with_capacity(capacity);
                    }
                    check(1, identity_bytes(&worker.result_identity))?;
                    conflicts.push(ValidationReducerConflict {
                        identity: worker.result_identity.clone(),
                    });
                }
                previous = Some(&worker.result_identity);
            }
        }
        self.stages[self.stage_count] = Some(ReducedStage {
            output: input.output,
            conflicts,
            summary_bytes: self.summary.owned_allocation_capacity_bytes() - before_summary,
            input_memory: input.memory,
            auxiliary_memory: None,
        });
        self.stage_count += 1;
        Ok(self)
    }
}

fn compare_envelopes(
    left: &InvariantWorkerEnvelope,
    right: &InvariantWorkerEnvelope,
) -> std::cmp::Ordering {
    (
        &left.reduction_key,
        &left.results[0].result_identity,
        left.packet_index,
    )
        .cmp(&(
            &right.reduction_key,
            &right.results[0].result_identity,
            right.packet_index,
        ))
}

impl ReducedStage {
    fn auxiliary_bytes(&self) -> u64 {
        (self.conflicts.capacity() as u64)
            .saturating_mul(std::mem::size_of::<ValidationReducerConflict>() as u64)
            .saturating_add(self.conflicts.iter().fold(0_u64, |total, conflict| {
                total.saturating_add(identity_bytes(&conflict.identity))
            }))
            .saturating_add(self.summary_bytes)
    }
}

fn order_denial() -> crate::execution::PacketExecutionStop {
    crate::execution::PacketExecutionStop::Admission(
        PacketAdmissionDenial::ExpectedIdentitiesNotCanonical,
    )
}

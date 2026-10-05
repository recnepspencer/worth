use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

use crate::data::error::SignalError;
use crate::data::output::MemoizedResultOrigin;
use crate::data::reuse::ReuseOrigin;
use crate::data::telemetry::InvalidationPerformedCounter;
use crate::facade::SignalObservationRequest;
use crate::logic::planner::StageExecutionOutcome;
use crate::logic::planner::{TaskExecutionOutcome, TaskReason};

use super::CompiledFinancialLocalityWorld;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FinancialSemanticWorkRow {
    pub(crate) target_ordinal: u32,
    pub(crate) dependency_revision: u64,
    pub(crate) stage_index: u32,
    pub(crate) task_id: u64,
    pub(crate) semantic_segment_id: u64,
    pub(crate) scheduled_reason: TaskReason,
    pub(crate) outcome: TaskExecutionOutcome,
    pub(crate) memoized_origin: MemoizedResultOrigin,
    pub(crate) reuse_origin: ReuseOrigin,
    pub(crate) readiness_epoch: u64,
    pub(crate) recomputed: bool,
}

/// The measured output of one production locality-performance sequence.
///
/// This is deliberately a small, test-owned report.  It carries the timing,
/// bounded-work, allocation, and retained-state facts needed by the M10
/// performance packet without turning the runtime's public API into a
/// benchmark protocol.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FinancialPerformanceBatchReport {
    pub(crate) node_count: usize,
    pub(crate) batch_count: usize,
    pub(crate) warm_median_micros: u128,
    pub(crate) warm_p95_micros: u128,
    pub(crate) completed_batches_per_second: u64,
    pub(crate) completed_batches_per_second_milli: u64,
    pub(crate) peak_batch_memory_items: u64,
    pub(crate) batch_local_allocations: u64,
    pub(crate) retained_work_items: usize,
    pub(crate) peak_touched_nodes: usize,
    pub(crate) parallel_stage_dispatches: u64,
    pub(crate) mutation_widths: Vec<usize>,
    pub(crate) semantic_work_rows: Vec<Vec<FinancialSemanticWorkRow>>,
}

impl CompiledFinancialLocalityWorld {
    pub(crate) fn run_performance_sequence(
        &mut self,
        batch_count: usize,
        workers: usize,
        observe: bool,
    ) -> Result<FinancialPerformanceBatchReport, SignalError> {
        let trace_count = self.locality_definition().action_traces().len();
        if trace_count == 0 || batch_count == 0 {
            return Err(SignalError::invalid_input(
                "performance sequence requires nonempty traces and batches",
            ));
        }
        let session = observe.then(|| {
            self.runtime
                .begin_observation_session(SignalObservationRequest::operation())
        });
        let session = match session {
            Some(Ok(session)) => Some(session),
            Some(Err(denial)) => return Err(SignalError::invalid_input(denial.to_string())),
            None => None,
        };

        let mut samples = Vec::with_capacity(batch_count);
        let mut peak_touched_nodes = 0;
        let mut mutation_widths = Vec::with_capacity(batch_count);
        let mut parallel_stage_dispatches = 0;
        let mut semantic_work_rows = Vec::with_capacity(batch_count);
        let output_by_node = self
            .handles
            .iter()
            .map(|(output, node)| (*node, *output))
            .collect::<BTreeMap<_, _>>();
        for batch in 0..batch_count {
            let trace = &self.locality_definition().action_traces()[batch % trace_count];
            let mutations = trace.committed_mutations();
            mutation_widths.push(mutations.len());
            let started = Instant::now();
            self.apply_mutations(&mutations)?;
            let settlement = self.settle_mutations_with_retries_at_batch(
                &mutations,
                &[],
                workers,
                batch,
                false,
            )?;
            peak_touched_nodes = peak_touched_nodes.max(settlement.evaluated_outputs.len());
            parallel_stage_dispatches += settlement
                .stage_outcomes
                .iter()
                .filter(|outcome| matches!(outcome, StageExecutionOutcome::CompletedParallel))
                .count() as u64;
            // Stop the production timing interval before constructing the
            // test-owned semantic evidence.  Evidence assembly must not
            // contaminate the performance result with an O(tasks × outputs)
            // oracle scan.
            samples.push(started.elapsed());
            let mut semantic_rows = Vec::new();
            for stage in &settlement.stage_records {
                for task in &stage.task_records {
                    let output = output_by_node.get(&task.node).copied().ok_or_else(|| {
                        SignalError::internal("stage task target is not financial")
                    })?;
                    semantic_rows.push(FinancialSemanticWorkRow {
                        target_ordinal: output.ordinal(),
                        dependency_revision: self.runtime.graph().dependency_revision(task.node)?.0,
                        stage_index: stage.stage_index,
                        task_id: task.id.0,
                        semantic_segment_id: task.semantic_segment_id.0,
                        scheduled_reason: task.scheduled_reason,
                        outcome: task.outcome,
                        memoized_origin: task.memoized_origin,
                        reuse_origin: task.reuse_origin,
                        readiness_epoch: self
                            .runtime
                            .graph()
                            .current_invalidation_readiness_epoch()
                            .0,
                        recomputed: task.recomputed,
                    });
                }
            }
            if semantic_rows.is_empty() {
                return Err(SignalError::internal(
                    "production settlement did not provide semantic work identity",
                ));
            }
            semantic_work_rows.push(semantic_rows);
        }

        let captured_counters = self.runtime.graph().invalidation_performed_counters();
        let captured_retained_work_items = self.runtime.graph().invalidation_performed_work().len();

        if let Some(session) = session {
            self.runtime
                .finish_observation_session(&session)
                .map_err(|denial| SignalError::invalid_input(denial.to_string()))?;
        }

        let counters = captured_counters;
        let total_elapsed = samples
            .iter()
            .copied()
            .fold(Duration::ZERO, |total, sample| total.saturating_add(sample));
        samples.sort_unstable();
        let median = samples[samples.len() / 2].as_micros().max(1);
        let p95 = samples[((samples.len() * 95).saturating_sub(1) / 100).min(samples.len() - 1)]
            .as_micros()
            .max(1);
        let throughput = if total_elapsed.is_zero() {
            0
        } else {
            ((batch_count as f64 / total_elapsed.as_secs_f64()) as u64).max(1)
        };
        let throughput_milli = if total_elapsed.is_zero() {
            0
        } else {
            ((batch_count as f64 / total_elapsed.as_secs_f64() * 1_000.0) as u64).max(1)
        };

        Ok(FinancialPerformanceBatchReport {
            node_count: self.locality_definition().outputs().len(),
            batch_count,
            warm_median_micros: median,
            warm_p95_micros: p95,
            completed_batches_per_second: throughput,
            completed_batches_per_second_milli: throughput_milli,
            peak_batch_memory_items: counters
                .value(InvalidationPerformedCounter::PeakBatchMemoryItems),
            batch_local_allocations: counters
                .value(InvalidationPerformedCounter::BatchLocalAllocations),
            retained_work_items: captured_retained_work_items,
            peak_touched_nodes,
            parallel_stage_dispatches,
            mutation_widths,
            semantic_work_rows,
        })
    }
}

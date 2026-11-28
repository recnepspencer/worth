use crate::diagnostics::failure::FailureSummary;

use super::super::super::transaction_types::{
    SignalTransaction, TransactionOutcome, TransactionReplayEntry, TransactionResult,
    TransactionTiming,
};

pub(super) struct CapturedFinalization {
    pub(super) result: TransactionResult,
    pub(super) failure: Option<FailureSummary>,
    pub(super) replay_events: Vec<TransactionReplayEntry>,
}

impl<'a, D, I, E, Ctx, T> SignalTransaction<'a, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) fn capture_finalization_boundary(
        &mut self,
        outcome: TransactionOutcome,
        touched_nodes: u32,
        commit_nanos: u128,
    ) -> CapturedFinalization {
        let rollback = self.scratch.semantic_delta.rollback.take();
        let failure = self.scratch.semantic_delta.failure_summary.take();
        let observation = std::mem::take(&mut self.scratch.semantic_delta.observation);
        let replay_events = std::mem::take(&mut self.scratch.semantic_delta.replay_events);
        let event_epochs = std::mem::take(&mut self.scratch.semantic_delta.event_epochs);
        let timing = TransactionTiming {
            total_nanos: self.graph.elapsed_nanos(self.started_at),
            evaluation_nanos: self.execution_state.evaluation_nanos,
            event_flush_nanos: self.scratch.staged_event_flush_nanos,
            commit_nanos,
        };
        let temporal_evidence = self
            .scratch
            .temporal
            .boundary_evidence(self.temporal.clock_basis());
        let reconstructability = self.boundary_reconstructability(&replay_events);
        let result = TransactionResult::from_boundary_state(
            outcome,
            self.execution_state.latest_report.take(),
            timing,
            touched_nodes,
            std::mem::take(&mut self.execution_state.summary),
            std::mem::take(&mut self.scratch.temporal.summary),
            temporal_evidence,
            &replay_events,
            reconstructability,
            event_epochs,
            rollback,
            failure.clone(),
            observation,
            self.telemetry_snapshot(),
        );
        CapturedFinalization {
            result,
            failure,
            replay_events,
        }
    }
}

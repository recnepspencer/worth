use std::collections::BTreeMap;

use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;
use worth_relational::facade::identity::EntityId;

mod navigation;
mod replay;
mod retention;
mod update;
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};
use crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowDefinition;
use navigation::unique_successor;
pub(in crate::domain_computation::primary_graph) use replay::{
    WorkflowTransitionReplayProjection, WorkflowTransitionReplayRetention,
};
pub use retention::WorthQueryWorkflowInstanceProgressCounters;
pub(in crate::domain_computation::primary_graph) use retention::{
    default_progress_retention_shards, RetainedWorkflowInstanceProgressProjection,
    WorkflowInstanceProgressKey, WorkflowInstanceProgressRetention,
    WorkflowInstanceProgressRetentionDenial,
};
pub(in crate::domain_computation::primary_graph) use update::{
    PreparedWorkflowProgressUpdate, WorkflowTransitionProgressBasis,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct SettledWorkflowTransition {
    node: EntityId,
    occurrence: u64,
    outcome: ApplicationWorkflowControlOutcome,
    operation_receipt_identity: Option<[u8; 32]>,
}

impl SettledWorkflowTransition {
    pub(in crate::domain_computation::primary_graph) const fn new(
        node: EntityId,
        occurrence: u64,
        outcome: ApplicationWorkflowControlOutcome,
        operation_receipt_identity: Option<[u8; 32]>,
    ) -> Self {
        Self {
            node,
            occurrence,
            outcome,
            operation_receipt_identity,
        }
    }

    pub(in crate::domain_computation::primary_graph) const fn occurrence(self) -> u64 {
        self.occurrence
    }

    pub(in crate::domain_computation::primary_graph) const fn node(self) -> EntityId {
        self.node
    }

    pub(in crate::domain_computation::primary_graph) const fn outcome(
        self,
    ) -> ApplicationWorkflowControlOutcome {
        self.outcome
    }

    pub(in crate::domain_computation::primary_graph) const fn operation_receipt_identity(
        self,
    ) -> Option<[u8; 32]> {
        self.operation_receipt_identity
    }
}

/// Rebuildable, authority-free summary of an instance path up to, but not
/// including, a settled terminal transition.
///
/// The summary is small enough to retain independently of history. Admission
/// must still prove its source revision and live facts current before use. A
/// cold rebuild of a completed instance excludes its final terminal settlement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct WorkflowInstanceProgress {
    head: EntityId,
    next_occurrence: u64,
    retry_counts: BTreeMap<(EntityId, ApplicationWorkflowControlOutcome), usize>,
}

impl WorkflowInstanceProgress {
    pub(super) fn retained_charge_bytes(&self) -> usize {
        const CONSERVATIVE_TREE_NODE_ALLOWANCE: usize = 128;
        std::mem::size_of::<Self>().saturating_add(
            self.retry_counts.len().saturating_mul(
                std::mem::size_of::<((EntityId, ApplicationWorkflowControlOutcome), usize)>()
                    .saturating_add(CONSERVATIVE_TREE_NODE_ALLOWANCE),
            ),
        )
    }

    pub(in crate::domain_computation::primary_graph) fn reconstruct(
        compiled: &CompiledWorkflowDefinition,
        settled: &mut [SettledWorkflowTransition],
    ) -> Result<Self, WorthQueryApplicationAttemptDenial> {
        settled.sort_unstable_by_key(|transition| transition.occurrence());
        let mut progress = Self {
            head: compiled.start().entity(),
            next_occurrence: 0,
            retry_counts: BTreeMap::new(),
        };
        for transition in settled {
            progress.advance(compiled, *transition)?;
        }
        Ok(progress)
    }

    #[cfg(test)]
    fn reconstruct_with(
        settled: &mut [SettledWorkflowTransition],
        start: EntityId,
        mut successor: impl FnMut(
            EntityId,
            ApplicationWorkflowControlOutcome,
            usize,
        ) -> Result<EntityId, WorthQueryApplicationAttemptDenial>,
    ) -> Result<Self, WorthQueryApplicationAttemptDenial> {
        settled.sort_unstable_by_key(|transition| transition.occurrence());
        let mut progress = Self {
            head: start,
            next_occurrence: 0,
            retry_counts: BTreeMap::new(),
        };
        for transition in settled {
            progress.advance_with(*transition, &mut successor)?;
        }
        Ok(progress)
    }

    pub(in crate::domain_computation::primary_graph) const fn head(&self) -> EntityId {
        self.head
    }

    pub(in crate::domain_computation::primary_graph) const fn next_occurrence(&self) -> u64 {
        self.next_occurrence
    }

    pub(in crate::domain_computation::primary_graph) fn advance(
        &mut self,
        compiled: &CompiledWorkflowDefinition,
        transition: SettledWorkflowTransition,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.validate_next(transition)?;
        let source = transition.node();
        let outcome = transition.outcome();
        let attempts = if compiled.retry_successors(source, outcome).next().is_some() {
            let attempts = self.retry_counts.entry((source, outcome)).or_default();
            *attempts = attempts.saturating_add(1);
            *attempts
        } else {
            1
        };
        let successor = unique_successor(compiled, source, outcome, attempts)?.entity();
        self.finish_advance(successor)
    }

    #[cfg(test)]
    fn advance_with(
        &mut self,
        transition: SettledWorkflowTransition,
        successor: &mut impl FnMut(
            EntityId,
            ApplicationWorkflowControlOutcome,
            usize,
        ) -> Result<EntityId, WorthQueryApplicationAttemptDenial>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.validate_next(transition)?;
        let key = (transition.node(), transition.outcome());
        let attempts = self.retry_counts.entry(key).or_default();
        *attempts = attempts.saturating_add(1);
        let successor = successor(transition.node(), transition.outcome(), *attempts)?;
        self.finish_advance(successor)
    }

    fn validate_next(
        &self,
        transition: SettledWorkflowTransition,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        if transition.occurrence() != self.next_occurrence || transition.node() != self.head {
            return Err(denial(
                "workflow transition history is not a contiguous compiled path",
            ));
        }
        Ok(())
    }

    fn finish_advance(
        &mut self,
        successor: EntityId,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.head = successor;
        self.next_occurrence = self.next_occurrence.checked_add(1).ok_or_else(|| {
            WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionIdentityUnavailable,
                "workflow transition history exceeds supported occurrence range",
            )
        })?;
        Ok(())
    }
}

fn denial(subject: &'static str) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
        subject,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_relational::facade::identity::PartitionId;

    fn entity(slot: u64) -> EntityId {
        EntityId::new(PartitionId::new(7), slot, 1)
    }

    fn completed(node: EntityId, occurrence: u64) -> SettledWorkflowTransition {
        SettledWorkflowTransition::new(
            node,
            occurrence,
            ApplicationWorkflowControlOutcome::Completed,
            None,
        )
    }

    #[test]
    fn out_of_order_history_reconstructs_one_incremental_head() {
        let start = entity(10);
        let middle = entity(11);
        let terminal = entity(12);
        let edges = BTreeMap::from([(start, middle), (middle, terminal)]);
        let mut history = [completed(middle, 1), completed(start, 0)];

        let progress =
            WorkflowInstanceProgress::reconstruct_with(&mut history, start, |source, _, _| {
                edges
                    .get(&source)
                    .copied()
                    .ok_or_else(|| denial("missing successor"))
            })
            .expect("the immutable history selects its compiled terminal");

        assert_eq!(progress.head(), terminal);
        assert_eq!(progress.next_occurrence(), 2);
    }

    #[test]
    fn duplicate_gap_and_wrong_node_histories_fail_closed() {
        let start = entity(20);
        let middle = entity(21);
        let terminal = entity(22);
        let edges = BTreeMap::from([(start, middle), (middle, terminal)]);
        for mut history in [
            vec![completed(start, 0), completed(middle, 0)],
            vec![completed(start, 0), completed(middle, 2)],
            vec![completed(entity(99), 0)],
        ] {
            assert!(WorkflowInstanceProgress::reconstruct_with(
                &mut history,
                start,
                |source, _, _| edges
                    .get(&source)
                    .copied()
                    .ok_or_else(|| denial("missing successor")),
            )
            .is_err());
        }
    }

    #[test]
    fn retry_attempt_counts_advance_without_rescanning_history() {
        let start = entity(30);
        let mut history = [completed(start, 0), completed(start, 1)];
        let mut observed_attempts = Vec::new();
        WorkflowInstanceProgress::reconstruct_with(&mut history, start, |source, _, attempts| {
            observed_attempts.push(attempts);
            Ok(source)
        })
        .expect("retry history is contiguous");

        assert_eq!(observed_attempts, [1, 2]);
    }

    #[test]
    fn completed_history_requires_the_terminal_settlement_to_be_excluded() {
        let start = entity(40);
        let terminal = entity(41);
        let edges = BTreeMap::from([(start, terminal)]);
        let mut untrimmed = [completed(start, 0), completed(terminal, 1)];

        assert!(WorkflowInstanceProgress::reconstruct_with(
            &mut untrimmed,
            start,
            |source, _, _| edges
                .get(&source)
                .copied()
                .ok_or_else(|| denial("settled terminal has no successor")),
        )
        .is_err());
    }
}

use im::OrdMap;
#[cfg(test)]
use std::collections::BTreeMap;

use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;
use worth_relational::facade::identity::EntityId;

mod accounting;
mod collection;
mod locator;
mod navigation;
mod replay;
mod retention;
#[cfg(test)]
mod scaling;
mod update;
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};
use crate::domain_computation::primary_graph::workflow::definition::{
    CompiledWorkflowDefinition, CompiledWorkflowNodeKind,
};
pub(in crate::domain_computation::primary_graph) use locator::{
    WorkflowAssessmentEvidenceLocator, WorkflowTransitionLocator,
    WorkflowTransitionProgressObservation,
};
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
    retry_counts: OrdMap<(EntityId, ApplicationWorkflowControlOutcome), usize>,
    path_depth: u64,
    path: OrdMap<u64, WorkflowPathFrame>,
    back_blocked_by_operation: bool,
    back_edge_iterations: OrdMap<(EntityId, EntityId), u64>,
    latest_transitions: OrdMap<EntityId, WorkflowTransitionLocator>,
    latest_transition_identities: OrdMap<EntityId, (u64, String)>,
    latest_assessment_evidence: OrdMap<EntityId, WorkflowAssessmentEvidenceLocator>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct WorkflowPathFrame {
    source: EntityId,
}

impl WorkflowInstanceProgress {
    pub(in crate::domain_computation::primary_graph) fn start(
        compiled: &CompiledWorkflowDefinition,
    ) -> Self {
        Self {
            head: compiled.start().entity(),
            next_occurrence: 0,
            retry_counts: OrdMap::new(),
            path_depth: 0,
            path: OrdMap::new(),
            back_blocked_by_operation: false,
            back_edge_iterations: OrdMap::new(),
            latest_transitions: OrdMap::new(),
            latest_transition_identities: OrdMap::new(),
            latest_assessment_evidence: OrdMap::new(),
        }
    }

    pub(super) fn retained_charge_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(accounting::map_charge_bytes(&self.retry_counts))
            .saturating_add(accounting::map_charge_bytes(&self.path))
            .saturating_add(accounting::map_charge_bytes(&self.back_edge_iterations))
            .saturating_add(accounting::map_charge_bytes(&self.latest_transitions))
            .saturating_add(accounting::map_charge_bytes(
                &self.latest_transition_identities,
            ))
            .saturating_add(accounting::map_charge_bytes(
                &self.latest_assessment_evidence,
            ))
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
            retry_counts: OrdMap::new(),
            path_depth: 0,
            path: OrdMap::new(),
            back_blocked_by_operation: false,
            back_edge_iterations: OrdMap::new(),
            latest_transitions: OrdMap::new(),
            latest_transition_identities: OrdMap::new(),
            latest_assessment_evidence: OrdMap::new(),
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

    pub(in crate::domain_computation::primary_graph) const fn back_edge_iterations(
        &self,
    ) -> &OrdMap<(EntityId, EntityId), u64> {
        &self.back_edge_iterations
    }

    pub(in crate::domain_computation::primary_graph) fn latest_transition_identity(
        &self,
        node: EntityId,
    ) -> Option<&str> {
        self.latest_transition_identities
            .get(&node)
            .map(|(_, identity)| identity.as_str())
    }

    pub(in crate::domain_computation::primary_graph) fn retain_transition_identity(
        &mut self,
        node: EntityId,
        occurrence: u64,
        identity: String,
    ) {
        if self
            .latest_transition_identities
            .get(&node)
            .is_none_or(|(latest, _)| occurrence > *latest)
        {
            self.latest_transition_identities
                .insert(node, (occurrence, identity));
        }
    }

    pub(in crate::domain_computation::primary_graph) fn back_target(
        &self,
    ) -> Result<EntityId, WorthQueryApplicationAttemptDenial> {
        let depth = self.path_depth.checked_sub(1).ok_or_else(|| {
            if self.back_blocked_by_operation {
                WorthQueryApplicationAttemptDenial::new(
                    WorthQueryApplicationAttemptDenialKind::WorkflowTransitionNodeUnsupported,
                    "workflow Back cannot cross a performed operation",
                )
            } else {
                denial("workflow cannot navigate Back before a settled forward transition")
            }
        })?;
        let frame = self.path.get(&depth).ok_or_else(|| {
            denial("workflow Back path is not reconstructible from settled transitions")
        })?;
        Ok(frame.source)
    }

    pub(in crate::domain_computation::primary_graph) fn advance(
        &mut self,
        compiled: &CompiledWorkflowDefinition,
        transition: SettledWorkflowTransition,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.validate_next(transition)?;
        if transition.outcome() == ApplicationWorkflowControlOutcome::NavigatedBack {
            self.back_target()?;
            return self.advance_back();
        }
        let source = transition.node();
        let outcome = transition.outcome();
        let retry = compiled.retry_successors(source, outcome).next();
        let attempts = if retry.is_some() {
            let attempts = self.retry_counts.entry((source, outcome)).or_default();
            *attempts = attempts.saturating_add(1);
            *attempts
        } else {
            1
        };
        let successor = unique_successor(compiled, source, outcome, attempts)?.entity();
        if retry.is_some_and(|(_, maximum)| attempts <= usize::from(maximum)) {
            self.increment_back_edge(source, successor)?;
        }
        if matches!(
            compiled.node(source).map(|node| node.kind()),
            Some(CompiledWorkflowNodeKind::Operation { .. })
        ) {
            self.cross_operation_boundary();
        } else {
            self.push_forward(transition)?;
        }
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
        if transition.outcome() == ApplicationWorkflowControlOutcome::NavigatedBack {
            return self.advance_back();
        }
        let key = (transition.node(), transition.outcome());
        let attempts = self.retry_counts.entry(key).or_default();
        *attempts = attempts.saturating_add(1);
        let successor = successor(transition.node(), transition.outcome(), *attempts)?;
        if transition.operation_receipt_identity().is_some() {
            self.cross_operation_boundary();
        } else {
            self.push_forward(transition)?;
        }
        self.finish_advance(successor)
    }

    fn cross_operation_boundary(&mut self) {
        self.path.clear();
        self.path_depth = 0;
        self.back_blocked_by_operation = true;
    }

    fn push_forward(
        &mut self,
        transition: SettledWorkflowTransition,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let next_depth = self
            .path_depth
            .checked_add(1)
            .ok_or_else(|| denial("workflow Back path exceeds supported depth"))?;
        self.path.insert(
            self.path_depth,
            WorkflowPathFrame {
                source: transition.node(),
            },
        );
        self.path_depth = next_depth;
        Ok(())
    }

    fn advance_back(&mut self) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let target = self.back_target()?;
        self.increment_back_edge(self.head, target)?;
        self.path_depth -= 1;
        self.path.remove(&self.path_depth);
        self.finish_advance(target)
    }

    fn increment_back_edge(
        &mut self,
        source: EntityId,
        target: EntityId,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let iterations = self
            .back_edge_iterations
            .entry((source, target))
            .or_default();
        *iterations = iterations
            .checked_add(1)
            .ok_or_else(|| denial("workflow back-edge iteration exceeds supported range"))?;
        Ok(())
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
#[path = "progression/tests.rs"]
mod tests;

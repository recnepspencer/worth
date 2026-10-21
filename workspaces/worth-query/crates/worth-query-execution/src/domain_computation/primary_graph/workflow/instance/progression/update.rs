use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;
use worth_relational::facade::identity::{EntityId, VersionId};

use super::{
    SettledWorkflowTransition, WorkflowInstanceProgress, WorkflowInstanceProgressKey,
    WorkflowInstanceProgressRetention, WorkflowInstanceProgressRetentionDenial,
    WorkflowTransitionLocator, WorkflowTransitionProgressObservation,
    WorkflowTransitionReplayProjection,
};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptDenial;
use crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowDefinition;

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct WorkflowTransitionProgressBasis {
    key: WorkflowInstanceProgressKey,
    revision: Option<VersionId>,
    compiled: CompiledWorkflowDefinition,
    progress: WorkflowInstanceProgress,
}

impl WorkflowTransitionProgressBasis {
    pub(in crate::domain_computation::primary_graph) fn new(
        key: WorkflowInstanceProgressKey,
        revision: Option<VersionId>,
        compiled: CompiledWorkflowDefinition,
        progress: WorkflowInstanceProgress,
    ) -> Self {
        Self {
            key,
            revision,
            compiled,
            progress,
        }
    }

    pub(in crate::domain_computation::primary_graph) const fn progress(
        &self,
    ) -> &WorkflowInstanceProgress {
        &self.progress
    }

    pub(in crate::domain_computation::primary_graph) const fn replay_retention(
        &self,
    ) -> (WorkflowInstanceProgressKey, Option<VersionId>) {
        (self.key, self.revision)
    }

    pub(in crate::domain_computation::primary_graph) fn prepare(
        &self,
        node: EntityId,
        occurrence: u64,
        outcome: ApplicationWorkflowControlOutcome,
        operation_receipt_identity: Option<[u8; 32]>,
        transition_identity: String,
        transition_identity_bytes: [u8; 32],
        node_path: String,
    ) -> Result<PreparedWorkflowProgressUpdate, WorthQueryApplicationAttemptDenial> {
        let settlement =
            SettledWorkflowTransition::new(node, occurrence, outcome, operation_receipt_identity);
        let mut advanced = self.progress.clone();
        advanced.advance(&self.compiled, settlement)?;
        Ok(PreparedWorkflowProgressUpdate {
            key: self.key,
            source_revision: self.revision,
            source: self.progress.clone(),
            advanced,
            settlement,
            evidence_bytes: 0,
            captured_inputs: None,
            replay: WorkflowTransitionReplayProjection {
                identity: transition_identity,
                identity_bytes: transition_identity_bytes,
                node_path,
                // Terminal settlement has no successor and bypasses progress updates.
                terminal: false,
                navigation_back: outcome == ApplicationWorkflowControlOutcome::NavigatedBack,
                operation_receipt_identity,
            },
        })
    }

    pub(in crate::domain_computation::primary_graph) fn prepare_assessment_collection(
        &self,
        node: EntityId,
        occurrence: u64,
        transition_identity: String,
        transition_identity_bytes: [u8; 32],
        node_path: String,
    ) -> Result<PreparedWorkflowProgressUpdate, WorthQueryApplicationAttemptDenial> {
        let settlement = SettledWorkflowTransition::new(
            node,
            occurrence,
            ApplicationWorkflowControlOutcome::Completed,
            None,
        );
        let mut advanced = self.progress.clone();
        advanced.collect_assessment(&self.compiled, settlement)?;
        Ok(PreparedWorkflowProgressUpdate {
            key: self.key,
            source_revision: self.revision,
            source: self.progress.clone(),
            advanced,
            settlement,
            evidence_bytes: 0,
            captured_inputs: None,
            replay: WorkflowTransitionReplayProjection {
                identity: transition_identity,
                identity_bytes: transition_identity_bytes,
                node_path,
                terminal: false,
                navigation_back: false,
                operation_receipt_identity: None,
            },
        })
    }
}

#[derive(Clone)]
#[doc(hidden)]
pub struct PreparedWorkflowProgressUpdate {
    captured_inputs: Option<std::sync::Arc<crate::domain_computation::primary_graph::workflow::captured_input::WorkflowCapturedInputSupport>>,
    key: WorkflowInstanceProgressKey,
    source_revision: Option<VersionId>,
    source: WorkflowInstanceProgress,
    advanced: WorkflowInstanceProgress,
    settlement: SettledWorkflowTransition,
    evidence_bytes: u64,
    replay: WorkflowTransitionReplayProjection,
}

impl PreparedWorkflowProgressUpdate {
    pub(in crate::domain_computation::primary_graph) fn retaining_captured_inputs(
        mut self,
        support: Option<std::sync::Arc<crate::domain_computation::primary_graph::workflow::captured_input::WorkflowCapturedInputSupport>>,
    ) -> Self {
        self.captured_inputs = support;
        self
    }
    pub(in crate::domain_computation::primary_graph) const fn key(
        &self,
    ) -> WorkflowInstanceProgressKey {
        self.key
    }

    /// Charges the assessment evidence this step writes to the progress it
    /// commits.
    pub(in crate::domain_computation::primary_graph) const fn charging_evidence(
        mut self,
        bytes: u64,
    ) -> Self {
        self.evidence_bytes = bytes;
        self
    }

    pub(in crate::domain_computation::primary_graph) fn apply(
        self,
        retention: &mut WorkflowInstanceProgressRetention,
        committed_revision: VersionId,
        transition: EntityId,
        assessment_evidence: Option<EntityId>,
    ) -> Result<(), WorkflowInstanceProgressRetentionDenial> {
        let Self {
            key,
            source_revision,
            source,
            mut advanced,
            settlement,
            evidence_bytes,
            captured_inputs,
            replay,
        } = self;
        advanced.retain_transition_identity(
            settlement.node(),
            settlement.occurrence(),
            replay.identity.clone(),
        );
        advanced.retain_observation(
            WorkflowTransitionProgressObservation::new(
                WorkflowTransitionLocator::new(transition, settlement),
                assessment_evidence,
            )
            .retaining_evidence_bytes(evidence_bytes),
        );
        if let Some(support) = captured_inputs {
            advanced.retain_captured_inputs(
                WorkflowTransitionLocator::new(transition, settlement),
                support,
            );
        }
        retention.advance(
            key,
            source_revision,
            &source,
            Some(committed_revision),
            advanced,
            replay,
        )
    }
}

use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;
use worth_relational::facade::identity::{EntityId, VersionId};

use super::{
    SettledWorkflowTransition, WorkflowInstanceProgress, WorkflowInstanceProgressKey,
    WorkflowInstanceProgressRetention, WorkflowInstanceProgressRetentionDenial,
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
            replay: WorkflowTransitionReplayProjection {
                identity: transition_identity,
                identity_bytes: transition_identity_bytes,
                node_path,
                operation_receipt_identity,
            },
        })
    }
}

#[derive(Clone)]
#[doc(hidden)]
pub struct WorkflowTransitionReplayProjection {
    pub(in crate::domain_computation::primary_graph) identity: String,
    pub(in crate::domain_computation::primary_graph) identity_bytes: [u8; 32],
    pub(in crate::domain_computation::primary_graph) node_path: String,
    pub(in crate::domain_computation::primary_graph) operation_receipt_identity: Option<[u8; 32]>,
}

impl WorkflowTransitionReplayProjection {
    pub(super) fn retained_charge_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(self.identity.capacity())
            .saturating_add(self.node_path.capacity())
    }
}

#[derive(Clone)]
#[doc(hidden)]
pub struct PreparedWorkflowProgressUpdate {
    key: WorkflowInstanceProgressKey,
    source_revision: Option<VersionId>,
    source: WorkflowInstanceProgress,
    advanced: WorkflowInstanceProgress,
    replay: WorkflowTransitionReplayProjection,
}

impl PreparedWorkflowProgressUpdate {
    pub(in crate::domain_computation::primary_graph) const fn key(
        &self,
    ) -> WorkflowInstanceProgressKey {
        self.key
    }

    pub(in crate::domain_computation::primary_graph) fn apply(
        self,
        retention: &mut WorkflowInstanceProgressRetention,
        committed_revision: VersionId,
    ) -> Result<(), WorkflowInstanceProgressRetentionDenial> {
        let Self {
            key,
            source_revision,
            source,
            advanced,
            replay,
        } = self;
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

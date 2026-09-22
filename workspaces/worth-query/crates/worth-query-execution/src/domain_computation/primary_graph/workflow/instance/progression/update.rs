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

    pub(in crate::domain_computation::primary_graph) fn prepare(
        &self,
        node: EntityId,
        occurrence: u64,
        outcome: ApplicationWorkflowControlOutcome,
        operation_receipt_identity: Option<[u8; 32]>,
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
        })
    }
}

#[derive(Clone)]
#[doc(hidden)]
pub struct PreparedWorkflowProgressUpdate {
    key: WorkflowInstanceProgressKey,
    source_revision: Option<VersionId>,
    source: WorkflowInstanceProgress,
    advanced: WorkflowInstanceProgress,
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
        } = self;
        retention.advance(
            key,
            source_revision,
            &source,
            Some(committed_revision),
            advanced,
        )
    }
}

//! Completed history may feed a new episode, never another step on its source.
use super::super::fact::WorkflowHistoryRequirement;
use super::{
    denial, instance_binding, ObservedWorkflowInstance, WorkflowInstanceObservationPurpose,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;
use worth_relational::facade::{
    identity::EntityId, runtime::RelationalRuntime, snapshots::SnapshotHandle,
};
mod output_match;

/// Has no progression basis, replay collection or step-allowance method.
pub(in crate::domain_computation::primary_graph::application_attempt) struct CompletedWorkflowSource
{
    facts: Vec<WorthQueryApplicationObservedFact>,
    transitions: Vec<super::ObservedWorkflowTransition>,
}

impl CompletedWorkflowSource {
    pub(in crate::domain_computation::primary_graph::application_attempt) fn into_facts(
        self,
    ) -> Vec<WorthQueryApplicationObservedFact> {
        self.facts
    }
}

impl ObservedWorkflowInstance {
    pub(in crate::domain_computation::primary_graph::application_attempt) fn into_completed_source(
        mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        layout: &WorthQueryWorkflowLayout,
        instance: EntityId,
    ) -> Result<CompletedWorkflowSource, WorthQueryApplicationAttemptDenial> {
        if self.live_membership.is_some() {
            return Err(WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::WorkflowInstanceNotCompleted,
                "succession requires a completed predecessor",
            ));
        }
        if self.purpose != WorkflowInstanceObservationPurpose::Close || !self.history_materialized {
            return Err(denial("completed source requires settled command history"));
        }
        let custody = instance_binding::fence_owner_custody(
            execution,
            runtime,
            snapshot,
            layout,
            instance,
            &self.transitions,
            "succession predecessor has unsettled owner custody",
        )?;
        let Some(WorthQueryApplicationObservedFact::WorkflowHistoryBasis {
            instance: recorded,
            requirement,
            snapshot: basis,
            ..
        }) = self.facts.iter_mut().find(|fact| {
            matches!(
                fact,
                WorthQueryApplicationObservedFact::WorkflowHistoryBasis { .. }
            )
        })
        else {
            return Err(denial("completed source history is unavailable"));
        };
        if *recorded != instance
            || *requirement != WorkflowHistoryRequirement::ReplayOnly
            || basis.runtime_instance_id() != snapshot.runtime_instance_id()
            || basis.branch_id() != snapshot.branch_id()
            || basis.version_id() != snapshot.version_id()
        {
            return Err(denial("completed source history affinity changed"));
        }
        // The observer already proved Completed and absent live membership. The
        // consumed source keeps every fact, but cannot issue a step allowance.
        *requirement = WorkflowHistoryRequirement::Exact;
        self.facts.push(custody);
        Ok(CompletedWorkflowSource {
            facts: self.facts,
            transitions: self.transitions,
        })
    }
}

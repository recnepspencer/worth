//! The predecessor owner carries owned structural support onto a genuine candidate.
use super::{
    PreparedSourceFactRebase, RebaseVerificationFailure, WorthQueryApplicationObservedFact as Fact,
};
use crate::domain_computation::primary_graph::{
    application_attempt::{
        WorthQueryApplicationAttemptDenial as Denial,
        WorthQueryApplicationAttemptDenialKind as Kind,
    },
    projection_read_origin::ProjectionReadOriginRef,
};
use worth_relational::facade::runtime::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead, RelationalRuntime,
};
use worth_relational::facade::snapshots::SnapshotHandle;
type Candidate = RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>;
impl PreparedSourceFactRebase {
    pub(in crate::domain_computation::primary_graph) fn support_equal_after_own_effect(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        before: &SnapshotHandle,
        fact: &Fact,
        after_own: &Candidate,
        complete: &Candidate,
    ) -> Result<bool, Denial> {
        let refreshed = if let Some(predecessor) = &self.predecessor {
            super::predecessor_output::refresh_on_pin(
                execution,
                after_own,
                ProjectionReadOriginRef::Applied(after_own.candidate_identity().qualification()),
                fact,
                predecessor,
                |entity| {
                    after_own
                        .project_candidate()
                        .read_entity_retirement(execution, entity)
                        .map(|retired| retired.is_some())
                        .map_err(RebaseVerificationFailure::execution)
                },
            )
            .map_err(|stop| match stop.into_reason() {
                super::RebaseVerificationReason::Execution(cause) => {
                    Denial::relational_read_stopped(cause, "coupled predecessor support")
                }
                cause => Denial::new(
                    Kind::MissingAuthoritativeFact,
                    format!("coupled predecessor support: {cause:?}"),
                ),
            })?
        } else {
            None
        };
        refreshed
            .as_ref()
            .unwrap_or(fact)
            .support_equal_after_own_effect(execution, runtime, before, after_own, complete)
    }
}

//! Whole-edit support preserves the query or output owner's actual evidence.
use crate::domain_computation::primary_graph::{
    application_contribution::producer::VerifiedAppliedParent,
    application_query::AppliedQuerySupport, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationObservedFact as Fact,
};
use worth_relational::facade::{
    runtime::{RelationalAppliedCandidateReadIdentity, RelationalPinnedRead, RelationalRuntime},
    snapshots::SnapshotHandle,
};
pub(in crate::domain_computation::primary_graph) enum CandidateReadSupport {
    Query(AppliedQuerySupport),
    Parent(VerifiedAppliedParent),
}
impl CandidateReadSupport {
    pub(in crate::domain_computation::primary_graph) fn subject(&self) -> &str {
        match self {
            Self::Query(support) => support.query(),
            Self::Parent(parent) => parent.family(),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn description(&self) -> &'static str {
        match self {
            Self::Query(_) => "discovery support",
            Self::Parent(_) => "verified output support",
        }
    }
    pub(in crate::domain_computation::primary_graph) fn changed_fact(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        before: &SnapshotHandle,
        complete: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    ) -> Result<Option<&Fact>, WorthQueryApplicationAttemptDenial> {
        match self {
            Self::Query(support) => support.changed_fact(execution, runtime, before, complete),
            Self::Parent(parent) => parent.changed_fact(execution, runtime, before, complete),
        }
    }
}

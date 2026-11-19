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
    Outgoing(AppliedQuerySupport),
    Parent(VerifiedAppliedParent),
}
/// One parent output in a coupled assembly: a verified parent it retained as
/// read support, or a route's own derived output. Each route whose derivation
/// consumes that output carries it, so a refused change to the parent's
/// support or decision reads can tell whether its writer is the parent's own
/// consumer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct ParentRead(
    pub(in crate::domain_computation::primary_graph) usize,
);
impl CandidateReadSupport {
    pub(in crate::domain_computation::primary_graph) fn subject(&self) -> &str {
        match self {
            Self::Query(support) | Self::Outgoing(support) => support.query(),
            Self::Parent(parent) => parent.family(),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn description(&self) -> &'static str {
        match self {
            Self::Query(_) => "discovery support",
            Self::Outgoing(_) => "outgoing discovery support",
            Self::Parent(_) => "verified output support",
        }
    }
    /// The candidate image the read observed: its own position's evidence.
    pub(in crate::domain_computation::primary_graph) fn pin(
        &self,
    ) -> &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity> {
        match self {
            Self::Query(support) | Self::Outgoing(support) => support.pin(),
            Self::Parent(parent) => parent.pin(),
        }
    }
    /// The first observed fact whose support differs between two images.
    pub(in crate::domain_computation::primary_graph) fn changed_fact(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        before: &SnapshotHandle,
        earlier: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        image: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    ) -> Result<Option<&Fact>, WorthQueryApplicationAttemptDenial> {
        match self {
            Self::Query(support) | Self::Outgoing(support) => {
                support.changed_fact(execution, runtime, before, earlier, image)
            }
            Self::Parent(parent) => parent.changed_fact(execution, runtime, before, earlier, image),
        }
    }
}

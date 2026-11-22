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
/// One retained candidate read with the applied position it was observed at.
/// A superseded read was reopened by qualified use (PR C R2): it is checked
/// only up to the image before `window_end`, its recorded invalidator, whose
/// own exact comparison invalidated it.
pub(in crate::domain_computation::primary_graph) struct RetainedCandidateRead {
    pub(in crate::domain_computation::primary_graph) source:
        worth_relational::facade::identity::EntityId,
    pub(in crate::domain_computation::primary_graph) position: usize,
    /// The wave task whose discovery retained it, if a task owns it.
    pub(in crate::domain_computation::primary_graph) owner: Option<usize>,
    /// Its owner's incoming prerequisite selection, not its children's.
    pub(in crate::domain_computation::primary_graph) incoming: bool,
    pub(in crate::domain_computation::primary_graph) support: CandidateReadSupport,
    pub(in crate::domain_computation::primary_graph) window_end: Option<usize>,
}
/// The answer of requalifying a retained verified parent at the current image.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum RetainedReadRequalification {
    /// Its support is unchanged.
    Equal,
    /// It changed, and base settlement accepts that writer.
    Settled,
    /// It changed and is superseded at the invalidator: derive it again.
    Superseded,
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

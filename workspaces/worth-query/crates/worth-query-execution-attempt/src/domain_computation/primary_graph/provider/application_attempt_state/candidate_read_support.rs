//! Whole-edit support preserves the query or output owner's actual evidence.
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationObservedFact as Fact,
};
use crate::ports::CandidateReadSupportPort;
use worth_relational::facade::{
    runtime::{RelationalAppliedCandidateReadIdentity, RelationalPinnedRead, RelationalRuntime},
    snapshots::SnapshotHandle,
};

pub enum CandidateReadSupport {
    Query(Box<dyn CandidateReadSupportPort>),
    Outgoing(Box<dyn CandidateReadSupportPort>),
    Parent(Box<dyn CandidateReadSupportPort>),
}
/// One retained candidate read with the applied position it was observed at.
/// A superseded read was reopened by qualified use (PR C R2): it is checked
/// only up to the image before `window_end`, its recorded invalidator, whose
/// own exact comparison invalidated it.
pub struct RetainedCandidateRead {
    pub source: worth_relational::facade::identity::EntityId,
    pub position: usize,
    /// The wave task whose discovery retained it, if a task owns it.
    pub owner: Option<usize>,
    /// Its owner's incoming prerequisite selection, not its children's.
    pub incoming: bool,
    pub support: CandidateReadSupport,
    pub window_end: Option<usize>,
    /// `window_end` is the next authored stage's append that ended a root
    /// selection's stage (#238), judged by owed order, not physically: only
    /// rounds that sequentially run after that stage are outside it.
    pub stage_window: bool,
}
/// The answer of requalifying a retained verified parent at the current image.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetainedReadRequalification {
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
pub struct ParentRead(pub usize);

impl CandidateReadSupport {
    pub fn subject(&self) -> &str {
        match self {
            Self::Query(support) | Self::Outgoing(support) | Self::Parent(support) => {
                support.subject()
            }
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Query(_) => "discovery support",
            Self::Outgoing(_) => "outgoing discovery support",
            Self::Parent(_) => "verified output support",
        }
    }

    /// The candidate image the read observed: its own position's evidence.
    pub fn pin(&self) -> &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity> {
        match self {
            Self::Query(support) | Self::Outgoing(support) | Self::Parent(support) => support.pin(),
        }
    }

    /// The first observed fact whose support differs between two images.
    pub fn changed_fact(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        before: &SnapshotHandle,
        earlier: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        image: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    ) -> Result<Option<&Fact>, WorthQueryApplicationAttemptDenial> {
        match self {
            Self::Query(support) | Self::Outgoing(support) | Self::Parent(support) => {
                support.changed_fact(execution, runtime, before, earlier, image)
            }
        }
    }
}

/// Whether a mid-envelope comparison's failure must stop the edit. Only an
/// answer the intermediate candidate image cannot give is unknown: an
/// authored `MissingAuthoritativeFact` with no native execution, allocation
/// or query-read cause behind it (a source, field, selection or aggregate
/// comparison unavailable at that image). Qualified use then keeps the
/// derivation and final support, which compares every writer fact and
/// retained read strictly against the complete candidate, decides. Every
/// other failure stops: cancellation, the deadline, work, fact and capacity
/// budgets, allocation, storage, native execution, authority, and any
/// managed-index stop other than its controlled unavailability.
pub fn is_hard_comparison_stop(
    cause: &crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
) -> bool {
    !is_unavailable_comparison(cause)
}

/// The one failure class a mid-envelope comparison treats as unknown: an
/// unavailable answer, plain or from a managed index's controlled
/// unavailability. A managed-index request, storage, capacity, provider or
/// coverage stop is hard.
pub fn is_unavailable_comparison(
    cause: &crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
) -> bool {
    use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind as Kind;
    use worth_query_execution_contracts::facade::managed_index_denial::{
        WorthQueryManagedIndexStop as Stop, WorthQueryManagedIndexWorkDenial as Work,
    };
    let native = cause.relational_execution_cause().is_some()
        || cause.allocation_denial().is_some()
        || cause.native_query_read_stop().is_some();
    match cause.kind() {
        Kind::MissingAuthoritativeFact => !native,
        Kind::ManagedIndexSourceDenied => matches!(
            cause.managed_index_denial(),
            Some(Work::Controlled(
                Stop::MembershipUnavailable
                    | Stop::RefinementUnavailable
                    | Stop::SummaryUnavailable
                    | Stop::SourceSlotUnavailable
                    | Stop::IncrementalUnavailable
                    | Stop::SourceStructureUnavailable
            ))
        ),
        _ => false,
    }
}

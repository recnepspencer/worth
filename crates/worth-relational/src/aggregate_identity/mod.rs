//! One owner for committed aggregate naming and occurrence-qualified lineage.
mod changes;
mod cohorts;
mod consequence_plan;
mod funding;
mod materialization;
mod observations;
mod pinned;
mod preparation;
mod projection;
mod proposal;
mod read;
mod retention_extent;
mod stored_lineage;
mod stored_read;
mod stored_record;
mod transition;
pub use observations::AggregateReadObservations;
pub use pinned::{
    PinnedAggregateRead, PinnedAggregateReadDenial, PinnedAggregateReadDenialKind,
    PinnedAggregateReadScope,
};
pub(crate) use preparation::AggregateAdmission;
pub(crate) use proposal::{AggregateInvocation, AggregateProposal, AggregateProposalBorrow};
pub use read::{AggregateRead, AggregateReadDenial, AggregateReadLineageEvent, AggregateReadPort};
pub(crate) mod guard;
pub(crate) mod record;

pub(crate) use transition::{
    transition, AggregateAssignment, AggregateLineageEvent, AggregateLineageKind,
    AggregateTransitionDenial, PriorAggregate,
};

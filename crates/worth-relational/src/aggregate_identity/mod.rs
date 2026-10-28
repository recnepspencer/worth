//! One owner for committed aggregate naming and occurrence-qualified lineage.
mod changes;
mod cohorts;
mod consequence_plan;
mod derived_source;
mod final_laws;
mod funding;
pub(crate) use final_laws::validate_final_derived_laws;
mod materialization;
mod membership;
mod observations;
mod pinned;
mod preparation;
pub(crate) mod projected_storage;
mod projection;
mod proposal;
mod read;
mod retention_extent;
mod selection;
mod stored_lineage;
mod stored_read;
mod stored_record;
mod transition;
pub use observations::AggregateReadObservations;
pub use pinned::{
    PinnedAggregateLineageRead, PinnedAggregateLineageScope, PinnedAggregateRead,
    PinnedAggregateReadDenial, PinnedAggregateReadDenialKind, PinnedAggregateReadScope,
};
pub(crate) use preparation::AggregateAdmission;
pub(crate) use proposal::{AggregateInvocation, AggregateProposal, AggregateProposalBorrow};
pub use read::{
    AggregateCensusRead, AggregateCensusReadIdentity, AggregateRead, AggregateReadCollection,
    AggregateReadDenial, AggregateReadLineageEvent, AggregateReadPort, PreparedAggregateCensus,
};
pub(crate) mod guard;
pub(crate) mod record;

pub(crate) use transition::{
    transition, AggregateAssignment, AggregateLineageEvent, AggregateLineageKind,
    AggregateTransitionDenial, PriorAggregate,
};

//! One owner for committed aggregate naming and occurrence-qualified lineage.
mod cohorts;
mod consequence_plan;
mod funding;
mod materialization;
mod preparation;
mod retention_extent;
mod transition;
pub(crate) use preparation::{prepare, AggregateAdmission};
pub(crate) mod guard;
pub(crate) mod record;

pub(crate) use transition::{
    transition, AggregateAssignment, AggregateLineageEvent, AggregateLineageKind,
    AggregateTransitionDenial, PriorAggregate,
};

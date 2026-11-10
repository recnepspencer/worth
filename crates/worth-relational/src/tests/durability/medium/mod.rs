//! Publication to, and recovery from, a durable medium.

// CONTRACT: durable_medium
// LANES: success, failure, recovery

mod branch_transition_races;
mod lineage_frontiers;
mod slow_publication;

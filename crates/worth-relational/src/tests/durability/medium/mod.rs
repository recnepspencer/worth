//! Publication to, and recovery from, a durable medium.

// CONTRACT: durable_medium
// LANES: success, failure, recovery

mod branch_transition_races;
mod cold_merge_history_matrix;
mod durable_before_live;
mod lifecycle_lock_order;
mod recovery_refusals;
mod reservation_wait;
mod slow_publication;
mod unresolved_fate;

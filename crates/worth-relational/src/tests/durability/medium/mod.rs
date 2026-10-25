//! Publication to, and recovery from, a durable medium.

// CONTRACT: durable_medium
// LANES: success, failure, recovery

mod durable_before_live;
mod recovered_publication;
mod recovery_refusals;
mod reservation_wait;
mod single_owner;
mod unresolved_fate;

mod branch_transition_races;
mod cold_members;
mod lifecycle_lock_order;
mod slow_publication;

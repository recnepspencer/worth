//! Publication to, and recovery from, a durable medium.

// CONTRACT: durable_medium
// LANES: success, failure, recovery

mod durable_before_live;
mod recovered_publication;
mod recovery_refusals;
mod unresolved_fate;

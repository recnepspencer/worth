//! Deterministic schedules over the real owner boundaries.

#[path = "operation_control/cancellation.rs"]
mod cancellation;
#[path = "operation_control/cancellation_progress.rs"]
mod cancellation_progress;
#[path = "operation_control/close.rs"]
mod close;
#[path = "operation_control/close_progress.rs"]
mod close_progress;
#[path = "operation_control/independent_progress.rs"]
mod independent_progress;
#[path = "operation_control/observation_race.rs"]
mod observation_race;
#[path = "operation_control/ordered_race.rs"]
mod ordered_race;
#[path = "operation_control/ordered_races.rs"]
mod ordered_races;
#[path = "operation_control/panic.rs"]
mod panic;
#[path = "operation_control/races.rs"]
mod races;
#[path = "operation_control/snapshot_progress.rs"]
mod snapshot_progress;

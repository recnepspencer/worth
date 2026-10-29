#[path = "operation_control/callback_rollback.rs"]
// Deterministic schedules over the real owner boundaries.
pub(crate) mod callback_rollback;

#[path = "operation_control/cancellation.rs"]
pub(crate) mod cancellation;

#[path = "operation_control/cancellation_progress.rs"]
pub(crate) mod cancellation_progress;

#[path = "operation_control/close.rs"]
pub(crate) mod close;

#[path = "operation_control/close_progress.rs"]
pub(crate) mod close_progress;

#[path = "operation_control/independent_progress.rs"]
pub(crate) mod independent_progress;

#[path = "operation_control/observation_race.rs"]
pub(crate) mod observation_race;

#[path = "../../../fixtures/signal_owner_services/adversarial/operation_control/ordered_race.rs"]
pub(crate) mod ordered_race;

#[allow(unused_imports)]
pub(crate) use crate::exhaustive::signal_owner_services::adversarial::operation_control::ordered_races;

#[path = "operation_control/panic.rs"]
pub(crate) mod panic;

#[path = "operation_control/races.rs"]
pub(crate) mod races;

#[allow(unused_imports)]
pub(crate) use crate::exhaustive::signal_owner_services::adversarial::operation_control::snapshot_progress;

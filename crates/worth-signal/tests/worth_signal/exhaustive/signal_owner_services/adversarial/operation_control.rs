// Deterministic schedules over the real owner boundaries.
#[allow(unused_imports)]
pub(crate) use crate::fast::signal_owner_services::adversarial::operation_control::*;

#[path = "operation_control/cancellation.rs"]
pub(crate) mod cancellation;

#[path = "operation_control/independent_progress.rs"]
pub(crate) mod independent_progress;

#[path = "operation_control/ordered_races.rs"]
pub(crate) mod ordered_races;

#[path = "operation_control/panic.rs"]
pub(crate) mod panic;

#[path = "operation_control/snapshot_progress.rs"]
pub(crate) mod snapshot_progress;

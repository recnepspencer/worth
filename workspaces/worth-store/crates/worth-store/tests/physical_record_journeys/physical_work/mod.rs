mod authority;
mod authority_mutants;
mod authority_sealing;
mod capacity;
mod close_plan;
mod concurrency;
mod durability_signal_binding;
mod e2e_trace;
mod execution_capability;
mod executor;
pub(crate) mod failure;
mod fault_fixture;
mod fixture;
mod invalidation;
mod lifecycle;
mod locality;
mod phase_16_lifecycle_maelstrom;
mod post_dispatch_cancellation;
mod profile;
mod read_partition_failures;
mod read_partitions;
mod readiness;
mod record_read_cancellation;
mod record_read_damage;
mod record_read_failures;
mod record_read_path;
mod record_read_signal_cleanup;
mod residency_generation_fencing;
mod residency_pressure_projection;
mod residency_projection_failure;
mod residency_writeback_lifecycle;
mod residency_writeback_retry;
mod scan_continuation_damage;
mod scheduler;
mod scheduler_reservation;
mod serving_frame_residency;
mod speculative_residency;

pub(super) use super::{configuration, media, serving_from_initialization, success};
pub(super) use fixture::{serving_from_initialization_with_work_profile, work_fixture};
pub(super) use scheduler::policy_receipt;
pub(super) use scheduler_reservation::{reserved_buffered_file_read, reserved_page_write};

pub(crate) fn phase_16_maelstrom_reopener(root: &std::path::Path) {
    phase_16_lifecycle_maelstrom::fresh_process::reopener(root);
}

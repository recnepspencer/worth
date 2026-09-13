#[path = "history/artifacts.rs"]
mod artifacts;
#[path = "history/parent_oracle.rs"]
mod parent_oracle;
#[path = "history/persisted_fates.rs"]
mod persisted_fates;
#[path = "history/physical_history.rs"]
mod physical_history;
#[path = "history/process.rs"]
mod process;
#[path = "history/raw_in_flight.rs"]
mod raw_in_flight;
#[path = "history/schedule.rs"]
mod schedule;
#[path = "history/writer_history.rs"]
mod writer_history;
#[path = "history/writer_process.rs"]
mod writer_process;

pub(super) const ARTIFACT_SET_DOMAIN: &[u8] = b"worth.store.recovery-observer.artifact-set.v1";
pub(super) const ARTIFACT_IDENTITY_DOMAIN: &[u8] =
    b"worth.store.recovery-observer.artifact-identity.v1";
pub(super) const DEFAULT_OPERATION_COUNT: usize = 96;

pub(super) use parent_oracle::{
    capture_cleanup_candidate, verify_cleanup_preserved, verify_cleanup_transition,
    CleanupCandidateProof, CleanupTransitionProof,
};
pub(super) use persisted_fates::classify_persisted_fates;
pub(super) use physical_history::ParentPhysicalHistory;
pub(super) use process::{
    launch_killed_cleanup_writer_with_operation_count,
    launch_killed_durable_unacknowledged_writer_with_operation_count,
    launch_killed_mutation_writer, launch_killed_mutation_writer_with_operation_count,
    launch_killed_production_writer, launch_killed_production_writer_with_operation_count,
    KilledProductionWriter,
};
pub(super) use raw_in_flight::InFlightMutationFate;
pub(super) use schedule::{create_checkpoint_operation_program, MutationCrashWorkload};
pub(super) use writer_history::{ExpectedWriterHistory, SubmittedOperationProgram};
pub(super) use writer_process::c8_writer_binary_path;

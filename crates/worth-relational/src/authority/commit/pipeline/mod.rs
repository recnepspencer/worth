pub(crate) mod artifact_execution;
mod authority_context;
mod boundary_validation;
mod bulk_mutation_telemetry;
pub(crate) mod candidate_preparation;
mod draft_execution;
mod draft_preparation_phase;
mod execution;
pub(crate) mod execution_admission;
mod history_binding;
pub(crate) mod history_preparation;
mod history_resolution_phase;
pub(crate) mod merge_draft;
mod mutation_execution;
pub(crate) mod performed_outcome;
mod phase_backing;
pub(crate) mod publication_execution;
mod rejection;
mod semantic_result;
mod snapshot_validation;

pub(crate) use crate::execution::request_custody::request_lease;
pub(crate) use authority_context::{AuthoritativeCommitContext, SelectedCommitMutation};
pub(crate) use execution::execute_authoritative_commit;
pub(crate) use publication_execution::PreparedCommitPublicationCompletion;
pub(crate) use publication_execution::PreparedCommitPublicationExecution;
pub(crate) use publication_execution::{publish_commit_execution, PublishedCommitExecution};

mod request_checkpoints;
#[cfg(test)]
pub(crate) use request_checkpoints::observation as stage_observation;
pub(crate) use request_checkpoints::{consult_stage, CommitStage};

pub(crate) use candidate_preparation::prepare_consulted_authoritative_commit;
pub use phase_backing::RelationalCommitPhaseBackingForTest;
pub(crate) use phase_backing::{CommitPhaseBacking, CommitPhaseReservation};
#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;

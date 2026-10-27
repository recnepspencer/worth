mod artifact_execution;
mod authority_context;
mod boundary_validation;
mod bulk_mutation_telemetry;
mod draft_execution;
mod draft_preparation_phase;
mod execution;
mod execution_admission;
mod history_binding;
mod history_resolution_phase;
mod invariant_phase;
mod merge_draft;
mod mutation_execution;
mod publication_execution;
mod rejection;
mod request_resources;
mod result_assembly;
mod snapshot_validation;

pub(crate) use authority_context::AuthoritativeCommitContext;
pub(crate) use execution::{
    execute_authoritative_commit, prepare_consulted_authoritative_commit,
    publish_prepared_authoritative_commit,
};
pub(crate) use publication_execution::publish_commit_execution;
pub(crate) use publication_execution::PreparedCommitPublicationCompletion;
pub(crate) use publication_execution::PreparedCommitPublicationExecution;
pub(crate) use request_resources::request_lease;
pub(crate) use result_assembly::assemble_commit_result;
pub(crate) use result_assembly::CommitResultSeal;

mod request_checkpoints;
#[cfg(test)]
pub(crate) use request_checkpoints::observation as stage_observation;
pub(crate) use request_checkpoints::{consult_stage, consult_stage_control, CommitStage};

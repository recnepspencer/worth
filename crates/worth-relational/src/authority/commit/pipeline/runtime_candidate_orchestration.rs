//! Issue one runtime-bound candidate through preparation and registration.
use worth_relational_contracts::internal_port::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use super::authority_context::AuthoritativeCommitContext;
use super::candidate_preparation::stages;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_execution::ExecutionRequest;

pub(crate) fn prepare_consulted_authoritative_commit(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalPreparationRuntime,
    mut context: AuthoritativeCommitContext<crate::mvcc::RelationalTransactionValidationInput>,
) -> Result<crate::mvcc::publication::PreparedRelationalCommitCandidate, TransactionCommitError> {
    if context.authority_input.retained().is_some()
        && runtime.durability_admission.has_attached_medium()
    {
        return Err(TransactionCommitError::publication_denied(
            crate::mvcc::publication::outcome::refusals::RelationalPublicationDenial::RetainedEntitySelectionDurabilityUnsupported,
        ));
    }
    runtime
        .durability_admission
        .admit_home_program_write(
            context
                .validation_input
                .home_step()
                .map(std::sync::Arc::as_ref),
            context
                .validation_input
                .recovered_codec_publication_custody()
                .map(std::sync::Arc::as_ref),
        )
        .map_err(|cause| {
            TransactionCommitError::publication_denied(
                crate::mvcc::publication::outcome::refusals::RelationalPublicationDenial::HomeProgramTransition(cause),
            )
        })?;
    if let Some(step) = context.validation_input.home_step()
        && !context
            .caller_association
            .as_ref()
            .is_some_and(|association| step.admits_association(association.as_bytes()))
    {
        return Err(TransactionCommitError::publication_denied(
            crate::mvcc::publication::outcome::refusals::RelationalPublicationDenial::HomeProgramTransition(
                crate::durability::medium::home_write_denial::RelationalHomeProgramWriteDenial::TransitionRequired,
            ),
        ));
    }
    let original = context
        .recovered_index_catalog
        .take()
        .map(|catalog| catalog.original_for(runtime.runtime_instance_id()))
        .transpose()?;
    let history = super::candidate_preparation::prepare_history(
        execution,
        runtime.candidate_history_inputs(),
        context,
    )?;
    let (symbols, validated) = stages::validate(execution, history)?;
    let artifact_owners = runtime.bind_candidate_artifacts(&symbols, original.as_ref());
    let assembled = stages::assemble(execution, artifact_owners, validated)?;
    let publication_owners = runtime.publication_phase_owners(&symbols.dictionary);
    let published = stages::publish(execution, publication_owners, assembled)?;
    stages::register(
        execution,
        runtime.candidate_registration_inputs(&symbols.dictionary),
        published,
    )
}

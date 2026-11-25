//! A history document checks a pre-admitted stage (a correction's corrective
//! stage) with the same owner every stage it admits goes through (plan §2.2,
//! census gap 1.5.2): the operation is enrolled, its model is the document's,
//! and its scope's owner at the head is the document's model.
use super::*;
type Failure = crate::application_entry::WorthQueryPerformedMutationExecutionDenial;

pub(super) fn require_admitted<S, B>(
    context: &EnvelopeContext<'_, S>,
    phase: &WorthQueryAdvancementPhase<'_>,
    ordinal: usize,
    admitted: &AuthoredEditStageAdmission<S, B>,
) -> Result<(), Failure>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
{
    let Some(restriction) = &context.history else {
        return Ok(());
    };
    let access = worth_query_execution::publication_boundary::program_publication_access();
    let Err(denial) = context.application.validate_history_envelope_stage::<B>(
        phase,
        &access,
        restriction,
        ordinal,
        admitted.operation(),
    ) else {
        return Ok(());
    };
    // As for a stage admitted here, the typed reason stays with the history owner.
    restriction.record_refusal(ordinal, denial);
    Err(Failure::Mutation(WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Handler(HandlerExecutionDenial::new(
            stage_admission_refusal::<S, B>(&access, ordinal, "document history refused this stage".to_owned()),
        )),
    )))
}

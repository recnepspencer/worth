//! Native decision and candidate authoring for the declared retention mutation.
use super::*;
use worth_query_host::facade::declaration::application_operation::ApplicationCandidateRequirements;

pub struct SetRetentionHandler;

impl OperationHandler<DocumentRetentionSchema, SetRetentionBinding> for SetRetentionHandler {
    fn decide(
        &self,
        input: &SetRetentionInput,
        reader: &mut DecisionReader<'_, '_, '_, DocumentRetentionSchema, SetRetentionBinding>,
    ) -> HandlerResult<
        WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>,
        SetRetentionDenial,
    > {
        candidate_tracking::record_decision(input.retention_days);
        let document = match reader
            .resolve_entity(DocumentIdentityField::reference(), input.identity.clone())
        {
            Ok(document) => document,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        match reader.field(&document, DocumentRetentionField::reference()) {
            Ok(Some(current)) if current == input.retention_days => {
                return HandlerResult::DomainDenied(SetRetentionDenial::RetentionUnchanged)
            }
            Ok(_) => {}
            Err(error) => return HandlerResult::ExecutionDenied(error),
        }
        match reader.mutation_target(&document) {
            Ok(target) => HandlerResult::Completed(target),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }

    fn candidate_requirements(
        &self,
        _: &SetRetentionInput,
        _: &WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>,
    ) -> ApplicationCandidateRequirements {
        SetRetentionBinding::CANDIDATES
            .finite_requirements()
            .expect("fixed-shape handler declares finite candidate policy")
    }

    fn build_candidate(
        &self,
        input: &SetRetentionInput,
        target: WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>,
        writer: &mut CandidateWriter<'_, DocumentRetentionSchema, SetRetentionBinding>,
    ) -> HandlerResult<DocumentRetentionWritten, SetRetentionDenial> {
        candidate_tracking::record_candidate(input.retention_days);
        let document = match writer.projected_entity(&target) {
            Ok(document) => document,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        if let Err(error) = writer.write_field(
            &document,
            DocumentRetentionField::reference(),
            input.retention_days,
        ) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
        }
        HandlerResult::Completed(DocumentRetentionWritten {
            retention_days: input.retention_days,
        })
    }
}

//! Native decision and candidate authoring for the declared retention mutation.
pub use super::*;

#[path = "set_retention_handler/observation.rs"]
mod observation;
pub use observation::RetentionHandlerObservationForTest;

pub struct SetRetentionHandler {
    contacts: Option<std::sync::Arc<observation::Contacts>>,
}

impl Default for SetRetentionHandler {
    fn default() -> Self {
        Self {
            contacts: observation::capture(),
        }
    }
}

impl SetRetentionHandler {
    /// Place a readonly observer during the ordinary synchronous installation.
    /// It is captured once by this receiver, never consulted for runtime decisions.
    pub fn observe_installation_for_test<T>(
        build: impl FnOnce() -> T,
    ) -> (T, RetentionHandlerObservationForTest) {
        observation::install(build)
    }
}

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
        if let Some(contacts) = &self.contacts {
            contacts.decision();
        }
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

    fn build_candidate(
        &self,
        input: &SetRetentionInput,
        target: WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>,
        writer: &mut CandidateWriter<'_, DocumentRetentionSchema, SetRetentionBinding>,
    ) -> HandlerResult<DocumentRetentionWritten, SetRetentionDenial> {
        candidate_tracking::record_candidate(input.retention_days);
        if let Some(contacts) = &self.contacts {
            contacts.candidate();
        }
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

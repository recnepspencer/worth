//! A real native document write with receiver-owned independent contact counts.
use super::schema::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_query_host::facade::{declaration::application_operation::*, primary_graph::*};
#[derive(Default)]
pub struct Contacts {
    decision: AtomicUsize,
    candidate: AtomicUsize,
}
impl Contacts {
    pub fn counts(&self) -> (usize, usize) {
        (
            self.decision.load(Ordering::Relaxed),
            self.candidate.load(Ordering::Relaxed),
        )
    }
}
pub struct RetentionHandler(pub Arc<Contacts>);
impl<B> OperationHandler<CapabilitySchema, B> for RetentionHandler
where
    B: ApplicationMutationBinding<
        CapabilitySchema,
        Input = SetRetentionInput,
        Decision = WorthQueryInvariantMutationTarget<CapabilitySchema, Document>,
        Denial = SetRetentionDenial,
        Result = DocumentRetentionWritten,
        Operation = SetRetention,
    >,
{
    fn decide(
        &self,
        input: &SetRetentionInput,
        reader: &mut DecisionReader<'_, '_, '_, CapabilitySchema, B>,
    ) -> HandlerResult<
        WorthQueryInvariantMutationTarget<CapabilitySchema, Document>,
        SetRetentionDenial,
    > {
        self.0.decision.fetch_add(1, Ordering::Relaxed);
        let document = match reader
            .resolve_entity(DocumentIdentityField::reference(), input.identity.clone())
        {
            Ok(document) => document,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        if let Err(error) = reader.field(&document, DocumentIdentityField::reference()) {
            return HandlerResult::ExecutionDenied(error);
        }
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
        _: &WorthQueryInvariantMutationTarget<CapabilitySchema, Document>,
    ) -> ApplicationCandidateRequirements {
        B::CANDIDATES
            .fixed_requirements()
            .expect("fixed candidate contract")
    }
    fn build_candidate(
        &self,
        input: &SetRetentionInput,
        target: WorthQueryInvariantMutationTarget<CapabilitySchema, Document>,
        writer: &mut CandidateWriter<'_, CapabilitySchema, B>,
    ) -> HandlerResult<DocumentRetentionWritten, SetRetentionDenial> {
        self.0.candidate.fetch_add(1, Ordering::Relaxed);
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

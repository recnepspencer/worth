//! The guarded counterpart of the ordinary retention write used by workflow courts.

use worth_query_host::facade::{
    declaration::{
        application_operation::{
            ApplicationCandidateRequirements, ApplicationMutationBinding,
            ApplicationMutationIntent, NoApplicationMutationOutputs, NoApplicationMutationSource,
        },
        application_schema::{
            ApplicationFieldRef, ApplicationPrincipalBindingRef, EqualityPredicate,
            NoApplicationUnit, ReadOnly, U64ApplicationValueBinding,
        },
    },
    primary_graph::{
        CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
        WorthQueryInvariantMutationTarget,
    },
};

use super::super::schema::{
    Document, DocumentFacts, DocumentIdentityField, DocumentPrincipalBinding,
    DocumentRetentionField, DocumentRetentionSchema, ExternalMapping, Principal, SetRetention,
    SetRetentionInputBinding,
};
use super::{
    DocumentRetentionWritten, SetRetentionBinding, SetRetentionDenial, SetRetentionInput,
    SetRetentionScope,
};

#[derive(Clone, Debug)]
pub struct ReviewedSetRetentionIntent {
    pub input: SetRetentionInput,
}

pub struct ReviewedSetRetentionBinding;

impl ApplicationMutationBinding<DocumentRetentionSchema> for ReviewedSetRetentionBinding {
    type Input = SetRetentionInput;
    type InputBinding = SetRetentionInputBinding;
    type Result = DocumentRetentionWritten;
    type ResultBinding = super::DocumentRetentionWrittenBinding;
    type IdempotencyKey = u64;
    type Operation = SetRetention;
    type Decision = WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>;
    type Denial = SetRetentionDenial;
    type DenialBinding = super::SetRetentionDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = SetRetentionScope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;

    const IDENTITY: &'static str =
        "worth.query.certification.document-retention.reviewed-set-binding.v1";
    const HANDLER_IDENTITY: &'static str =
        "worth.query.certification.document-retention.reviewed-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.certification.document-retention.reviewed-command.v1";
    const REQUIRES_WORKFLOW_AUTHORITY: bool = true;
    const CANDIDATES: ApplicationCandidateRequirements = SetRetentionBinding::CANDIDATES;

    fn idempotency_key_identity(key: &u64) -> [u8; 32] {
        SetRetentionBinding::idempotency_key_identity(key)
    }

    fn input_identity(input: &SetRetentionInput) -> [u8; 32] {
        SetRetentionBinding::input_identity(input)
    }

    fn scope_field() -> ApplicationFieldRef<
        DocumentRetentionSchema,
        Document,
        DocumentFacts,
        DocumentIdentityField,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        DocumentIdentityField::reference()
    }

    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DocumentRetentionSchema,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DocumentPrincipalBinding::reference()
    }
}

impl ApplicationMutationIntent<DocumentRetentionSchema> for ReviewedSetRetentionIntent {
    type Binding = ReviewedSetRetentionBinding;

    fn input(&self) -> &SetRetentionInput {
        &self.input
    }

    fn scope_binding(&self) -> SetRetentionScope {
        SetRetentionScope::new(
            DocumentIdentityField::reference(),
            self.input.identity.clone(),
        )
    }
}

pub struct ReviewedSetRetentionHandler;

impl OperationHandler<DocumentRetentionSchema, ReviewedSetRetentionBinding>
    for ReviewedSetRetentionHandler
{
    fn decide(
        &self,
        input: &SetRetentionInput,
        reader: &mut DecisionReader<
            '_,
            '_,
            '_,
            DocumentRetentionSchema,
            ReviewedSetRetentionBinding,
        >,
    ) -> HandlerResult<
        WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>,
        SetRetentionDenial,
    > {
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
        ReviewedSetRetentionBinding::CANDIDATES
    }

    fn build_candidate(
        &self,
        input: &SetRetentionInput,
        target: WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>,
        writer: &mut CandidateWriter<'_, DocumentRetentionSchema, ReviewedSetRetentionBinding>,
    ) -> HandlerResult<DocumentRetentionWritten, SetRetentionDenial> {
        super::candidate_tracking::record_candidate(input.retention_days);
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

//! Separate ordinary assessment action; the existing conditional operation stays exclusive.
use super::*;

worth_query_operation!(pub PublishOrdinaryRetentionAssessment for DocumentRetentionSchema, input RetentionAssessmentInputBinding);
worth_query_operation_reads!(PublishOrdinaryRetentionAssessment => [Document, DocumentIdentityField, DocumentRetentionField]);

pub struct OrdinaryRetentionAssessmentBinding;
impl ApplicationMutationBinding<DocumentRetentionSchema> for OrdinaryRetentionAssessmentBinding {
    type Input = RetentionAssessmentInput;
    type InputBinding = RetentionAssessmentInputBinding;
    type Result = RetentionAssessmentPublished;
    type ResultBinding = RetentionAssessmentPublishedBinding;
    type IdempotencyKey = u64;
    type Operation = PublishOrdinaryRetentionAssessment;
    type Decision = WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>;
    type Denial = RetentionAssessmentDenial;
    type DenialBinding = RetentionAssessmentDenialBinding;
    type Output = RetentionAssessmentOutputs;
    type ScopeBinding = RetentionAssessmentScope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<DocumentRetentionQuery>;

    const IDENTITY: &'static str =
        "worth.query.certification.ordinary-retention-assessment.binding.v1";
    const HANDLER_IDENTITY: &'static str =
        "worth.query.certification.ordinary-retention-assessment.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.certification.ordinary-retention-assessment.command.v1";
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 0, 0),
            ApplicationCandidateResourceCeiling::bounded(512, 256),
        );

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

#[derive(Clone)]
pub struct OrdinaryRetentionAssessmentIntent(pub RetentionAssessmentInput);
impl ApplicationMutationIntent<DocumentRetentionSchema> for OrdinaryRetentionAssessmentIntent {
    type Binding = OrdinaryRetentionAssessmentBinding;
    fn input(&self) -> &RetentionAssessmentInput {
        &self.0
    }
    fn scope_binding(&self) -> RetentionAssessmentScope {
        RetentionAssessmentScope::new(DocumentIdentityField::reference(), self.0.identity.clone())
    }
}

pub struct OrdinaryRetentionAssessmentHandler;

impl OperationHandler<DocumentRetentionSchema, OrdinaryRetentionAssessmentBinding>
    for OrdinaryRetentionAssessmentHandler
{
    fn decide(
        &self,
        input: &RetentionAssessmentInput,
        reader: &mut DecisionReader<
            '_,
            '_,
            '_,
            DocumentRetentionSchema,
            OrdinaryRetentionAssessmentBinding,
        >,
    ) -> HandlerResult<
        WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>,
        RetentionAssessmentDenial,
    > {
        let document = match reader
            .resolve_entity(DocumentIdentityField::reference(), input.identity.clone())
        {
            Ok(document) => document,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        match reader.field(&document, DocumentRetentionField::reference()) {
            Ok(Some(retention_days)) if retention_days == input.retention_days => {}
            Ok(_) => return HandlerResult::DomainDenied(RetentionAssessmentDenial::SourceChanged),
            Err(error) => return HandlerResult::ExecutionDenied(error),
        }
        match reader.mutation_target(&document) {
            Ok(target) => HandlerResult::Completed(target),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }

    fn candidate_requirements(
        &self,
        _: &RetentionAssessmentInput,
        _: &WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>,
    ) -> ApplicationCandidateRequirements {
        OrdinaryRetentionAssessmentBinding::CANDIDATES
    }

    fn build_candidate(
        &self,
        input: &RetentionAssessmentInput,
        target: WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>,
        writer: &mut CandidateWriter<
            '_,
            DocumentRetentionSchema,
            OrdinaryRetentionAssessmentBinding,
        >,
    ) -> HandlerResult<RetentionAssessmentPublished, RetentionAssessmentDenial> {
        let document = match writer.projected_entity(&target) {
            Ok(document) => document,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        if let Err(error) = writer.preserve_output::<AssessmentOutput>(&document) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
        }
        HandlerResult::Completed(RetentionAssessmentPublished {
            retention_days: input.retention_days,
        })
    }
}

pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema> {
    schema
        .operation(
            PublishOrdinaryRetentionAssessment::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_decision_fact_budget(PublishOrdinaryRetentionAssessment::reference(), 8)
        .operation_projection_work_budget(PublishOrdinaryRetentionAssessment::reference(), 8)
        .operation_read_field(
            PublishOrdinaryRetentionAssessment::reference(),
            DocumentIdentityField::reference(),
        )
        .operation_read_field(
            PublishOrdinaryRetentionAssessment::reference(),
            DocumentRetentionField::reference(),
        )
        .application_mutation_binding::<OrdinaryRetentionAssessmentBinding>()
}

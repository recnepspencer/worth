//! The public request surface a caller uses to set and read a document's retention.
//!
//! One mutation binding writes the retention and one query binding reports it.
//! Both are declared on the schema, so every court step goes through the same
//! ordinary application entry an external consumer would use.

use worth_query_host::facade::{
    declaration::{
        application_operation::{
            ApplicationCandidateCardinalityCeiling, ApplicationCandidateRequirements,
            ApplicationCandidateResourceCeiling, ApplicationMutationBinding,
            ApplicationMutationFieldScope, ApplicationMutationIntent, NoApplicationMutationOutputs,
            NoApplicationMutationSource,
        },
        application_query::{
            ApplicationQueryBinding, ApplicationQueryBindingLimits, ApplicationQueryFieldScope,
            ApplicationQueryIntent, ApplicationQueryParameterSet,
        },
        application_schema::{
            ApplicationFieldRef, ApplicationPrincipalBindingRef,
            ApplicationSchemaDeclarationBuilder, EqualityPredicate, NoApplicationUnit, ReadOnly,
            U64ApplicationValueBinding,
        },
    },
    primary_graph::{
        CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
        WorthQueryInvariantMutationTarget,
    },
    worth_query_structured_value_binding,
};

use super::schema::{
    Document, DocumentFacts, DocumentIdentityField, DocumentPrincipalBinding,
    DocumentQueryParametersBinding, DocumentRetentionConditionBinding,
    DocumentRetentionConditionQuery, DocumentRetentionField, DocumentRetentionQuery,
    DocumentRetentionRowBinding, DocumentRetentionSchema, ExternalMapping, Principal, SetRetention,
    SetRetentionInput, SetRetentionInputBinding,
};

#[path = "retention_entry/candidate_tracking.rs"]
mod candidate_tracking;
pub use candidate_tracking::{candidate_count, reset_candidate_count};
#[path = "retention_entry/reviewed_write.rs"]
mod reviewed_write;
pub use reviewed_write::{
    ReviewedSetRetentionBinding, ReviewedSetRetentionHandler, ReviewedSetRetentionIntent,
};

/// The one document every host in this court seeds and both programs act on.
pub const DOCUMENT_IDENTITY: &str = "document-1";
pub const RELATED_DOCUMENT_IDENTITY: &str = "document-2";
/// Reserved fixture value whose candidate-authoring count proves migration recovery does not replay.
pub const MIGRATION_CANDIDATE_PROBE_RETENTION: u64 = 17;

#[derive(Clone, Debug)]
pub struct DocumentRetentionRead {
    pub identity: String,
}

worth_query_structured_value_binding!(pub DocumentRetentionReadBinding for DocumentRetentionRead {
    identity: "worth.query.certification.document-retention.read.v1"
});

pub struct DocumentRetentionQueryBinding;
pub struct DocumentRetentionConditionQueryBinding;

type DocumentRetentionQueryScope = ApplicationQueryFieldScope<
    DocumentRetentionSchema,
    Document,
    DocumentFacts,
    DocumentIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;

impl ApplicationQueryBinding<DocumentRetentionSchema> for DocumentRetentionQueryBinding {
    type Input = DocumentRetentionRead;
    type InputBinding = DocumentRetentionReadBinding;
    type Query = DocumentRetentionQuery;
    type ParameterBinding = DocumentQueryParametersBinding;
    type ResultBinding = DocumentRetentionRowBinding;
    type ScopeBinding = DocumentRetentionQueryScope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;

    const IDENTITY: &'static str = "worth.query.certification.document-retention.read-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::bounded(1, 64);

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

impl ApplicationQueryBinding<DocumentRetentionSchema> for DocumentRetentionConditionQueryBinding {
    type Input = DocumentRetentionConditionRead;
    type InputBinding = DocumentRetentionConditionReadBinding;
    type Query = DocumentRetentionConditionQuery;
    type ParameterBinding = DocumentQueryParametersBinding;
    type ResultBinding = DocumentRetentionConditionBinding;
    type ScopeBinding = DocumentRetentionQueryScope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;

    const IDENTITY: &'static str =
        "worth.query.certification.document-retention.condition-read-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::bounded(1, 64);

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

impl ApplicationQueryIntent<DocumentRetentionSchema> for DocumentRetentionRead {
    type Binding = DocumentRetentionQueryBinding;

    fn parameters(&self) -> ApplicationQueryParameterSet<DocumentRetentionQuery> {
        ApplicationQueryParameterSet::new()
    }

    fn into_scope(self) -> DocumentRetentionQueryScope {
        DocumentRetentionQueryScope::new(DocumentIdentityField::reference(), self.identity)
    }
}

#[derive(Clone, Debug)]
pub struct DocumentRetentionConditionRead {
    pub identity: String,
}

worth_query_structured_value_binding!(pub DocumentRetentionConditionReadBinding for DocumentRetentionConditionRead {
    identity: "worth.query.certification.document-retention.condition-read.v1"
});

impl ApplicationQueryIntent<DocumentRetentionSchema> for DocumentRetentionConditionRead {
    type Binding = DocumentRetentionConditionQueryBinding;

    fn parameters(&self) -> ApplicationQueryParameterSet<DocumentRetentionConditionQuery> {
        ApplicationQueryParameterSet::new()
    }

    fn into_scope(self) -> DocumentRetentionQueryScope {
        DocumentRetentionQueryScope::new(DocumentIdentityField::reference(), self.identity)
    }
}

/// The request to give one document an exact retention period.
#[derive(Clone, Debug)]
pub struct SetRetentionIntent {
    pub input: SetRetentionInput,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentRetentionWritten {
    pub retention_days: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SetRetentionDenial {
    RetentionUnchanged,
}

worth_query_structured_value_binding!(pub DocumentRetentionWrittenBinding for DocumentRetentionWritten {
    identity: "worth.query.certification.document-retention.written.v1"
});
worth_query_structured_value_binding!(pub SetRetentionDenialBinding for SetRetentionDenial {
    identity: "worth.query.certification.document-retention.write-denial.v1"
});

pub struct SetRetentionBinding;

type SetRetentionScope = ApplicationMutationFieldScope<
    DocumentRetentionSchema,
    Document,
    DocumentFacts,
    DocumentIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;

impl ApplicationMutationBinding<DocumentRetentionSchema> for SetRetentionBinding {
    type Input = SetRetentionInput;
    type InputBinding = SetRetentionInputBinding;
    type Result = DocumentRetentionWritten;
    type ResultBinding = DocumentRetentionWrittenBinding;
    type IdempotencyKey = u64;
    type Operation = SetRetention;
    type Decision = WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>;
    type Denial = SetRetentionDenial;
    type DenialBinding = SetRetentionDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = SetRetentionScope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;

    const IDENTITY: &'static str = "worth.query.certification.document-retention.set-binding.v1";
    const HANDLER_IDENTITY: &'static str =
        "worth.query.certification.document-retention.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.certification.document-retention.command.v1";
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 2, 0),
            ApplicationCandidateResourceCeiling::bounded(1024, 1024),
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

impl ApplicationMutationIntent<DocumentRetentionSchema> for SetRetentionIntent {
    type Binding = SetRetentionBinding;

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

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema> {
    schema
        .application_query_binding::<DocumentRetentionQueryBinding>()
        .application_query_binding::<DocumentRetentionConditionQueryBinding>()
        .application_mutation_binding::<SetRetentionBinding>()
        .application_mutation_binding::<ReviewedSetRetentionBinding>()
}

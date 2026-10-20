//! The public request surface a caller uses to set and read a document's retention.
//!
//! One mutation binding writes the retention and one query binding reports it.
//! Both are declared on the schema, so every court step goes through the same
//! ordinary application entry an external consumer would use.

use worth_query_decl::facade::application_operation::{
    ApplicationCandidatePolicy, ApplicationCandidateRequirements,
};
use worth_query_host::facade::{
    declaration::{
        application_operation::{
            ApplicationCandidateCardinalityCeiling, ApplicationCandidateResourceCeiling,
            ApplicationMutationBinding, ApplicationMutationFieldScope, ApplicationMutationIntent,
            NoApplicationMutationOutputs, NoApplicationMutationSource,
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
pub use candidate_tracking::{
    assessment_counts, candidate_count, decision_count, ordinary_counts, reset_candidate_count,
    reset_decision_count, reset_ordinary_counts,
};
pub(super) use candidate_tracking::{
    record_assessment_candidate, record_assessment_decision, record_ordinary_candidate,
    record_ordinary_decision,
};
#[path = "retention_entry/occurrence_write.rs"]
mod occurrence_write;
#[path = "retention_entry/reviewed_write.rs"]
mod reviewed_write;
pub use occurrence_write::OccurrenceSetRetentionBinding;
#[path = "retention_entry/required_write.rs"]
mod required_write;
pub use required_write::{ExplicitReviewedSetRetentionBinding, ExplicitReviewedSetRetentionIntent};
pub use reviewed_write::{
    ReviewedSetRetentionBinding, ReviewedSetRetentionHandler, ReviewedSetRetentionIntent,
};
#[path = "retention_entry/set_retention_handler.rs"]
mod set_retention_handler;
pub use set_retention_handler::SetRetentionHandler;

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
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);

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

impl
    worth_query_decl::facade::application_operation::ApplicationMutationAdmissionBinding<
        DocumentRetentionSchema,
    > for SetRetentionBinding
{
    type Admission =
        worth_query_decl::facade::application_operation::ApplicationConventionalAdmission;
}

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
    fn workflow_execution_requirements(
    ) -> worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements
    {
        worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements::not_applicable()
    }

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
    const CANDIDATES: ApplicationCandidatePolicy =
        ApplicationCandidatePolicy::finite(ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 2, 0),
            ApplicationCandidateResourceCeiling::representation_bytes(1024),
        ));

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

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema> {
    schema
        .application_query_binding::<DocumentRetentionQueryBinding>()
        .application_query_binding::<DocumentRetentionConditionQueryBinding>()
        .application_mutation_binding::<SetRetentionBinding>()
        .application_mutation_binding::<ReviewedSetRetentionBinding>()
        .application_mutation_binding::<OccurrenceSetRetentionBinding>()
        .application_mutation_binding::<ExplicitReviewedSetRetentionBinding>()
}

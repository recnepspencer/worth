//! Real output-demand producer used by workflow assessment certification.

use worth_query_host::facade::{
    application_contribution::{
        WorthQueryApplicationOutputDemand, WorthQueryApplicationProducerBinding,
        WorthQueryApplicationProducerProvider, WorthQueryProducerApplicability,
        WorthQueryProducerDemandResources, WorthQueryProducerInvariantRequirement,
        WorthQueryProducerLifecyclePosture, WorthQueryProducerOutputFamily,
        WorthQueryWorkflowAssessmentOutputFamily, WorthQueryWorkflowAssessmentPosture,
    },
    declaration::{
        application_operation::{
            ApplicationCandidateCardinalityCeiling, ApplicationCandidateRequirements,
            ApplicationCandidateResourceCeiling, ApplicationMutationBinding,
            ApplicationMutationFieldScope, ApplicationMutationIntent,
            ApplicationMutationOutputContract, ApplicationMutationOutputPosture,
            ApplicationMutationOutputRoleDescriptor, ApplicationMutationOutputRoleFamilyDescriptor,
            ApplicationQueryMutationSource,
        },
        application_schema::{
            ApplicationFieldRef, ApplicationPrincipalBindingRef,
            ApplicationSchemaDeclarationBuilder, EqualityPredicate, NoApplicationUnit, ReadOnly,
            U64ApplicationValueBinding,
        },
    },
    primary_graph::{
        CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
        WorthQueryApplicationOutputRole, WorthQueryInvariantMutationTarget,
    },
    worth_query_operation, worth_query_operation_reads, worth_query_structured_value_binding,
};

use super::{
    retention_entry::{DocumentRetentionQueryBinding, DocumentRetentionRead},
    schema::{
        Document, DocumentFacts, DocumentIdentityField, DocumentPrincipalBinding,
        DocumentRetentionField, DocumentRetentionQuery, DocumentRetentionRow,
        DocumentRetentionSchema, ExternalMapping, Principal,
    },
};

const INITIAL: WorthQueryProducerApplicability = WorthQueryProducerApplicability::new(
    "retention-assessment",
    WorthQueryProducerLifecyclePosture::Initial,
);
const PRESERVE: WorthQueryProducerApplicability = WorthQueryProducerApplicability::new(
    "retention-assessment",
    WorthQueryProducerLifecyclePosture::Preserve,
);
const APPLICABILITY: &[WorthQueryProducerApplicability] = &[INITIAL, PRESERVE];

#[path = "assessment_output/lookalike.rs"]
mod lookalike;
pub use lookalike::LookalikeRetentionAssessmentDemand;

#[derive(Clone, Debug)]
pub struct RetentionAssessmentInput {
    identity: String,
    retention_days: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetentionAssessmentPublished {
    retention_days: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RetentionAssessmentDenial {
    SourceChanged,
}

worth_query_structured_value_binding!(pub RetentionAssessmentInputBinding for RetentionAssessmentInput {
    identity: "worth.query.certification.retention-assessment.input.v1"
});
worth_query_structured_value_binding!(pub RetentionAssessmentPublishedBinding for RetentionAssessmentPublished {
    identity: "worth.query.certification.retention-assessment.published.v1"
});
worth_query_structured_value_binding!(pub RetentionAssessmentDenialBinding for RetentionAssessmentDenial {
    identity: "worth.query.certification.retention-assessment.denial.v1"
});
worth_query_operation!(pub PublishRetentionAssessment for DocumentRetentionSchema, input RetentionAssessmentInputBinding);
worth_query_operation_reads!(PublishRetentionAssessment => [Document, DocumentIdentityField, DocumentRetentionField]);

pub struct RetentionAssessmentOutputs;

impl ApplicationMutationOutputContract<DocumentRetentionSchema> for RetentionAssessmentOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[ApplicationMutationOutputRoleDescriptor::for_entity::<
            DocumentRetentionSchema,
            Document,
        >(
            "assessment", ApplicationMutationOutputPosture::Preserve
        )];
    const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] = &[];
}

pub struct RetentionAssessmentBinding;
type RetentionAssessmentScope = ApplicationMutationFieldScope<
    DocumentRetentionSchema,
    Document,
    DocumentFacts,
    DocumentIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;

impl ApplicationMutationBinding<DocumentRetentionSchema> for RetentionAssessmentBinding {
    type Input = RetentionAssessmentInput;
    type InputBinding = RetentionAssessmentInputBinding;
    type Result = RetentionAssessmentPublished;
    type ResultBinding = RetentionAssessmentPublishedBinding;
    type IdempotencyKey = u64;
    type Operation = PublishRetentionAssessment;
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

    const IDENTITY: &'static str = "worth.query.certification.retention-assessment.binding.v1";
    const HANDLER_IDENTITY: &'static str =
        "worth.query.certification.retention-assessment.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.certification.retention-assessment.command.v1";
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 0, 0),
            ApplicationCandidateResourceCeiling::bounded(512, 256),
        );

    fn idempotency_key_identity(key: &u64) -> [u8; 32] {
        hash(&key.to_le_bytes())
    }

    fn input_identity(input: &RetentionAssessmentInput) -> [u8; 32] {
        let mut bytes = input.identity.as_bytes().to_vec();
        bytes.extend_from_slice(&input.retention_days.to_le_bytes());
        hash(&bytes)
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

impl ApplicationMutationIntent<DocumentRetentionSchema> for RetentionAssessmentInput {
    type Binding = RetentionAssessmentBinding;

    fn input(&self) -> &Self {
        self
    }

    fn scope_binding(&self) -> RetentionAssessmentScope {
        RetentionAssessmentScope::new(DocumentIdentityField::reference(), self.identity.clone())
    }
}

pub struct RetentionAssessmentHandler;

impl OperationHandler<DocumentRetentionSchema, RetentionAssessmentBinding>
    for RetentionAssessmentHandler
{
    fn decide(
        &self,
        input: &RetentionAssessmentInput,
        reader: &mut DecisionReader<
            '_,
            '_,
            '_,
            DocumentRetentionSchema,
            RetentionAssessmentBinding,
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
        RetentionAssessmentBinding::CANDIDATES
    }

    fn build_candidate(
        &self,
        input: &RetentionAssessmentInput,
        target: WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>,
        writer: &mut CandidateWriter<'_, DocumentRetentionSchema, RetentionAssessmentBinding>,
    ) -> HandlerResult<RetentionAssessmentPublished, RetentionAssessmentDenial> {
        let document = match writer.projected_entity(&target) {
            Ok(document) => document,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        if let Err(error) = writer.preserve_output(
            WorthQueryApplicationOutputRole::from_static("assessment"),
            &document,
        ) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
        }
        HandlerResult::Completed(RetentionAssessmentPublished {
            retention_days: input.retention_days,
        })
    }
}

pub struct RetentionAssessmentOutputFamily;

impl WorthQueryProducerOutputFamily<DocumentRetentionSchema> for RetentionAssessmentOutputFamily {
    type Source = DocumentRetentionQueryBinding;
    const IDENTITY: &'static str = "worth.query.certification.retention-assessment.output.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = APPLICABILITY;

    fn profile_kind(_: &DocumentRetentionRow) -> &'static str {
        "retention-assessment"
    }
}

impl WorthQueryWorkflowAssessmentOutputFamily<DocumentRetentionSchema>
    for RetentionAssessmentOutputFamily
{
    fn assessment_posture(row: &DocumentRetentionRow) -> WorthQueryWorkflowAssessmentPosture {
        if row.retention_days == 6 {
            WorthQueryWorkflowAssessmentPosture::Failing
        } else {
            WorthQueryWorkflowAssessmentPosture::Passing
        }
    }
}

#[derive(Clone)]
pub struct RetentionAssessmentDemand {
    identity: String,
}

impl RetentionAssessmentDemand {
    pub fn new(identity: impl Into<String>) -> Self {
        Self {
            identity: identity.into(),
        }
    }
}

impl WorthQueryApplicationOutputDemand<DocumentRetentionSchema> for RetentionAssessmentDemand {
    type OutputFamily = RetentionAssessmentOutputFamily;

    fn source_intent(&self) -> DocumentRetentionRead {
        DocumentRetentionRead {
            identity: self.identity.clone(),
        }
    }
}

pub struct RetentionAssessmentProducer;
pub struct RetentionAssessmentProvider;

impl WorthQueryApplicationProducerBinding<DocumentRetentionSchema> for RetentionAssessmentProducer {
    type Operation = RetentionAssessmentBinding;
    type OutputFamily = RetentionAssessmentOutputFamily;
    type Provider = RetentionAssessmentProvider;
    const IDENTITY: &'static str = "worth.query.certification.retention-assessment.producer.v1";
    const OUTPUT_ROLE: &'static str = "assessment";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = APPLICABILITY;
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const RESOURCE_POLICY: &'static str = "bounded-synchronous";
    const REUSE_POLICY: &'static str = "exact-source";
}

impl WorthQueryApplicationProducerProvider<DocumentRetentionSchema, RetentionAssessmentProducer>
    for RetentionAssessmentProvider
{
    const SEMANTIC_IDENTITY: &'static str =
        "worth.query.certification.retention-assessment.provider.v1";

    fn operation_input(&self, source: &DocumentRetentionRow) -> RetentionAssessmentInput {
        RetentionAssessmentInput {
            identity: source.identity.clone(),
            retention_days: source.retention_days,
        }
    }

    fn idempotency_key(&self, _: &DocumentRetentionRow, source_identity: &[u8; 32]) -> u64 {
        source_identity
            .chunks_exact(8)
            .map(|chunk| u64::from_le_bytes(chunk.try_into().unwrap()))
            .fold(0, u64::wrapping_add)
    }

    fn demand_resources(&self, _: &DocumentRetentionRow) -> WorthQueryProducerDemandResources {
        WorthQueryProducerDemandResources::new(256, 512)
    }
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema> {
    schema
        .operation(
            PublishRetentionAssessment::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_decision_fact_budget(PublishRetentionAssessment::reference(), 8)
        .operation_projection_work_budget(PublishRetentionAssessment::reference(), 8)
        .operation_read_field(
            PublishRetentionAssessment::reference(),
            DocumentIdentityField::reference(),
        )
        .operation_read_field(
            PublishRetentionAssessment::reference(),
            DocumentRetentionField::reference(),
        )
        .application_mutation_binding::<RetentionAssessmentBinding>()
}

fn hash(bytes: &[u8]) -> [u8; 32] {
    let mut identity = [0_u8; 32];
    let mut accumulator = 0xcbf2_9ce4_8422_2325_u64;
    for (index, byte) in bytes.iter().enumerate() {
        accumulator = (accumulator ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3);
        identity[index % identity.len()] ^= (accumulator >> ((index % 8) * 8)) as u8;
    }
    identity
}

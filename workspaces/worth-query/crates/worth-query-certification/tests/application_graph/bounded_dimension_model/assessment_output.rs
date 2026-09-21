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
    dimension_entry::{PartDimensionQueryBinding, PartDimensionRead},
    schema::{
        BoundedDimensionSchema, ExternalMapping, Part, PartDimensionField, PartDimensionQuery,
        PartDimensionRow, PartFacts, PartIdentityField, PartPrincipalBinding, Principal,
    },
};

const INITIAL: WorthQueryProducerApplicability = WorthQueryProducerApplicability::new(
    "part-assessment",
    WorthQueryProducerLifecyclePosture::Initial,
);
const PRESERVE: WorthQueryProducerApplicability = WorthQueryProducerApplicability::new(
    "part-assessment",
    WorthQueryProducerLifecyclePosture::Preserve,
);
const APPLICABILITY: &[WorthQueryProducerApplicability] = &[INITIAL, PRESERVE];

#[derive(Clone, Debug)]
pub struct PartAssessmentInput {
    identity: String,
    dimension: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PartAssessmentPublished {
    dimension: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PartAssessmentDenial {
    SourceChanged,
}

worth_query_structured_value_binding!(pub PartAssessmentInputBinding for PartAssessmentInput {
    identity: "worth.query.certification.part-assessment.input.v1"
});
worth_query_structured_value_binding!(pub PartAssessmentPublishedBinding for PartAssessmentPublished {
    identity: "worth.query.certification.part-assessment.published.v1"
});
worth_query_structured_value_binding!(pub PartAssessmentDenialBinding for PartAssessmentDenial {
    identity: "worth.query.certification.part-assessment.denial.v1"
});
worth_query_operation!(pub PublishPartAssessment for BoundedDimensionSchema, input PartAssessmentInputBinding);
worth_query_operation_reads!(PublishPartAssessment => [Part, PartIdentityField, PartDimensionField]);

pub struct PartAssessmentOutputs;

impl ApplicationMutationOutputContract<BoundedDimensionSchema> for PartAssessmentOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[ApplicationMutationOutputRoleDescriptor::for_entity::<
            BoundedDimensionSchema,
            Part,
        >(
            "assessment", ApplicationMutationOutputPosture::Preserve
        )];
    const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] = &[];
}

pub struct PartAssessmentBinding;
type PartAssessmentScope = ApplicationMutationFieldScope<
    BoundedDimensionSchema,
    Part,
    PartFacts,
    PartIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;

impl ApplicationMutationBinding<BoundedDimensionSchema> for PartAssessmentBinding {
    type Input = PartAssessmentInput;
    type InputBinding = PartAssessmentInputBinding;
    type Result = PartAssessmentPublished;
    type ResultBinding = PartAssessmentPublishedBinding;
    type IdempotencyKey = u64;
    type Operation = PublishPartAssessment;
    type Decision = WorthQueryInvariantMutationTarget<BoundedDimensionSchema, Part>;
    type Denial = PartAssessmentDenial;
    type DenialBinding = PartAssessmentDenialBinding;
    type Output = PartAssessmentOutputs;
    type ScopeBinding = PartAssessmentScope;
    type PrincipalBinding = PartPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<PartDimensionQuery>;

    const IDENTITY: &'static str = "worth.query.certification.part-assessment.binding.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.certification.part-assessment.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.certification.part-assessment.command.v1";
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 0, 0),
            ApplicationCandidateResourceCeiling::bounded(512, 256),
        );

    fn idempotency_key_identity(key: &u64) -> [u8; 32] {
        hash(&key.to_le_bytes())
    }

    fn input_identity(input: &PartAssessmentInput) -> [u8; 32] {
        let mut bytes = input.identity.as_bytes().to_vec();
        bytes.extend_from_slice(&input.dimension.to_le_bytes());
        hash(&bytes)
    }

    fn scope_field() -> ApplicationFieldRef<
        BoundedDimensionSchema,
        Part,
        PartFacts,
        PartIdentityField,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        PartIdentityField::reference()
    }

    fn principal_binding() -> ApplicationPrincipalBindingRef<
        BoundedDimensionSchema,
        PartPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        PartPrincipalBinding::reference()
    }
}

impl ApplicationMutationIntent<BoundedDimensionSchema> for PartAssessmentInput {
    type Binding = PartAssessmentBinding;

    fn input(&self) -> &Self {
        self
    }

    fn scope_binding(&self) -> PartAssessmentScope {
        PartAssessmentScope::new(PartIdentityField::reference(), self.identity.clone())
    }
}

pub struct PartAssessmentHandler;

impl OperationHandler<BoundedDimensionSchema, PartAssessmentBinding> for PartAssessmentHandler {
    fn decide(
        &self,
        input: &PartAssessmentInput,
        reader: &mut DecisionReader<'_, '_, '_, BoundedDimensionSchema, PartAssessmentBinding>,
    ) -> HandlerResult<
        WorthQueryInvariantMutationTarget<BoundedDimensionSchema, Part>,
        PartAssessmentDenial,
    > {
        let part =
            match reader.resolve_entity(PartIdentityField::reference(), input.identity.clone()) {
                Ok(part) => part,
                Err(error) => return HandlerResult::ExecutionDenied(error),
            };
        match reader.field(&part, PartDimensionField::reference()) {
            Ok(Some(dimension)) if dimension == input.dimension => {}
            Ok(_) => return HandlerResult::DomainDenied(PartAssessmentDenial::SourceChanged),
            Err(error) => return HandlerResult::ExecutionDenied(error),
        }
        match reader.mutation_target(&part) {
            Ok(target) => HandlerResult::Completed(target),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }

    fn candidate_requirements(
        &self,
        _: &PartAssessmentInput,
        _: &WorthQueryInvariantMutationTarget<BoundedDimensionSchema, Part>,
    ) -> ApplicationCandidateRequirements {
        PartAssessmentBinding::CANDIDATES
    }

    fn build_candidate(
        &self,
        input: &PartAssessmentInput,
        target: WorthQueryInvariantMutationTarget<BoundedDimensionSchema, Part>,
        writer: &mut CandidateWriter<'_, BoundedDimensionSchema, PartAssessmentBinding>,
    ) -> HandlerResult<PartAssessmentPublished, PartAssessmentDenial> {
        let part = match writer.projected_entity(&target) {
            Ok(part) => part,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        if let Err(error) = writer.preserve_output(
            WorthQueryApplicationOutputRole::from_static("assessment"),
            &part,
        ) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
        }
        HandlerResult::Completed(PartAssessmentPublished {
            dimension: input.dimension,
        })
    }
}

pub struct PartAssessmentOutputFamily;

impl WorthQueryProducerOutputFamily<BoundedDimensionSchema> for PartAssessmentOutputFamily {
    type Source = PartDimensionQueryBinding;
    const IDENTITY: &'static str = "worth.query.certification.part-assessment.output.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = APPLICABILITY;

    fn profile_kind(_: &PartDimensionRow) -> &'static str {
        "part-assessment"
    }
}

impl WorthQueryWorkflowAssessmentOutputFamily<BoundedDimensionSchema>
    for PartAssessmentOutputFamily
{
    fn assessment_posture(row: &PartDimensionRow) -> WorthQueryWorkflowAssessmentPosture {
        if row.dimension == 6 {
            WorthQueryWorkflowAssessmentPosture::Failing
        } else {
            WorthQueryWorkflowAssessmentPosture::Passing
        }
    }
}

#[derive(Clone)]
pub struct PartAssessmentDemand {
    identity: String,
}

impl PartAssessmentDemand {
    pub fn new(identity: impl Into<String>) -> Self {
        Self {
            identity: identity.into(),
        }
    }
}

impl WorthQueryApplicationOutputDemand<BoundedDimensionSchema> for PartAssessmentDemand {
    type OutputFamily = PartAssessmentOutputFamily;

    fn source_intent(&self) -> PartDimensionRead {
        PartDimensionRead {
            identity: self.identity.clone(),
        }
    }
}

pub struct LookalikePartAssessmentOutputFamily;

impl WorthQueryProducerOutputFamily<BoundedDimensionSchema>
    for LookalikePartAssessmentOutputFamily
{
    type Source = PartDimensionQueryBinding;
    const IDENTITY: &'static str = PartAssessmentOutputFamily::IDENTITY;
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = APPLICABILITY;

    fn profile_kind(_: &PartDimensionRow) -> &'static str {
        "part-assessment"
    }
}

impl WorthQueryWorkflowAssessmentOutputFamily<BoundedDimensionSchema>
    for LookalikePartAssessmentOutputFamily
{
    fn assessment_posture(_: &PartDimensionRow) -> WorthQueryWorkflowAssessmentPosture {
        WorthQueryWorkflowAssessmentPosture::Failing
    }
}

pub struct LookalikePartAssessmentDemand {
    identity: String,
}

impl LookalikePartAssessmentDemand {
    pub fn new(identity: impl Into<String>) -> Self {
        Self {
            identity: identity.into(),
        }
    }
}

impl WorthQueryApplicationOutputDemand<BoundedDimensionSchema> for LookalikePartAssessmentDemand {
    type OutputFamily = LookalikePartAssessmentOutputFamily;

    fn source_intent(&self) -> PartDimensionRead {
        PartDimensionRead {
            identity: self.identity.clone(),
        }
    }
}

pub struct PartAssessmentProducer;
pub struct PartAssessmentProvider;

impl WorthQueryApplicationProducerBinding<BoundedDimensionSchema> for PartAssessmentProducer {
    type Operation = PartAssessmentBinding;
    type OutputFamily = PartAssessmentOutputFamily;
    type Provider = PartAssessmentProvider;
    const IDENTITY: &'static str = "worth.query.certification.part-assessment.producer.v1";
    const OUTPUT_ROLE: &'static str = "assessment";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = APPLICABILITY;
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const RESOURCE_POLICY: &'static str = "bounded-synchronous";
    const REUSE_POLICY: &'static str = "exact-source";
}

impl WorthQueryApplicationProducerProvider<BoundedDimensionSchema, PartAssessmentProducer>
    for PartAssessmentProvider
{
    const SEMANTIC_IDENTITY: &'static str = "worth.query.certification.part-assessment.provider.v1";

    fn operation_input(&self, source: &PartDimensionRow) -> PartAssessmentInput {
        PartAssessmentInput {
            identity: source.identity.clone(),
            dimension: source.dimension,
        }
    }

    fn idempotency_key(&self, _: &PartDimensionRow, source_identity: &[u8; 32]) -> u64 {
        source_identity
            .chunks_exact(8)
            .map(|chunk| u64::from_le_bytes(chunk.try_into().unwrap()))
            .fold(0, u64::wrapping_add)
    }

    fn demand_resources(&self, _: &PartDimensionRow) -> WorthQueryProducerDemandResources {
        WorthQueryProducerDemandResources::new(256, 512)
    }
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<BoundedDimensionSchema>,
) -> ApplicationSchemaDeclarationBuilder<BoundedDimensionSchema> {
    schema
        .operation(
            PublishPartAssessment::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_decision_fact_budget(PublishPartAssessment::reference(), 8)
        .operation_projection_work_budget(PublishPartAssessment::reference(), 8)
        .operation_read_field(
            PublishPartAssessment::reference(),
            PartIdentityField::reference(),
        )
        .operation_read_field(
            PublishPartAssessment::reference(),
            PartDimensionField::reference(),
        )
        .application_mutation_binding::<PartAssessmentBinding>()
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

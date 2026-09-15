use std::marker::PhantomData;

use worth_query_consumer_values::{PlanarAdjustmentResult, PlanarMutationDenial};
use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_operation,
    worth_query_operation_reads, worth_query_operation_writes,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::application_contribution;
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryApplicationProducerBinding, WorthQueryApplicationProducerProvider,
        WorthQueryProducerApplicability, WorthQueryProducerDemandResources,
        WorthQueryProducerInvariantRequirement, WorthQueryProducerLifecyclePosture,
    },
    domain,
    primary_graph::{
        CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
        WorthQueryApplicationOutputRole,
    },
};

use super::*;

pub(crate) const ALTERNATE_OUTPUT_APPLICABILITY: WorthQueryProducerApplicability =
    WorthQueryProducerApplicability::new(
        "manual-certification",
        WorthQueryProducerLifecyclePosture::Initial,
    );

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AlternatePlanarOutput {
    pub scope_key: String,
    pub output_key: String,
}

worth_query_structured_value_binding!(pub AlternatePlanarOutputInputBinding for AlternatePlanarOutput {
    identity: "worth.query.certification.alternate-planar-output-input.v1"
});
worth_query_structured_value_binding!(pub AlternatePlanarOutputResultBinding for PlanarAdjustmentResult {
    identity: "worth.query.certification.alternate-planar-output-result.v1"
});
worth_query_structured_value_binding!(pub AlternatePlanarOutputDenialBinding for PlanarMutationDenial {
    identity: "worth.query.certification.alternate-planar-output-denial.v1"
});
worth_query_operation!(pub PublishAlternatePlanarOutput for Schema: TopologySchemaBinding, input AlternatePlanarOutputInputBinding);
worth_query_operation_reads!(PublishAlternatePlanarOutput => [Body, BodyKey, PositionY]);
worth_query_operation_writes!(PublishAlternatePlanarOutput => [PositionY]);

pub struct AlternatePlanarOutputBinding<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> ApplicationMutationBinding<Schema>
    for AlternatePlanarOutputBinding<Schema>
{
    type Input = AlternatePlanarOutput;
    type InputBinding = AlternatePlanarOutputInputBinding;
    type Result = PlanarAdjustmentResult;
    type ResultBinding = AlternatePlanarOutputResultBinding;
    type IdempotencyKey = u64;
    type Operation = PublishAlternatePlanarOutput;
    type Decision = worth_query_consumer_values::PositiveLength;
    type Denial = PlanarMutationDenial;
    type DenialBinding = AlternatePlanarOutputDenialBinding;
    type Output = PlanarOutputs;
    type ScopeBinding = PlanarMutationScope<Schema>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<PlanarQuery>;

    const IDENTITY: &'static str = "worth.query.certification.alternate-planar-output.v1";
    const HANDLER_IDENTITY: &'static str =
        "worth.query.certification.alternate-planar-output-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.certification.alternate-planar-output-command.v1";
    const CANDIDATES: ApplicationCandidateRequirements = requirements(0, 0, 0, 1, 1024, 4096);

    fn idempotency_key_identity(key: &u64) -> [u8; 32] {
        super::mutation_identity::key_identity(*key)
    }

    fn input_identity(input: &AlternatePlanarOutput) -> [u8; 32] {
        super::mutation_identity::input_identity(&PlanarMutation {
            scope_key: input.scope_key.clone(),
            operation: worth_query_consumer_values::PlanarOperation::VerifyCurrentOutputs(vec![
                worth_query_consumer_values::PlanarCurrentOutputExpectation {
                    producer_key: input.scope_key.clone(),
                    output_key: input.output_key.clone(),
                },
            ]),
            validator_work: 0,
        })
    }

    fn scope_field() -> ApplicationFieldRef<
        Schema,
        Body,
        PlanarPosition,
        BodyKey,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        BodyKey::reference()
    }

    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        ConsumerPrincipalBinding,
        ExternalPrincipalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ConsumerPrincipalBinding::reference()
    }
}

impl<Schema: TopologySchemaBinding> ApplicationMutationIntent<Schema> for AlternatePlanarOutput {
    type Binding = AlternatePlanarOutputBinding<Schema>;

    fn scope_binding(&self) -> PlanarMutationScope<Schema> {
        PlanarMutationScope::new(BodyKey::reference(), self.scope_key.clone())
    }
}

pub struct AlternatePlanarOutputHandler;

impl<Schema: TopologySchemaBinding> OperationHandler<Schema, AlternatePlanarOutputBinding<Schema>>
    for AlternatePlanarOutputHandler
{
    fn decide(
        &self,
        input: &AlternatePlanarOutput,
        reader: &mut DecisionReader<'_, '_, '_, Schema, AlternatePlanarOutputBinding<Schema>>,
    ) -> HandlerResult<worth_query_consumer_values::PositiveLength, PlanarMutationDenial> {
        let anchor = match reader.resolve_entity(BodyKey::reference(), input.output_key.clone()) {
            Ok(anchor) => anchor,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        match reader.field(&anchor, PositionY::reference()) {
            Ok(Some(y)) => HandlerResult::Completed(y),
            Ok(None) => HandlerResult::DomainDenied(PlanarMutationDenial::MissingCoordinate),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }

    fn candidate_requirements(
        &self,
        _: &AlternatePlanarOutput,
        _: &worth_query_consumer_values::PositiveLength,
    ) -> ApplicationCandidateRequirements {
        AlternatePlanarOutputBinding::<Schema>::CANDIDATES
    }

    fn build_candidate(
        &self,
        input: &AlternatePlanarOutput,
        y: worth_query_consumer_values::PositiveLength,
        writer: &mut CandidateWriter<'_, Schema, AlternatePlanarOutputBinding<Schema>>,
    ) -> HandlerResult<PlanarAdjustmentResult, PlanarMutationDenial> {
        let anchor = match writer.resolve_entity(BodyKey::reference(), input.output_key.clone()) {
            Ok(anchor) => anchor,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        if let Err(error) = writer.write_field(&anchor, PositionY::reference(), y) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
        }
        match writer.preserve_output(
            WorthQueryApplicationOutputRole::from_static("anchor"),
            &anchor,
        ) {
            Ok(()) => HandlerResult::Completed(PlanarAdjustmentResult {
                changed_vertices: 0,
            }),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}

pub struct AlternatePlanarOutputProvider;

impl<Schema: TopologySchemaBinding>
    WorthQueryApplicationProducerProvider<Schema, AlternatePlanarOutputProducer<Schema>>
    for AlternatePlanarOutputProvider
{
    const SEMANTIC_IDENTITY: &'static str =
        "worth.query.certification.alternate-planar-output-provider.v1";

    fn operation_input(&self, source: &PlanarReadResult) -> AlternatePlanarOutput {
        AlternatePlanarOutput {
            scope_key: source.body_key.clone(),
            output_key: source.body_key.clone(),
        }
    }

    fn idempotency_key(&self, _: &PlanarReadResult, source_identity: &[u8; 32]) -> u64 {
        super::planar_source_key(source_identity) ^ u64::MAX
    }

    fn demand_resources(&self, _: &PlanarReadResult) -> WorthQueryProducerDemandResources {
        WorthQueryProducerDemandResources::new(1024, 1024)
    }
}

pub struct AlternatePlanarOutputProducer<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> WorthQueryApplicationProducerBinding<Schema>
    for AlternatePlanarOutputProducer<Schema>
{
    type Operation = AlternatePlanarOutputBinding<Schema>;
    type OutputFamily = PlanarOutputFamily;
    type Provider = AlternatePlanarOutputProvider;

    const IDENTITY: &'static str = "worth.query.certification.alternate-planar-producer.v1";
    const OUTPUT_ROLE: &'static str = "anchor";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] =
        &[ALTERNATE_OUTPUT_APPLICABILITY];
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const RESOURCE_POLICY: &'static str = "manual-certification-only";
    const REUSE_POLICY: &'static str = "exact-source";
}

pub(crate) fn declare_alternate_output<Schema: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let operation = PublishAlternatePlanarOutput::reference::<Schema>();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_decision_fact_budget(operation, 16)
        .operation_projection_work_budget(operation, 8)
        .operation_read_entity(operation, Body::reference())
        .operation_read_field(operation, BodyKey::reference())
        .operation_read_field(operation, PositionY::reference())
        .operation_write(operation, PositionY::reference())
        .application_mutation_binding::<AlternatePlanarOutputBinding<Schema>>()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AlternateReadinessDomain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AlternateReadinessOperation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AlternateReadinessFamily;

worth_query_host::facade::worth_query_conditional_node!(
    pub AlternatePlanarReadyNode in AlternateReadinessDomain, AlternateReadinessOperation,
    AlternateReadinessFamily => operation "alternate-planar-output-ready"
);

pub struct AlternatePlanarReadiness<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> AlternatePlanarReadiness<Schema> {
    fn conditional_binding() -> domain::WorthQueryApplicationConditionalOperationBinding<
        Schema,
        PublishAlternatePlanarOutput,
        AlternatePlanarOutput,
        AlternateReadinessDomain,
        AlternateReadinessOperation,
        AlternateReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            PublishAlternatePlanarOutput::reference::<Schema>(),
            alternate_readiness_definition().reference(),
        )
    }
}

impl<Schema: TopologySchemaBinding>
    application_contribution::WorthQueryApplicationConditionalBinding<Schema>
    for AlternatePlanarReadiness<Schema>
{
    type Configuration = ();
    type Installed = ();

    const IDENTITY: &'static str = "worth.query.certification.alternate-planar-output-readiness.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] =
        &["worth.query.certification.alternate-planar-producer.v1"];

    fn package_contract(
    ) -> application_contribution::WorthQueryApplicationConditionalPackageContract {
        application_contribution::WorthQueryApplicationConditionalPackageContract::new(
            alternate_readiness_definition().into_portable(),
            Self::conditional_binding().portable().clone(),
            AlternatePlanarReadyNode::reference().node_identity(),
        )
    }

    fn install(
        _: (),
        _: &application_contribution::WorthQueryApplicationConditionalProducerAccess<'_, Schema>,
        installation: &mut worth_query_host::facade::primary_graph::WorthQueryConditionalApplicationRuntimeInstallation<Schema>,
    ) -> Result<
        (),
        worth_query_host::facade::primary_graph::WorthQueryConditionalRuntimeInstallationDenial,
    > {
        let operation = installation
            .installed_schema()
            .installed_operation(PublishAlternatePlanarOutput::reference::<Schema>())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &Self::conditional_binding())
            .unwrap()
            .bind_node(AlternatePlanarReadyNode::reference())
            .unwrap();
        installation
            .bind_output_readiness::<AlternatePlanarOutputProducer<Schema>, _, _, _, _, _, _>(
                node, 0,
            )
    }
}

fn alternate_readiness_definition() -> domain::WorthQueryDomainOperationDefinition<
    AlternateReadinessDomain,
    AlternateReadinessOperation,
    AlternateReadinessFamily,
> {
    let dependency = super::readiness::output_change_dependency();
    application_contribution::WorthQueryOutputReadinessContractBuilder::new(
        domain::WorthQueryDomainOperationIdentity::new("alternate-planar-output-readiness", 1),
        "alternate-planar-output-ready",
        super::readiness::output_change_projection(),
        super::readiness::canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("anchor").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("alternate-planar-readiness").unwrap(),
        128,
        128,
        "alternate-planar-readiness-v1",
    )
    .semantic_reads([super::readiness::output_change_projection()])
    .dependencies([dependency.clone()])
    .readiness_dependencies([dependency])
    .build()
    .expect("alternate planar readiness declaration is canonical")
}

use std::marker::PhantomData;
use std::sync::{atomic::AtomicUsize, Arc};

use worth_query_decl::facade::application_schema::{
    ApplicationInvariantExecutionPoint, ApplicationSchema, ApplicationSchemaComposition,
    ApplicationSchemaContribution, ApplicationSchemaContributionAuthoring,
    ApplicationSchemaContributionIdentity, ApplicationSchemaDeclaration,
    ApplicationSchemaDeclarationBuilder, ApplicationSchemaDeclarationDenial,
};
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryApplicationContribution, WorthQueryApplicationContributionContracts,
        WorthQueryApplicationContributionSetup, WorthQueryApplicationProducerBinding,
        WorthQueryApplicationProducerProvider, WorthQueryProducerApplicability,
        WorthQueryProducerDemandResources, WorthQueryProducerInvariantRequirement,
        WorthQueryProducerLifecyclePosture, WorthQueryProducerOutputFamily,
    },
    application_installation,
    primary_graph::{
        WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
    },
};
use worth_query_parameter_entry::{ParameterContribution, ParameterSchemaBinding};
use worth_query_topology_entry::{
    Body, PlanarAnchorOutput, PlanarMutation, PlanarMutationBinding, PlanarReadBinding,
    PlanarReadResult, TopologyConfiguration, TopologyContribution, TopologySchemaBinding,
    VertexReplacement, VertexReplacementAnchorOutput, VertexReplacementBinding,
};

use super::{assert_contribution_denial, limits, validated_denial_program};

const INITIAL: WorthQueryProducerApplicability =
    WorthQueryProducerApplicability::new("planar", WorthQueryProducerLifecyclePosture::Initial);

pub(super) fn run() {
    missing_required_invariant_is_denied_before_initial_state();
    foreign_source_selector_is_denied_before_initial_state();
}

struct ContractProvider;

struct MissingInvariantFamily;

impl<Schema: TopologySchemaBinding> WorthQueryProducerOutputFamily<Schema>
    for MissingInvariantFamily
{
    type Source = PlanarReadBinding<Schema>;
    type Entity = Body;
    const IDENTITY: &'static str = "worth.query.certification.missing-invariant-output.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = &[INITIAL];
    fn profile_kind(_: &PlanarReadResult) -> &'static str {
        "planar"
    }
}

struct MissingInvariantProducer<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> WorthQueryApplicationProducerBinding<Schema>
    for MissingInvariantProducer<Schema>
{
    type Operation = PlanarMutationBinding<Schema>;
    type OutputFamily = MissingInvariantFamily;
    type Provider = ContractProvider;

    type OutputRole = PlanarAnchorOutput<Schema>;

    const IDENTITY: &'static str = "worth.query.certification.missing-invariant-producer.v1";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = &[INITIAL];
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] =
        &[WorthQueryProducerInvariantRequirement::new(
            "AbsentInvariant",
            1,
            0,
            ApplicationInvariantExecutionPoint::CommitBoundary,
        )];
    const RESOURCE_POLICY: &'static str = "bounded-synchronous";
    const REUSE_POLICY: &'static str = "exact-source";
}

struct ForeignOperationFamily;

impl<Schema: TopologySchemaBinding> WorthQueryProducerOutputFamily<Schema>
    for ForeignOperationFamily
{
    type Source = PlanarReadBinding<Schema>;
    type Entity = Body;
    const IDENTITY: &'static str = "worth.query.certification.foreign-operation-output.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = &[INITIAL];
    fn profile_kind(_: &PlanarReadResult) -> &'static str {
        "planar"
    }
}

struct ForeignOperationProducer<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> WorthQueryApplicationProducerBinding<Schema>
    for ForeignOperationProducer<Schema>
{
    type Operation = VertexReplacementBinding<Schema>;
    type OutputFamily = ForeignOperationFamily;
    type Provider = ContractProvider;

    type OutputRole = VertexReplacementAnchorOutput<Schema>;

    const IDENTITY: &'static str = "worth.query.certification.foreign-operation-producer.v1";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = &[INITIAL];
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const RESOURCE_POLICY: &'static str = "bounded-synchronous";
    const REUSE_POLICY: &'static str = "exact-source";
}

impl<Schema: TopologySchemaBinding>
    WorthQueryApplicationProducerProvider<Schema, MissingInvariantProducer<Schema>>
    for ContractProvider
{
    const SEMANTIC_IDENTITY: &'static str = "worth.query.certification.contract-provider.v1";

    fn operation_input(&self, source: &PlanarReadResult) -> PlanarMutation {
        worth_query_topology_entry::planar_producer_input(source)
    }

    fn idempotency_key(&self, _: &PlanarReadResult, source_identity: &[u8; 32]) -> u64 {
        worth_query_topology_entry::planar_source_key(source_identity)
    }

    fn demand_resources(&self, _: &PlanarReadResult) -> WorthQueryProducerDemandResources {
        worth_query_topology_entry::planar_producer_resources()
    }
}

impl<Schema: TopologySchemaBinding>
    WorthQueryApplicationProducerProvider<Schema, ForeignOperationProducer<Schema>>
    for ContractProvider
{
    const SEMANTIC_IDENTITY: &'static str = "worth.query.certification.contract-provider.v1";

    fn operation_input(&self, source: &PlanarReadResult) -> VertexReplacement {
        use worth_query_consumer_values::{PlanarVertex, PlanarVertexReplacement};
        VertexReplacement {
            scope_key: source.body_key.clone(),
            replacement: PlanarVertexReplacement {
                retired_key: source.body_key.clone(),
                next_key: source.body_key.clone(),
                replacement: PlanarVertex {
                    body_key: format!("{}:replacement", source.body_key),
                    x: source.y,
                    y: source.y,
                },
            },
        }
    }

    fn idempotency_key(&self, _: &PlanarReadResult, source_identity: &[u8; 32]) -> u64 {
        worth_query_topology_entry::planar_source_key(source_identity)
    }

    fn demand_resources(&self, _: &PlanarReadResult) -> WorthQueryProducerDemandResources {
        worth_query_topology_entry::planar_producer_resources()
    }
}

struct MissingInvariantContribution;

impl<Schema: TopologySchemaBinding> ApplicationSchemaContribution<Schema>
    for MissingInvariantContribution
{
    const IDENTITY: ApplicationSchemaContributionIdentity =
        <TopologyContribution as ApplicationSchemaContribution<Schema>>::IDENTITY;

    fn register_members(
        builder: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> ApplicationSchemaDeclarationBuilder<Schema> {
        <TopologyContribution as ApplicationSchemaContribution<Schema>>::register_members(builder)
    }
}

impl<Schema: TopologySchemaBinding> WorthQueryApplicationContribution<Schema>
    for MissingInvariantContribution
{
    type Configuration = ();

    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        contracts.producer::<MissingInvariantProducer<Schema>>()?;
        Ok(())
    }

    fn configure(
        _: (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.producer::<MissingInvariantProducer<Schema>>(ContractProvider)
    }
}

struct MissingInvariantSchema;
impl TopologySchemaBinding for MissingInvariantSchema {}
impl ParameterSchemaBinding for MissingInvariantSchema {}
impl ApplicationSchema for MissingInvariantSchema {
    const OWNER: &'static str = "worth.query.certification.producer-contract-denials";
    const NAME: &'static str = "MissingInvariantSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        ApplicationSchemaContributionAuthoring::contributions(
            ApplicationSchemaDeclarationBuilder::<Self>::for_schema(),
        )
        .register::<MissingInvariantContribution>()?
        .register::<ParameterContribution>()?
        .build()
    }
}
impl ApplicationSchemaComposition for MissingInvariantSchema {
    type Contributions = (MissingInvariantContribution, ParameterContribution);
}

struct ForeignOperationContribution;

impl<Schema: ParameterSchemaBinding> ApplicationSchemaContribution<Schema>
    for ForeignOperationContribution
{
    const IDENTITY: ApplicationSchemaContributionIdentity =
        <ParameterContribution as ApplicationSchemaContribution<Schema>>::IDENTITY;

    fn register_members(
        builder: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> ApplicationSchemaDeclarationBuilder<Schema> {
        <ParameterContribution as ApplicationSchemaContribution<Schema>>::register_members(builder)
    }
}

impl<Schema> WorthQueryApplicationContribution<Schema> for ForeignOperationContribution
where
    Schema: ParameterSchemaBinding + TopologySchemaBinding,
{
    type Configuration = ();

    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        contracts.producer::<ForeignOperationProducer<Schema>>()?;
        Ok(())
    }

    fn configure(
        _: (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.producer::<ForeignOperationProducer<Schema>>(ContractProvider)
    }
}

struct ForeignOperationSchema;
impl TopologySchemaBinding for ForeignOperationSchema {}
impl ParameterSchemaBinding for ForeignOperationSchema {}
impl ApplicationSchema for ForeignOperationSchema {
    const OWNER: &'static str = "worth.query.certification.foreign-operation-denial";
    const NAME: &'static str = "ForeignOperationSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        ApplicationSchemaContributionAuthoring::contributions(
            ApplicationSchemaDeclarationBuilder::<Self>::for_schema(),
        )
        .register::<TopologyContribution>()?
        .register::<ForeignOperationContribution>()?
        .build()
    }
}
impl ApplicationSchemaComposition for ForeignOperationSchema {
    type Contributions = (TopologyContribution, ForeignOperationContribution);
}

fn missing_required_invariant_is_denied_before_initial_state() {
    let result = application_installation::in_memory_program(
        validated_denial_program::<MissingInvariantSchema>(),
        MissingInvariantSchema::declaration().unwrap(),
        ((), Arc::new(AtomicUsize::new(0))),
        limits(),
        |_, _| panic!("missing producer invariant must deny before initial state"),
    );
    assert_contribution_denial(
        result,
        WorthQueryPrimaryGraphInstallationDenialKind::ContributionMemberMismatch,
    );
}

fn foreign_source_selector_is_denied_before_initial_state() {
    let counter = || Arc::new(AtomicUsize::new(0));
    let topology = TopologyConfiguration {
        setup_calls: counter(),
        invariant_calls: counter(),
        invariant_probe: counter(),
        producer_authorization_denials: counter(),
    };
    let result = application_installation::in_memory_program(
        validated_denial_program::<ForeignOperationSchema>(),
        ForeignOperationSchema::declaration().unwrap(),
        (topology, ()),
        limits(),
        |_, _| panic!("foreign producer source must deny before initial state"),
    );
    match result {
        Err(application_installation::WorthQueryInMemoryApplicationDenial::Contributions(
            denial,
        )) => {
            assert_eq!(
                denial.kind(),
                WorthQueryPrimaryGraphInstallationDenialKind::ContributionMemberMismatch
            );
            assert_eq!(
                denial.subject(),
                <PlanarReadBinding<ForeignOperationSchema> as worth_query_decl::facade::application_query::ApplicationQueryBinding<ForeignOperationSchema>>::IDENTITY,
            );
        }
        Err(other) => panic!("expected foreign source contribution denial, got {other:?}"),
        Ok(_) => panic!("foreign producer source published an application"),
    }
}

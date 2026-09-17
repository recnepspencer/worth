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
    PlanarMutation, PlanarMutationBinding, PlanarReadBinding, PlanarReadResult,
    TopologyConfiguration, TopologyContribution, TopologySchemaBinding, VertexReplacement,
    VertexReplacementBinding,
};

use super::{assert_contribution_denial, limits, validated_denial_program};

const INITIAL: WorthQueryProducerApplicability =
    WorthQueryProducerApplicability::new("planar", WorthQueryProducerLifecyclePosture::Initial);

pub(super) fn run() {
    undeclared_output_role_is_denied_before_configuration();
    missing_required_invariant_is_denied_before_initial_state();
    foreign_source_selector_is_denied_before_initial_state();
}

struct ContractProvider;

struct InvalidRoleFamily;

impl<Schema: TopologySchemaBinding> WorthQueryProducerOutputFamily<Schema> for InvalidRoleFamily {
    type Source = PlanarReadBinding<Schema>;
    const IDENTITY: &'static str = "worth.query.certification.invalid-role-output.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = &[INITIAL];
    fn profile_kind(_: &PlanarReadResult) -> &'static str {
        "planar"
    }
}

struct InvalidRoleProducer<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> WorthQueryApplicationProducerBinding<Schema>
    for InvalidRoleProducer<Schema>
{
    type Operation = PlanarMutationBinding<Schema>;
    type OutputFamily = InvalidRoleFamily;
    type Provider = ContractProvider;

    const IDENTITY: &'static str = "worth.query.certification.invalid-role-producer.v1";
    const OUTPUT_ROLE: &'static str = "undeclared";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = &[INITIAL];
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const RESOURCE_POLICY: &'static str = "bounded-synchronous";
    const REUSE_POLICY: &'static str = "exact-source";
}

struct MissingInvariantFamily;

impl<Schema: TopologySchemaBinding> WorthQueryProducerOutputFamily<Schema>
    for MissingInvariantFamily
{
    type Source = PlanarReadBinding<Schema>;
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

    const IDENTITY: &'static str = "worth.query.certification.missing-invariant-producer.v1";
    const OUTPUT_ROLE: &'static str = "anchor";
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

    const IDENTITY: &'static str = "worth.query.certification.foreign-operation-producer.v1";
    const OUTPUT_ROLE: &'static str = "anchor";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = &[INITIAL];
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const RESOURCE_POLICY: &'static str = "bounded-synchronous";
    const REUSE_POLICY: &'static str = "exact-source";
}

macro_rules! planar_contract_provider {
    ($producer:ident) => {
        impl<Schema: TopologySchemaBinding>
            WorthQueryApplicationProducerProvider<Schema, $producer<Schema>> for ContractProvider
        {
            const SEMANTIC_IDENTITY: &'static str =
                "worth.query.certification.contract-provider.v1";

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
    };
}

planar_contract_provider!(InvalidRoleProducer);
planar_contract_provider!(MissingInvariantProducer);

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

macro_rules! topology_wrapper {
    ($Contribution:ident, $Producer:ident) => {
        struct $Contribution;

        impl<Schema: TopologySchemaBinding> ApplicationSchemaContribution<Schema>
            for $Contribution
        {
            const IDENTITY: ApplicationSchemaContributionIdentity =
                <TopologyContribution as ApplicationSchemaContribution<Schema>>::IDENTITY;

            fn register_members(
                builder: ApplicationSchemaDeclarationBuilder<Schema>,
            ) -> ApplicationSchemaDeclarationBuilder<Schema> {
                <TopologyContribution as ApplicationSchemaContribution<Schema>>::register_members(
                    builder,
                )
            }
        }

        impl<Schema: TopologySchemaBinding> WorthQueryApplicationContribution<Schema>
            for $Contribution
        {
            type Configuration = ();

            fn contracts(
                contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
            ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
                contracts.producer::<$Producer<Schema>>()?;
                Ok(())
            }

            fn configure(
                _: (),
                setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
            ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
                setup.producer::<$Producer<Schema>>(ContractProvider)
            }
        }
    };
}

topology_wrapper!(InvalidRoleContribution, InvalidRoleProducer);
topology_wrapper!(MissingInvariantContribution, MissingInvariantProducer);

macro_rules! wrapper_schema {
    ($Schema:ident, $Contribution:ty) => {
        struct $Schema;
        impl TopologySchemaBinding for $Schema {}
        impl ParameterSchemaBinding for $Schema {}
        impl ApplicationSchema for $Schema {
            const OWNER: &'static str = "worth.query.certification.producer-contract-denials";
            const NAME: &'static str = stringify!($Schema);
            const MAJOR: u32 = 1;
            const MINOR: u32 = 0;

            fn declaration(
            ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial>
            {
                ApplicationSchemaContributionAuthoring::contributions(
                    ApplicationSchemaDeclarationBuilder::<Self>::for_schema(),
                )
                .register::<$Contribution>()?
                .register::<ParameterContribution>()?
                .build()
            }
        }
        impl ApplicationSchemaComposition for $Schema {
            type Contributions = ($Contribution, ParameterContribution);
        }
    };
}

wrapper_schema!(InvalidRoleSchema, InvalidRoleContribution);
wrapper_schema!(MissingInvariantSchema, MissingInvariantContribution);

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

fn undeclared_output_role_is_denied_before_configuration() {
    let result = application_installation::in_memory_program(
        validated_denial_program::<InvalidRoleSchema>(),
        InvalidRoleSchema::declaration().unwrap(),
        ((), Arc::new(AtomicUsize::new(0))),
        limits(),
        |_, _| panic!("invalid output role must deny before initial state"),
    );
    assert_contribution_denial(
        result,
        WorthQueryPrimaryGraphInstallationDenialKind::ProducerBindingMeaningMismatch,
    );
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

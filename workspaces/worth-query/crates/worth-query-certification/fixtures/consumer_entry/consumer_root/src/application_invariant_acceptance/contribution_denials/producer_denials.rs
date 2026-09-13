use std::marker::PhantomData;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use worth_query_decl::facade::application_schema::{
    ApplicationInvariantExecutionPoint, ApplicationInvariantMarkerIdentity, ApplicationSchema,
    ApplicationSchemaComposition, ApplicationSchemaContribution,
    ApplicationSchemaContributionAuthoring, ApplicationSchemaContributionIdentity,
    ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
    ApplicationSchemaDeclarationDenial,
};
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryApplicationContribution, WorthQueryApplicationContributionContracts,
        WorthQueryApplicationContributionSetup, WorthQueryApplicationProducerBinding,
        WorthQueryApplicationProducerProvider, WorthQueryProducerApplicability,
        WorthQueryProducerDemandResources, WorthQueryProducerInvariantRequirement,
        WorthQueryProducerLifecyclePosture,
    },
    application_installation,
    primary_graph::{
        WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
    },
};
use worth_query_parameter_entry::{ParameterContribution, ParameterSchemaBinding};
use worth_query_topology_entry::{
    InitialPlanarProducer, InitialPlanarProvider, PlanarHandler, PlanarMutation,
    PlanarMutationBinding, PlanarOutputFamily, PlanarReadResult, PositivePlanarTurn,
    PositiveTurnRule, TopologyContribution, TopologySchemaBinding,
};

use super::{assert_contribution_denial, limits};

pub(super) fn run() {
    missing_provider_precedes_initializer();
    duplicate_provider_is_denied();
    ambiguous_applicability_precedes_callbacks();
}

macro_rules! producer_schema {
    ($Schema:ident, $Contribution:ty) => {
        struct $Schema;
        impl TopologySchemaBinding for $Schema {}
        impl ParameterSchemaBinding for $Schema {}
        impl ApplicationSchema for $Schema {
            const OWNER: &'static str = "worth.query.certification.producer-denials";
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

producer_schema!(MissingProducerSchema, ProducerOmissionContribution);
producer_schema!(DuplicateProducerSchema, DuplicateProducerContribution);
producer_schema!(AmbiguousProducerSchema, AmbiguousProducerContribution);

fn missing_provider_precedes_initializer() {
    let setup_calls = Arc::new(AtomicUsize::new(0));
    let seed_calls = Arc::new(AtomicUsize::new(0));
    let result = application_installation::in_memory::<MissingProducerSchema>(
        MissingProducerSchema::declaration().unwrap(),
        (Arc::clone(&setup_calls), Arc::clone(&setup_calls)),
        limits(),
        |_, _| {
            seed_calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        },
    );
    assert_contribution_denial(
        result,
        WorthQueryPrimaryGraphInstallationDenialKind::MissingProducerProvider,
    );
    assert_eq!(setup_calls.load(Ordering::SeqCst), 2);
    assert_eq!(seed_calls.load(Ordering::SeqCst), 0);
}

fn duplicate_provider_is_denied() {
    let setup_calls = Arc::new(AtomicUsize::new(0));
    let result = application_installation::in_memory::<DuplicateProducerSchema>(
        DuplicateProducerSchema::declaration().unwrap(),
        (Arc::clone(&setup_calls), Arc::clone(&setup_calls)),
        limits(),
        |_, _| panic!("duplicate provider must deny before initial state"),
    );
    assert_contribution_denial(
        result,
        WorthQueryPrimaryGraphInstallationDenialKind::DuplicateProducerBinding,
    );
    assert_eq!(setup_calls.load(Ordering::SeqCst), 1);
}

fn ambiguous_applicability_precedes_callbacks() {
    let calls = Arc::new(AtomicUsize::new(0));
    let result = application_installation::in_memory::<AmbiguousProducerSchema>(
        AmbiguousProducerSchema::declaration().unwrap(),
        (Arc::clone(&calls), Arc::clone(&calls)),
        limits(),
        |_, _| panic!("ambiguous applicability must deny before initial state"),
    );
    assert_contribution_denial(
        result,
        WorthQueryPrimaryGraphInstallationDenialKind::AmbiguousApplicableProducer,
    );
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

struct ProducerOmissionContribution;

impl<Schema: TopologySchemaBinding> ApplicationSchemaContribution<Schema>
    for ProducerOmissionContribution
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
    for ProducerOmissionContribution
{
    type Configuration = Arc<AtomicUsize>;

    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        declare_planar_producers(contracts)?;
        Ok(())
    }

    fn configure(
        calls: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        calls.fetch_add(1, Ordering::SeqCst);
        install_topology_behavior(setup)
    }
}

struct DuplicateProducerContribution;

impl<Schema: TopologySchemaBinding> ApplicationSchemaContribution<Schema>
    for DuplicateProducerContribution
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
    for DuplicateProducerContribution
{
    type Configuration = Arc<AtomicUsize>;

    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        declare_planar_producers(contracts)?;
        Ok(())
    }

    fn configure(
        calls: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        calls.fetch_add(1, Ordering::SeqCst);
        install_topology_behavior(setup)?;
        setup.producer::<InitialPlanarProducer<Schema>>(InitialPlanarProvider)?;
        setup.producer::<InitialPlanarProducer<Schema>>(InitialPlanarProvider)
    }
}

struct AmbiguousProvider;

impl<Schema: TopologySchemaBinding>
    WorthQueryApplicationProducerProvider<Schema, AmbiguousProducer<Schema>> for AmbiguousProvider
{
    const SEMANTIC_IDENTITY: &'static str = "worth.query.certification.ambiguous-provider.v1";

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

struct AmbiguousProducer<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> WorthQueryApplicationProducerBinding<Schema>
    for AmbiguousProducer<Schema>
{
    type Operation = PlanarMutationBinding<Schema>;
    type OutputFamily = PlanarOutputFamily;
    type Provider = AmbiguousProvider;

    const IDENTITY: &'static str = "worth.query.certification.ambiguous-producer.v1";
    const OUTPUT_ROLE: &'static str = "anchor";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] =
        &[WorthQueryProducerApplicability::new(
            "planar",
            WorthQueryProducerLifecyclePosture::Initial,
        )];
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] =
        &[WorthQueryProducerInvariantRequirement::new(
            "PositivePlanarTurn",
            1,
            0,
            ApplicationInvariantExecutionPoint::CommitBoundary,
        )];
    const RESOURCE_POLICY: &'static str = "bounded-synchronous";
    const REUSE_POLICY: &'static str = "exact-source";
}

struct AmbiguousProducerContribution;

impl<Schema: TopologySchemaBinding> ApplicationSchemaContribution<Schema>
    for AmbiguousProducerContribution
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
    for AmbiguousProducerContribution
{
    type Configuration = Arc<AtomicUsize>;

    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        declare_planar_producers(contracts)?.producer::<AmbiguousProducer<Schema>>()?;
        Ok(())
    }

    fn configure(
        calls: Self::Configuration,
        _: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

fn declare_planar_producers<Schema: TopologySchemaBinding>(
    contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
) -> Result<
    &mut WorthQueryApplicationContributionContracts<Schema>,
    WorthQueryPrimaryGraphInstallationDenial,
> {
    contracts.producer::<InitialPlanarProducer<Schema>>()
}

fn install_topology_behavior<Schema: TopologySchemaBinding>(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    setup.invariant(
        PositivePlanarTurn::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        |_| -> Result<PositiveTurnRule<Schema>, String> {
            panic!("producer denial must precede invariant factory execution")
        },
    )?;
    setup.handler::<PlanarMutationBinding<Schema>, _>(PlanarHandler)?;
    setup.handler::<worth_query_topology_entry::VertexReplacementBinding<Schema>, _>(
        worth_query_topology_entry::VertexReplacementHandler,
    )
}

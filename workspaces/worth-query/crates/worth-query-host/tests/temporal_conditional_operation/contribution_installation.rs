use std::sync::Arc;
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;

use worth_query_host::facade::{
    application_contribution::{
        WorthQueryApplicationConditionalBinding, WorthQueryApplicationConditionalPackageContract,
        WorthQueryApplicationConditionalProducerAccess, WorthQueryApplicationContribution,
        WorthQueryApplicationContributionContracts, WorthQueryApplicationContributionSetup,
    },
    application_installation::{self, WorthQueryInMemoryApplicationLimits},
    declaration::{
        application_program::{
            ApplicationArtifactDependency, ApplicationArtifactRetention,
            ApplicationArtifactSuccession, ApplicationDerivedArtifact, ApplicationFeature,
            ApplicationFeatureInputLeaf, ApplicationFeatureSpec, ApplicationLocalityGranule,
            ApplicationLocalityScope, ApplicationNoOutputGraph, ApplicationOutputPort,
            ApplicationProgramAuthoring, ApplicationProgramDefinition, ApplicationProgramIdentity,
            ApplicationProgramOutputs, ApplicationRootComposition, ApplicationRuleLeaf,
        },
        application_query::ApplicationQueryParameterSet,
        application_schema::ApplicationSchemaComposition,
    },
    domain, primary_graph, product, runtime,
};

use super::adapters::{
    block_on, ClockController, ClockSource, ContactCounters, CourtroomClock, IntentProjector,
    Invoker, Predicate, PrincipalSource,
};
use super::contract::{self, TemporalReadyNode};
use super::schema::current_read::TemporalIntentCurrentReadBinding;
use super::schema::*;
use super::world::{
    admit_identity_adapter, request_scope, resources::product_world_resources, seed::seed_graph,
};

struct TemporalInstallationProgram;
struct TemporalInstallationFeature;
struct TemporalArtifactOutput;
struct TemporalArtifactLocality;
struct TemporalArtifact;

impl ApplicationFeature<TemporalHostSchema> for TemporalInstallationFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.host.temporal-installation-feature.v1";
}

impl ApplicationOutputPort<TemporalHostSchema, TemporalInstallationFeature>
    for TemporalArtifactOutput
{
    type Value = IntentQueryResultBinding;
    const IDENTITY: &'static str = "worth.query.host.temporal-artifact-output.v1";
}

impl ApplicationLocalityScope for TemporalArtifactLocality {
    const IDENTITY: &'static str = "worth.query.host.temporal-artifact-locality.v1";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Partition;
}

impl ApplicationDerivedArtifact<TemporalHostSchema, TemporalInstallationFeature>
    for TemporalArtifact
{
    type Output = TemporalArtifactOutput;
    type Locality = TemporalArtifactLocality;
    const IDENTITY: &'static str = "worth.query.host.temporal-artifact.v1";
    const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Disposable;
    const SUCCESSION: ApplicationArtifactSuccession = ApplicationArtifactSuccession::Recompute;
    const REQUIRED: bool = false;
    const PRODUCER_FAMILY: &'static str = "worth.query.host.temporal-producer.v1";
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] =
        &[ApplicationArtifactDependency::new(
            "worth.query.host.temporal-intent.v1",
        )];
    const REUSE_RULE: &'static str = "worth.query.host.temporal-same-basis.v1";

    const STOPPED_OUTCOME: &'static str = "worth.query.host.temporal-artifact-stopped.v1";
}

impl ApplicationProgramDefinition<TemporalHostSchema> for TemporalInstallationProgram {
    type Contributions = <TemporalHostSchema as ApplicationSchemaComposition>::Contributions;
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;

    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.host.temporal-installation.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<TemporalHostSchema, TemporalInstallationFeature>()
                .derived_artifact::<TemporalArtifact>()
                .conditional_operation::<ExecuteTemporal>()
                .operation::<AmendTemporal>()
                .finish(),
        ]
    }
}

fn validated_program(
) -> worth_query_host::facade::declaration::application_program::ValidatedApplicationProgram<
    TemporalHostSchema,
    TemporalInstallationProgram,
> {
    ApplicationProgramAuthoring::<TemporalHostSchema, TemporalInstallationProgram>::begin()
        .validated_program()
        .expect("the temporal installation program has no authored output obligations")
}

pub struct TemporalContributionConfiguration {
    installation_predicate: Predicate,
    definition_predicate: Arc<Predicate>,
    clock_source: ClockSource,
    clock_control: ClockController,
    contacts: ContactCounters,
    install_route: bool,
}

struct InstalledTemporalConditional {
    clock: primary_graph::WorthQueryConditionalClockHandle<
        TemporalHostSchema,
        TemporalReadyNode,
        CourtroomClock,
    >,
    definition_predicate: Arc<Predicate>,
    clock_control: ClockController,
    contacts: ContactCounters,
    invariant:
        Arc<primary_graph::WorthQueryApplicationInvariantProjectionAuthority<TemporalHostSchema>>,
}

struct TemporalConditional;

impl WorthQueryApplicationConditionalBinding<TemporalHostSchema> for TemporalConditional {
    type Configuration = TemporalContributionConfiguration;
    type Installed = Option<InstalledTemporalConditional>;
    type Operation = ExecuteTemporal;

    const IDENTITY: &'static str = "worth.query.host.courtroom.temporal-conditional.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] = &[];

    fn package_contract(
        _declaration: &worth_query_host::facade::declaration::application_schema::ErasedApplicationSchemaDeclaration,
    ) -> Result<
        WorthQueryApplicationConditionalPackageContract,
        worth_query_host::facade::application_contribution::WorthQueryOutputReadinessContractDenial,
    > {
        Ok({
            WorthQueryApplicationConditionalPackageContract::new(
                contract::operation_definition().into_portable(),
                contract::conditional_binding().portable().clone(),
                TemporalReadyNode::reference().node_identity(),
            )
        })
    }

    fn install(
        configuration: Self::Configuration,
        _: &WorthQueryApplicationConditionalProducerAccess<'_, TemporalHostSchema>,
        installation: &mut primary_graph::WorthQueryConditionalApplicationRuntimeInstallation<
            TemporalHostSchema,
        >,
    ) -> Result<Self::Installed, primary_graph::WorthQueryConditionalRuntimeInstallationDenial>
    {
        if !configuration.install_route {
            return Ok(None);
        }
        let schema = installation.installed_schema();
        let principal_binding = schema
            .principal_binding(TemporalPrincipalBinding::reference())
            .unwrap();
        let authentication = admit_identity_adapter(schema);
        let operation = schema
            .installed_operation(ExecuteTemporal::reference())
            .unwrap();
        let query = schema
            .installed_query_binding::<TemporalIntentCurrentReadBinding>()
            .unwrap();
        let conditional = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &contract::conditional_binding())
            .unwrap()
            .bind_node(TemporalReadyNode::reference())
            .unwrap()
            .bind_host_predicate_provider(configuration.installation_predicate)
            .unwrap()
            .bind_named_clock::<CourtroomClock, _>(configuration.clock_source)
            .unwrap()
            .bind_temporal_intent_projection(
                query.into_query(),
                ApplicationQueryParameterSet::new(),
                IntentProjector,
                domain::WorthQueryTemporalIntentBounds::new(8, 13, 8).unwrap(),
            )
            .unwrap();
        let invariant = Arc::new(installation.retain_invariant_projection_authority());
        let (invoker, _) = Invoker::controlled(configuration.contacts.clone());
        let execution = primary_graph::WorthQueryTemporalOperationExecution::with_authorization(
            Arc::clone(&invariant),
            invoker,
            IntentIdentityField::reference(),
            IntentRevisionField::reference(),
            IntentLifecycleField::reference(),
            "active".to_string(),
            "completed".to_string(),
            primary_graph::WorthQueryPublicTemporalOperationAuthorization,
        )
        .unwrap();
        let (principal_source, _) = PrincipalSource::controlled(authentication);
        let reconstruction = primary_graph::WorthQueryTemporalReconstructionAccess::new(
            principal_binding,
            principal_source,
            IntentIdentityField::reference(),
            "intent-1".to_string(),
        )
        .unwrap();
        let clock = installation.bind_temporal_operation(conditional, execution, reconstruction)?;
        Ok(Some(InstalledTemporalConditional {
            clock,
            definition_predicate: configuration.definition_predicate,
            clock_control: configuration.clock_control,
            contacts: configuration.contacts,
            invariant,
        }))
    }
}

impl WorthQueryApplicationContribution<TemporalHostSchema> for TemporalHostContribution {
    type Configuration = TemporalContributionConfiguration;

    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<TemporalHostSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        contracts.conditional::<TemporalConditional>()?;
        Ok(())
    }

    fn configure(
        configuration: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, TemporalHostSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        setup.conditional::<TemporalConditional>(configuration)
    }
}

#[path = "contribution_installation/publishes_delivers_and_executes.rs"]
mod publishes_delivers_and_executes;
pub(super) use publishes_delivers_and_executes::publishes_delivers_and_executes;

pub(super) fn zero_route_installation_is_denied() {
    let contacts = ContactCounters::default();
    let (installation_predicate, _) = Predicate::controlled(contacts.clone());
    let (definition_predicate, _) = Predicate::controlled(contacts.clone());
    let (clock_source, clock_control) = ClockSource::due();
    let result = application_installation::in_memory_program(
        validated_program(),
        TemporalHostSchema::declaration().unwrap(),
        (TemporalContributionConfiguration {
            installation_predicate,
            definition_predicate: Arc::new(definition_predicate),
            clock_source,
            clock_control,
            contacts,
            install_route: false,
        },),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        WorthQueryInMemoryApplicationLimits::new(
            product_world_resources(1_024),
            runtime::WorthQueryApplicationQueryResourceProfile::new(
                std::num::NonZeroUsize::new(128).expect("concurrent graph work is non-zero"),
            ),
            primary_graph::SignalConditionalEvaluationBudget::development(),
        ),
        |_, _, _| Ok(()),
    );
    match result {
        Err(application_installation::WorthQueryInMemoryApplicationDenial::ConditionalPublication(
            denial,
        )) => assert_eq!(
            denial.kind(),
            primary_graph::WorthQueryConditionalRuntimeInstallationDenialKind::IncompleteBindingInventory,
        ),
        Err(other) => panic!("expected incomplete conditional route denial, got {other:?}"),
        Ok(_) => panic!("a zero-route conditional installation published an application"),
    }
}

#[path = "contribution_installation/source_change.rs"]
mod source_change;
use source_change::change_input;
#[path = "contribution_installation/checkpoint.rs"]
mod checkpoint;
#[path = "contribution_installation/checkpoint_transition.rs"]
mod checkpoint_transition;
#[path = "contribution_installation/publication_limit.rs"]
mod publication_limit;
pub(super) use checkpoint::{
    application_checkpoint_denies_corrupt_incompatible_and_forged_bytes,
    application_checkpoint_restores_fresh_editable_authority,
};

#[path = "contribution_installation/application_fixture.rs"]
mod application_fixture;
#[path = "contribution_installation/request_cancellation.rs"]
mod request_cancellation;

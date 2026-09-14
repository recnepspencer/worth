use std::sync::Arc;

use worth_query_host::facade::{
    application_contribution::{
        WorthQueryApplicationConditionalBinding, WorthQueryApplicationConditionalPackageContract,
        WorthQueryApplicationConditionalProducerAccess, WorthQueryApplicationContribution,
        WorthQueryApplicationContributionContracts, WorthQueryApplicationContributionSetup,
    },
    application_installation::{self, WorthQueryInMemoryApplicationLimits},
    declaration::application_query::ApplicationQueryParameterSet,
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

    const IDENTITY: &'static str = "worth.query.host.courtroom.temporal-conditional.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] = &[];

    fn package_contract() -> WorthQueryApplicationConditionalPackageContract {
        WorthQueryApplicationConditionalPackageContract::new(
            contract::operation_definition().into_portable(),
            contract::conditional_binding().portable().clone(),
            TemporalReadyNode::reference().node_identity(),
        )
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
                domain::WorthQueryTemporalIntentBounds::new(8, 9, 8).unwrap(),
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

pub(super) fn publishes_delivers_and_executes() {
    let contacts = ContactCounters::default();
    let (installation_predicate, _) = Predicate::controlled(contacts.clone());
    let (definition_predicate, _) = Predicate::controlled(contacts.clone());
    let (clock_source, clock_control) = ClockSource::due();
    let configuration = TemporalContributionConfiguration {
        installation_predicate,
        definition_predicate: Arc::new(definition_predicate),
        clock_source,
        clock_control,
        contacts: contacts.clone(),
        install_route: true,
    };
    let application = application_installation::in_memory::<TemporalHostSchema>(
        TemporalHostSchema::declaration().unwrap(),
        (configuration,),
        WorthQueryInMemoryApplicationLimits::new(
            product_world_resources(1_024),
            runtime::WorthQueryApplicationCandidateResourceProfile::bounded(5_120, 2_048, 5_120)
                .unwrap(),
            runtime::WorthQueryApplicationQueryResourceProfile::bounded(
                5_120,
                2_048,
                usize::MAX,
                128,
            )
            .unwrap(),
            primary_graph::SignalConditionalEvaluationBudget::development(),
        ),
        |graph, installed| {
            let principal = installed
                .principal_binding(TemporalPrincipalBinding::reference())
                .unwrap();
            seed_graph(graph, &principal, "blocked", 0, 1, true);
            Ok(())
        },
    )
    .expect("the contribution-composed temporal application must install");
    let installed = application
        .conditional::<TemporalConditional>()
        .expect("the installed conditional handle must remain typed and reachable");
    let installed = installed
        .as_ref()
        .as_ref()
        .expect("the admitted conditional must retain its installed route");
    let branch = application.current_world();
    let selected = application.on_branch(branch).select().unwrap();
    let definition = selected
        .publish_conditional_definition(
            &installed.clock,
            Arc::clone(&installed.definition_predicate),
            &runtime::RuntimeWorldCancellationSource::new().token(),
        )
        .expect("the contribution-installed route must publish a real definition");
    assert!(matches!(
        definition,
        primary_graph::WorthQueryConditionalDefinitionPublicationOutcome::Performed(_)
    ));
    drop(definition);

    let mut publication = change_input(&application, &installed.invariant, branch, "changed")
        .require_committed()
        .expect("the source mutation must publish");
    let change = publication
        .take_performed_relational_product_change()
        .expect("the source mutation must retain World's performed patch");
    let selected = application.on_branch(branch).select().unwrap();
    let delivery = selected
        .deliver_relational_change_to_conditional(&installed.clock, 0, change)
        .expect("Bridge must admit the actual performed World patch");
    assert!(matches!(
        delivery,
        runtime::WorthQueryPerformedRelationalProductChangeDeliveryOutcome::Success(_)
    ));
    drop(delivery);

    installed.clock_control.push(3, 11);
    let observation = selected
        .conditional_clock(&installed.clock)
        .unwrap()
        .observe();
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(observation) =
        observation
    else {
        panic!("the contribution-installed conditional must execute")
    };
    assert_eq!(observation.committed_operation_count(), 1);
    assert_eq!(observation.authoritative_commit_count(), 1);
    assert_eq!(installed.contacts.snapshot(), (1, 1, 1, 1));
}

pub(super) fn zero_route_installation_is_denied() {
    let contacts = ContactCounters::default();
    let (installation_predicate, _) = Predicate::controlled(contacts.clone());
    let (definition_predicate, _) = Predicate::controlled(contacts.clone());
    let (clock_source, clock_control) = ClockSource::due();
    let result = application_installation::in_memory::<TemporalHostSchema>(
        TemporalHostSchema::declaration().unwrap(),
        (TemporalContributionConfiguration {
            installation_predicate,
            definition_predicate: Arc::new(definition_predicate),
            clock_source,
            clock_control,
            contacts,
            install_route: false,
        },),
        WorthQueryInMemoryApplicationLimits::new(
            product_world_resources(1_024),
            runtime::WorthQueryApplicationCandidateResourceProfile::bounded(5_120, 2_048, 5_120)
                .unwrap(),
            runtime::WorthQueryApplicationQueryResourceProfile::bounded(
                5_120,
                2_048,
                usize::MAX,
                128,
            )
            .unwrap(),
            primary_graph::SignalConditionalEvaluationBudget::development(),
        ),
        |_, _| Ok(()),
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

fn change_input(
    application: &primary_graph::WorthQueryPrimaryGraphApplicationRuntime<TemporalHostSchema>,
    invariant: &primary_graph::WorthQueryApplicationInvariantProjectionAuthority<
        TemporalHostSchema,
    >,
    branch: product::WorthQueryProductBranch,
    input: &str,
) -> primary_graph::WorthQueryApplicationCommitOutcome {
    let schema = application.installed_schema();
    let principal_binding = schema
        .principal_binding(TemporalPrincipalBinding::reference())
        .unwrap();
    let authentication = admit_identity_adapter(schema);
    let request = request_scope();
    let external = block_on(authentication.authenticate((), &request)).unwrap();
    let selected = application.on_branch(branch).select().unwrap();
    let principal = selected
        .resolve_authenticated_principal(
            &principal_binding,
            &external,
            &request,
            primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let intent = selected
        .resolve_entity(
            IntentIdentityField::reference(),
            "intent-1".to_string(),
            &request,
            primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let operation = schema
        .installed_operation(AmendTemporal::reference())
        .unwrap();
    let admission = selected
        .authorize_operation(
            &principal,
            &intent,
            &operation,
            Default::default(),
            &request,
        )
        .unwrap();
    let (_, projection, _) = invariant
        .project_admitted_operation(&admission, |reader, scope| {
            reader
                .decision_field(scope, IntentRevisionField::reference())
                .unwrap();
            reader
                .decision_field(scope, IntentLifecycleField::reference())
                .unwrap();
            reader
                .decision_field(scope, IntentGateField::reference())
                .unwrap();
            reader
                .decision_field(scope, IntentDueField::reference())
                .unwrap();
            reader
                .decision_field(scope, IntentInputField::reference())
                .unwrap();
        })
        .unwrap()
        .into_parts();
    let reads = application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap();
    let mut effects = reads
        .complete_projected_dependencies()
        .unwrap()
        .begin_effect_program();
    let intent = effects.existing_entity(&intent).unwrap();
    effects
        .write_field(&intent, IntentRevisionField::reference(), 2_u64)
        .unwrap();
    effects
        .write_field(
            &intent,
            IntentLifecycleField::reference(),
            "active".to_string(),
        )
        .unwrap();
    effects
        .write_field(&intent, IntentDueField::reference(), 11_u64)
        .unwrap();
    effects
        .write_field(&intent, IntentInputField::reference(), input.to_string())
        .unwrap();
    effects
        .write_field(&intent, IntentGateField::reference(), "ready".to_string())
        .unwrap();
    let admitted = product::WorthQueryAdmittedChange::new(
        effects.finish().unwrap(),
        primary_graph::WorthQueryApplicationIdempotencyBinding::new([0x7A; 32], [0xA7; 32]),
    );
    application
        .on_branch(branch)
        .transaction()
        .apply(admitted)
        .commit()
        .unwrap()
}

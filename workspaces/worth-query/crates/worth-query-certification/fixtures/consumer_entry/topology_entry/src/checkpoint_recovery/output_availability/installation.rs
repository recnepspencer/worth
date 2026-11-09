//! One installed population collection shared by Initial and Preserve handlers.
use super::*;
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
use worth_query_host::facade::application_installation::{
    self, WorthQueryProgramApplicationRuntime,
};
use worth_query_host::facade::declaration::authentication::WorthQueryPrincipalMappingStatus;
pub(super) type App = WorthQueryProgramApplicationRuntime<Schema, Program>;
mod enrollment_window;
pub(super) use enrollment_window::EnrollmentWindow;
pub(super) struct Contribution;
impl ApplicationSchemaContribution<Schema> for Contribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("availability-contribution");
    type Members = (
        crate::TopologyCoreMembers<Schema, members::AvailabilityRecords>,
        crate::CheckpointMembers<Schema>,
        ContributionContracts,
    );
}
impl WorthQueryApplicationContribution<Schema> for Contribution {
    type Configuration = (TopologyConfiguration, Arc<EnrollmentWindow>);
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
    ) -> Result<(), worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial>
    {
        <TopologyContribution as WorthQueryApplicationContribution<Schema>>::contracts(contracts)?;
        contracts.producer::<producer::Initial>()?;
        contracts.producer::<producer::Preserve>()?;
        contracts.conditional::<readiness::Readiness<true>>()?;
        contracts.conditional::<readiness::Readiness<false>>()?;
        Ok(())
    }
    fn configure(
        configuration: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial>
    {
        let (configuration, enrollment) = configuration;
        <TopologyContribution as WorthQueryApplicationContribution<Schema>>::configure(
            configuration,
            setup,
        )?;
        let installed = setup
            .derived_collection::<program::QuantityFeature, collection::Quantities, _>(
                collection::Owner { enrollment },
            )?;
        let initial = installed.clone();
        setup.handler::<binding::Initial, _>(publication::InitialHandler(Box::new(
            move |reader, root| match initial.prepare(reader, root) {
                Ok(prepared) => prepared.into_decision_plan(|computed| {
                    match computed.and_then(|value| value.complete()) {
                        Ok(value) => HandlerResult::Completed(value),
                        Err(error) => {
                            HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
                        }
                    }
                }),
                Err(error) => WorthQueryDecisionPlan::immediate(HandlerResult::ExecutionDenied(
                    HandlerExecutionDenial::new(error),
                )),
            },
        )))?;
        setup.handler::<binding::Preserve, _>(publication::PreserveHandler(Box::new(
            move |reader, root| match installed.prepare(reader, root) {
                Ok(prepared) => prepared.into_decision_plan(|computed| {
                    match computed.and_then(|value| value.complete()) {
                        Ok(value) => HandlerResult::Completed(value),
                        Err(error) => {
                            HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
                        }
                    }
                }),
                Err(error) => WorthQueryDecisionPlan::immediate(HandlerResult::ExecutionDenied(
                    HandlerExecutionDenial::new(error),
                )),
            },
        )))?;
        setup.producer::<producer::Initial>(producer::Provider)?;
        setup.producer::<producer::Preserve>(producer::Provider)?;
        setup.conditional::<readiness::Readiness<true>>(())?;
        setup.conditional::<readiness::Readiness<false>>(())
    }
}
pub(super) fn install() -> App {
    install_selected("measure-a")
}
pub(super) fn install_observed(contacts: &crate::producer::ProviderContacts) -> App {
    install_selected_with_window(
        "measure-a",
        Arc::new(EnrollmentWindow::default()),
        Some(contacts.clone()),
    )
}
pub(super) fn install_with_enrollment_window() -> (App, Arc<EnrollmentWindow>) {
    let window = Arc::new(EnrollmentWindow::default());
    (
        install_selected_with_window("measure-a", Arc::clone(&window), None),
        window,
    )
}
pub(super) fn install_selected(demand_key: &str) -> App {
    install_selected_with_window(demand_key, Arc::new(EnrollmentWindow::default()), None)
}
fn install_selected_with_window(
    demand_key: &str,
    window: Arc<EnrollmentWindow>,
    producer_contacts: Option<crate::producer::ProviderContacts>,
) -> App {
    let configuration = ((
        TopologyConfiguration {
            final_feature: crate::planar_final_feature_runtime(),
            setup_calls: Arc::new(AtomicUsize::new(0)),
            invariant_calls: Arc::new(AtomicUsize::new(0)),
            invariant_probe: Arc::new(AtomicUsize::new(0)),
            invariant_read_observer: None,
            producer_authorization_denials: Arc::new(AtomicUsize::new(0)),
            producer_domain_denial: Arc::new(AtomicBool::new(false)),
            producer_contacts,
        },
        window,
    ),);
    application_installation::in_memory_program(
        ApplicationProgramAuthoring::<Schema, Program>::begin()
            .with_schema(&<Schema as worth_query_decl::facade::application_schema::ApplicationSchema>::declaration().expect("the completed schema is valid"))
            .validated_program()
            .unwrap(),
        <Schema as worth_query_decl::facade::application_schema::ApplicationSchema>::declaration().unwrap(),
        configuration,
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        // This journey holds historical deliveries and multiple original output
        // owners through ordinary publication.
        support::limits_with_room(64, 64, 256, support::invalidation()),
        |_phase, graph, installed| {
            let binding = installed
                .principal_binding(ConsumerPrincipalBinding::reference::<Schema>())
                .unwrap();
            graph.bind_principal(
                &binding,
                worth_query_host::facade::primary_graph::WorthQueryApplicationPrincipalKey::new(
                    "model-owner",
                )
                .unwrap(),
                1u64,
                support::external_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            seed(graph, demand_key);
            Ok(())
        },
    )
    .unwrap()
}
fn seed(graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>, demand_key: &str) {
    // The population and measured member are real independent source occurrences.
    for (key, x, y, member) in [
        ("population", 1, 1, false),
        ("population-b", 10, 1, false),
        ("population-c", 1, 10, false),
        ("summary", 1, 1, false),
        ("summary-b", 10, 1, false),
        ("summary-c", 1, 10, false),
        ("measure-a", 1, 10, true),
        ("measure-b", 10, 10, false),
        ("measure-c", 1, 20, false),
    ] {
        let mut seed = WorthQueryApplicationEntitySeed::new(
            Body::reference(),
            WorthQueryApplicationEntityKey::new(key).unwrap(),
        )
        .field(BodyKey::at("BodyKey"), key.to_owned())
        .field(PositionX::at("PositionX"), length(x))
        .field(PositionY::at("PositionY"), length(y))
        .field(Length::at("Length"), length(1));
        if member {
            seed = seed
                .field(members::MemberNumber::at("MemberNumber"), 1u64)
                .field(members::DemandKey::at("DemandKey"), demand_key.to_owned())
                .field(
                    members::MemberKind::at("MemberKind"),
                    members::Kind::Geometry,
                );
        }
        if key == "measure-b" {
            seed = seed
                .field(members::MemberNumber::at("MemberNumber"), 3u64)
                .field(members::DemandKey::at("DemandKey"), "measure-b".to_owned())
                .field(
                    members::MemberKind::at("MemberKind"),
                    members::Kind::Geometry,
                );
        }
        if key == "population" {
            seed = seed
                .field(members::MemberNumber::at("MemberNumber"), 2u64)
                .field(members::DemandKey::at("DemandKey"), "population".to_owned())
                .field(
                    members::MemberKind::at("MemberKind"),
                    members::Kind::Quantity,
                );
        }
        graph.bind_entity(seed).unwrap();
    }
    for (from, to) in [
        ("population", "population-b"),
        ("population-b", "population-c"),
        ("population-c", "population"),
        ("summary", "summary-b"),
        ("summary-b", "summary-c"),
        ("summary-c", "summary"),
        ("measure-a", "measure-b"),
        ("measure-b", "measure-c"),
        ("measure-c", "measure-a"),
    ] {
        graph
            .bind_relation(WorthQueryApplicationRelationSeed::new(
                PlanarSuccessor::reference(),
                format!("{from}:{to}"),
                WorthQueryApplicationEntityKey::new(from).unwrap(),
                WorthQueryApplicationEntityKey::new(to).unwrap(),
            ))
            .unwrap();
    }
    // Independent real collection source used for outside-lane enrollment.
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            members::PopulationMember::reference(),
            "population-b:member",
            WorthQueryApplicationEntityKey::new("population-b").unwrap(),
            WorthQueryApplicationEntityKey::new("measure-b").unwrap(),
        ))
        .unwrap();
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            members::PopulationMember::reference(),
            "summary:member",
            WorthQueryApplicationEntityKey::new("summary").unwrap(),
            WorthQueryApplicationEntityKey::new("population").unwrap(),
        ))
        .unwrap();
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            members::PopulationMember::reference(),
            "population:member",
            WorthQueryApplicationEntityKey::new("population").unwrap(),
            WorthQueryApplicationEntityKey::new("measure-a").unwrap(),
        ))
        .unwrap();
}

pub struct ContributionContracts;
impl worth_query_decl::facade::application_schema::fragment::Registration<Schema>
    for ContributionContracts
{
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
    {
        Ok({
            let schema = builder;
            binding::declare(schema)
                .application_query(read::definition())
                .application_query_binding::<read::Binding>()
        })
    }
}

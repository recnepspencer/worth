//! Real package installation and bootstrap, with a matching static schema type.
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::probe;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::*;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntimeInstaller;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntityKey;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntitySeed;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantFactories;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationInvariantProjectionAuthority;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationPrincipalKey;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationRelationSeed;
use worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::with_test_advancement;
use worth_query_declaration::facade::application_schema::*;
use worth_query_declaration::facade::authentication::*;
use worth_query_installation::facade::*;

pub struct World<const O: bool = true> {
    pub application:
        WorthQueryPrimaryGraphApplicationRuntime<OrbitSchema<O>>,
    pub binding: WorthQueryInstalledPrincipalBinding<
        OrbitSchema<O>,
        IdentityBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    >,
    pub invariant:
        WorthQueryApplicationInvariantProjectionAuthority<OrbitSchema<O>>,
    pub observation: crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::probe::Observation,
}
pub struct Graph {
    pub nodes: usize,
    pub edges: Vec<(u64, u64)>,
    pub active: bool,
    pub blocked: Vec<u64>,
}
pub fn install_orbit(
    graph: Graph,
    observer: crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::probe::Probe,
) -> World {
    install(graph, observer)
}
pub fn install_with_probe(
    graph: Graph,
    observer: crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::probe::Probe<false>,
) -> World<false> {
    install(graph, observer)
}
pub fn install<const O: bool>(
    graph: Graph,
    observer: crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::probe::Probe<O>,
) -> World<O> {
    install_with_visibility(
        graph,
        observer,
        Vec::new(),
        worth_execution::MonotonicClock::system(),
    )
}
pub fn install_orbit_with_component_visibility(
    graph: Graph,
    observer: crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::probe::Probe,
    extra_visibility: Vec<(u64, u64)>,
) -> World {
    install_with_visibility(
        graph,
        observer,
        extra_visibility,
        worth_execution::MonotonicClock::system(),
    )
}
pub fn install_orbit_on_clock(
    graph: Graph,
    observer: crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::probe::Probe,
    clock: worth_execution::MonotonicClock,
) -> World {
    install_with_visibility(graph, observer, Vec::new(), clock)
}
pub fn install_with_visibility<const O: bool>(
    graph: Graph,
    observer: crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::probe::Probe<O>,
    extra_visibility: Vec<(u64, u64)>,
    clock: worth_execution::MonotonicClock,
) -> World<O> {
    let observation = crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::probe::Observation::default();
    with_test_advancement(|phase| {
        let declaration = OrbitSchema::<O>::declaration().unwrap();
        let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
            "invariant_orbit_test",
            1,
            0,
        ))
        .application_schema(declaration.clone())
        .validate()
        .unwrap();
        let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
            .admit(package)
            .unwrap();
        let (runtime, authority) = WorthQueryExecutionRuntimeInstaller::new()
            .install(WorthQueryInstallationGeneration::initial(), [admitted])
            .unwrap()
            .into_parts();
        let schema = runtime
            .installed_packages()
            .bind_application_schema(declaration)
            .unwrap();
        let binding = schema
            .principal_binding(IdentityBinding::reference())
            .unwrap();
        let invariant = schema
            .installed_invariant(
                crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::probe::OrbitRule::reference(),
                ApplicationInvariantExecutionPoint::CommitBoundary,
            )
            .unwrap();
        let mut factories = WorthQueryApplicationInvariantFactories::for_installed_schema(&schema);
        let recorder = observation.clone();
        let blocked = if graph.active {
            graph.blocked
        } else {
            Vec::new()
        };
        factories
            .bind(&invariant, move |resolver| {
                crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::probe::resolve(
                    resolver,
                    observer.clone(),
                    recorder.clone(),
                    blocked.clone(),
                )
            })
            .unwrap();
        let mut bootstrap = authority
            .prepare_primary_graph_with_relational_runtime_and_invariants_for_test(
                &phase.bootstrap_for_test(),
                &runtime,
                &schema,
                worth_relational::facade::runtime::RelationalRuntimeApi::builder()
                    .build()
                    .expect("valid native fixture configuration"),
                worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::resources::test_product_world_resources(clock.clone()),
                factories,
            )
            .unwrap();
        bootstrap
            .bind_principal(
                &binding,
                WorthQueryApplicationPrincipalKey::new("principal-0").unwrap(),
                1,
                WorthQueryExternalPrincipalIdentity::new("https://issuer.example", "alice")
                    .unwrap(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )
            .unwrap();
        for key in 0..graph.nodes as u64 {
            bootstrap
                .bind_entity(
                    WorthQueryApplicationEntitySeed::new(Node::reference(), node_key::<O>(key))
                        .field(NodeKey::reference(), key)
                        .field(Marked::reference(), false),
                )
                .unwrap();
        }
        for (index, (from, to)) in graph.edges.into_iter().enumerate() {
            // Stable independent visibility is authored at bootstrap. Later Next
            // publications alter coupling without rewriting these source paths.
            bootstrap
                .bind_relation(WorthQueryApplicationRelationSeed::new(
                    crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::components::binding::Visible::reference(),
                    format!("visible-{index}"),
                    node_key::<O>(from),
                    node_key::<O>(to),
                ))
                .unwrap();
            bootstrap
                .bind_relation(WorthQueryApplicationRelationSeed::new(
                    Next::reference(),
                    format!("next-{index}"),
                    node_key::<O>(from),
                    node_key::<O>(to),
                ))
                .unwrap();
        }
        for (index, (from, to)) in extra_visibility.into_iter().enumerate() {
            bootstrap
                .bind_relation(WorthQueryApplicationRelationSeed::new(
                    crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::components::binding::Visible::reference(),
                    format!("extra-visible-{index}"),
                    node_key::<O>(from),
                    node_key::<O>(to),
                ))
                .unwrap();
        }
        crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::capability::seed(&mut bootstrap);
        crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::handler::install(&schema, &mut bootstrap);
        crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::components::handler::install(&schema, &mut bootstrap);
        let invariant = bootstrap.retain_invariant_projection_authority();
        let application = bootstrap
            .publish_application_runtime_with_authorization_time_source(
                &phase.bootstrap_for_test(),
                runtime,
                authority,
                schema,
                worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
                worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::authorization_time::AuthorizationTimeController::on_clock(clock),
            )
            .unwrap();
        World {
            application,
            binding,
            invariant,
            observation,
        }
    })
}
pub fn node_key<const O: bool>(key: u64) -> WorthQueryApplicationEntityKey<OrbitSchema<O>, Node> {
    WorthQueryApplicationEntityKey::new(format!("node-{key}")).unwrap()
}
impl<const O: bool> World<O> {
    pub fn committed_version(&self) -> worth_relational::facade::identity::VersionId {
        self.application
            .with_host_advancement(|phase| self.invariant.snapshot(&phase).unwrap().version())
            .unwrap()
    }
    pub fn edges(&self) -> Vec<(u64, u64)> {
        self.application
            .with_host_advancement(|phase| {
                let host = phase.test_host_request_for_test();
                let execution = host
                    .execution_request_for_for_test(self.application.product_runtime())
                    .unwrap();
                let snapshot = self.invariant.snapshot(&phase).unwrap();
                let mut edges = snapshot
                    .relations(&execution, Next::reference())
                    .unwrap()
                    .iter()
                    .map(|edge| {
                        (
                            snapshot
                                .field(&execution, edge.from(), NodeKey::reference())
                                .unwrap()
                                .unwrap(),
                            snapshot
                                .field(&execution, edge.to(), NodeKey::reference())
                                .unwrap()
                                .unwrap(),
                        )
                    })
                    .collect::<Vec<_>>();
                edges.sort();
                edges
            })
            .unwrap()
    }
}

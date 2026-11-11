//! Install a real schema and rule, then publish one complete authored graph.
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::invariant_walks::observation::Observation;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::invariant_walks::schema::*;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntimeInstaller;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntityKey;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntitySeed;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantFactories;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationInvariantProjectionAuthority;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationPrincipalKey;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationRelationSeed;
use worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphBootstrap;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_bootstrap_advancement;
use worth_query_declaration::facade::application_schema::*;
use worth_query_declaration::facade::authentication::*;
use worth_query_installation::facade::*;

pub struct World {
    pub application: WorthQueryPrimaryGraphApplicationRuntime<WalkSchema>,
    pub binding: WorthQueryInstalledPrincipalBinding<
        WalkSchema,
        IdentityBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    >,
    pub invariant: WorthQueryApplicationInvariantProjectionAuthority<WalkSchema>,
    pub observation: Observation,
}

pub struct Graph {
    pub nodes: usize,
    pub edges: Vec<(u64, u64)>,
    pub active: bool,
    pub blocked: Vec<u64>,
}

impl Graph {
    pub fn three_hops(active: bool, blocked: Vec<u64>) -> Self {
        Self {
            nodes: 6,
            edges: vec![(0, 1), (1, 2), (2, 3)],
            active,
            blocked,
        }
    }
}

pub fn install(graph: Graph) -> World {
    install_world(graph, worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::invariant_walks::probe::Probe::disabled())
}

pub fn install_with_probe(
    graph: Graph,
    probe: worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::invariant_walks::probe::Probe,
) -> World {
    install_world(graph, probe)
}

pub fn install_world(
    graph: Graph,
    probe: worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::invariant_walks::probe::Probe,
) -> World {
    let observation = Observation::default();
    // Each node authors its own entity plus NodeOwner and GroupRoot relations.
    let setup_items = graph
        .nodes
        .checked_mul(3)
        .and_then(|rows| rows.checked_add(graph.edges.len()))
        .unwrap();
    const NATIVE_AUTHORING_ALLOWANCE_PER_ROW: u64 = 1_000_000;
    let setup_work = u64::try_from(setup_items)
        .unwrap()
        .checked_mul(NATIVE_AUTHORING_ALLOWANCE_PER_ROW)
        .and_then(|work| work.checked_add(8_000_000))
        .expect("authored graph fixture work is finite");
    with_test_bootstrap_advancement(setup_work, |phase| {
        let declaration = WalkSchema::declaration().unwrap();
        let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
            "invariant_walk_test",
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
                worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::invariant_walks::rule::WalkSafety::reference(),
                ApplicationInvariantExecutionPoint::CommitBoundary,
            )
            .unwrap();
        let mut factories = WorthQueryApplicationInvariantFactories::for_installed_schema(&schema);
        let recorder = observation.clone();
        factories
            .bind(&invariant, move |resolver| {
                worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::invariant_walks::rule::resolve(resolver, recorder)
            })
            .unwrap();
        let diagnostic = schema
            .installed_invariant(
                worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::invariant_walks::probe::WalkProbe::reference(),
                ApplicationInvariantExecutionPoint::CommitBoundary,
            )
            .unwrap();
        factories
            .bind(&diagnostic, move |resolver| {
                worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::invariant_walks::probe::resolve(resolver, probe)
            })
            .unwrap();
        let mut bootstrap = authority.prepare_primary_graph_with_relational_runtime_and_invariants_for_test(
            &phase.bootstrap_for_test(), &runtime, &schema,
            worth_relational::facade::runtime::RelationalRuntimeApi::builder().build().expect("valid native runtime configuration"),
            worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::resources::test_product_world_resources(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system()), factories,
        ).unwrap();
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
        crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::world::seed::graph(&mut bootstrap, graph);
        worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::invariant_walks::decision_walks::handler::install(&schema, &mut bootstrap);
        let invariant = bootstrap.retain_invariant_projection_authority();
        let application = bootstrap
            .publish_application_runtime(
                &phase.bootstrap_for_test(),
                runtime,
                authority,
                schema,
                worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
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

pub fn node_key(key: u64) -> WorthQueryApplicationEntityKey<WalkSchema, Node> {
    WorthQueryApplicationEntityKey::new(format!("node-{key}")).unwrap()
}

impl World {
    pub fn checkpoint(&self) -> crate::published_checkpoint::PublishedCheckpoint {
        self.application
            .with_host_advancement(|phase| {
                let execution = phase.borrowed_request_for_test();
                (*(*(*self.application.runtime_for_test()).primary_provider_for_test())
                    .graph_for_test())
                .with_runtime(&execution, |runtime| {
                    crate::published_checkpoint::capture(runtime, &execution)
                })
            })
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
                                .field(&execution, edge.from(), NodeKey::at("NodeKey"))
                                .unwrap()
                                .unwrap(),
                            snapshot
                                .field(&execution, edge.to(), NodeKey::at("NodeKey"))
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

#[path = "seed/mod.rs"]
pub mod seed;

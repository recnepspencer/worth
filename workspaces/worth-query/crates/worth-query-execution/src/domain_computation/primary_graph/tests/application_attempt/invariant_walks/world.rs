//! Install a real schema and rule, then publish one complete authored graph.
use super::{observation::Observation, rule, schema::*};
use crate::domain_computation::execution_runtime::*;
use crate::domain_computation::primary_graph::*;
use worth_query_declaration::facade::{application_schema::*, authentication::*};
use worth_query_installation::facade::*;

mod seed;

pub(in crate::domain_computation::primary_graph) struct World {
    pub(in crate::domain_computation::primary_graph) application:
        WorthQueryPrimaryGraphApplicationRuntime<WalkSchema>,
    pub(super) binding: WorthQueryInstalledPrincipalBinding<
        WalkSchema,
        IdentityBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    >,
    pub(in crate::domain_computation::primary_graph) invariant:
        WorthQueryApplicationInvariantProjectionAuthority<WalkSchema>,
    pub(super) observation: Observation,
}

pub(in crate::domain_computation::primary_graph) struct Graph {
    pub(in crate::domain_computation::primary_graph) nodes: usize,
    pub(in crate::domain_computation::primary_graph) edges: Vec<(u64, u64)>,
    pub(in crate::domain_computation::primary_graph) active: bool,
    pub(in crate::domain_computation::primary_graph) blocked: Vec<u64>,
}

impl Graph {
    pub(in crate::domain_computation::primary_graph) fn three_hops(
        active: bool,
        blocked: Vec<u64>,
    ) -> Self {
        Self {
            nodes: 6,
            edges: vec![(0, 1), (1, 2), (2, 3)],
            active,
            blocked,
        }
    }
}

pub(in crate::domain_computation::primary_graph) fn install(graph: Graph) -> World {
    install_world(graph, super::probe::Probe::disabled())
}

pub(super) fn install_with_probe(graph: Graph, probe: super::probe::Probe) -> World {
    install_world(graph, probe)
}

fn install_world(graph: Graph, probe: super::probe::Probe) -> World {
    let observation = Observation::default();
    with_test_advancement(|phase| {
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
                rule::WalkSafety::reference(),
                ApplicationInvariantExecutionPoint::CommitBoundary,
            )
            .unwrap();
        let mut factories = WorthQueryApplicationInvariantFactories::for_installed_schema(&schema);
        let recorder = observation.clone();
        factories
            .bind(&invariant, move |resolver| {
                rule::resolve(resolver, recorder)
            })
            .unwrap();
        let diagnostic = schema
            .installed_invariant(
                super::probe::WalkProbe::reference(),
                ApplicationInvariantExecutionPoint::CommitBoundary,
            )
            .unwrap();
        factories
            .bind(&diagnostic, move |resolver| {
                super::probe::resolve(resolver, probe)
            })
            .unwrap();
        let mut bootstrap = authority.prepare_primary_graph_with_relational_runtime_and_invariants(
            &phase.bootstrap_for_test(), &runtime, &schema,
            worth_relational::facade::runtime::RelationalRuntimeApi::builder().build(),
            crate::domain_computation::execution_runtime::product_world::test_product_world_resources(), factories,
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
        seed::graph(&mut bootstrap, graph);
        super::decision_walks::handler::install(&schema, &mut bootstrap);
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

pub(super) fn node_key(key: u64) -> WorthQueryApplicationEntityKey<WalkSchema, Node> {
    WorthQueryApplicationEntityKey::new(format!("node-{key}")).unwrap()
}

impl World {
    pub(super) fn checkpoint(&self) -> Vec<u8> {
        self.application.with_host_advancement(|phase| {
            self.application.capture_application_checkpoint(&phase,
                crate::facade::application_installation::WorthQueryCheckpointCapturePolicy::SystemAllocation,
            ).unwrap()
        }).unwrap().decode().unwrap().native.bytes().to_vec()
    }

    pub(super) fn edges(&self) -> Vec<(u64, u64)> {
        self.application
            .with_host_advancement(|phase| {
                let host = phase.test_host_request();
                let execution = host
                    .execution_request_for(self.application.product_runtime())
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

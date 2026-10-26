//! Real package installation and bootstrap, with a matching static schema type.
use super::{probe, schema::*};
use crate::domain_computation::{execution_runtime::*, primary_graph::*};
use worth_query_declaration::facade::{application_schema::*, authentication::*};
use worth_query_installation::facade::*;

pub(in crate::domain_computation::primary_graph) struct World<const O: bool = true> {
    pub(in crate::domain_computation::primary_graph) application:
        WorthQueryPrimaryGraphApplicationRuntime<OrbitSchema<O>>,
    pub(super) binding: WorthQueryInstalledPrincipalBinding<
        OrbitSchema<O>,
        IdentityBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    >,
    pub(in crate::domain_computation::primary_graph) invariant:
        WorthQueryApplicationInvariantProjectionAuthority<OrbitSchema<O>>,
    pub(super) observation: probe::Observation,
}
pub(super) struct Graph {
    pub(super) nodes: usize,
    pub(super) edges: Vec<(u64, u64)>,
    pub(super) active: bool,
    pub(super) blocked: Vec<u64>,
}
pub(super) fn install_orbit(graph: Graph, observer: probe::Probe) -> World {
    install(graph, observer)
}
pub(super) fn install_with_probe(graph: Graph, observer: probe::Probe<false>) -> World<false> {
    install(graph, observer)
}
fn install<const O: bool>(graph: Graph, observer: probe::Probe<O>) -> World<O> {
    let observation = probe::Observation::default();
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
                probe::OrbitRule::reference(),
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
                probe::resolve(
                    resolver,
                    observer.clone(),
                    recorder.clone(),
                    blocked.clone(),
                )
            })
            .unwrap();
        let mut bootstrap = authority
            .prepare_primary_graph_with_relational_runtime_and_invariants(
                &phase.bootstrap_for_test(),
                &runtime,
                &schema,
                worth_relational::facade::runtime::RelationalRuntimeApi::builder().build(),
                product_world::test_product_world_resources(),
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
            bootstrap
                .bind_relation(WorthQueryApplicationRelationSeed::new(
                    Next::reference(),
                    format!("next-{index}"),
                    node_key::<O>(from),
                    node_key::<O>(to),
                ))
                .unwrap();
        }
        super::declared_queries::capability::seed(&mut bootstrap);
        super::decision::handler::install(&schema, &mut bootstrap);
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
fn node_key<const O: bool>(key: u64) -> WorthQueryApplicationEntityKey<OrbitSchema<O>, Node> {
    WorthQueryApplicationEntityKey::new(format!("node-{key}")).unwrap()
}
impl<const O: bool> World<O> {
    pub(super) fn committed_version(&self) -> worth_relational::facade::identity::VersionId {
        self.application
            .with_host_advancement(|phase| self.invariant.snapshot(&phase).unwrap().version())
            .unwrap()
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

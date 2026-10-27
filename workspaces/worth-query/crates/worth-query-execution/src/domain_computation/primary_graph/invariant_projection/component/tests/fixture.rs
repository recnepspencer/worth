//! A genuinely installed application owns the component definitions and seed graph.
use crate::domain_computation::{execution_runtime::*, primary_graph::*};
use worth_query_declaration::{facade::application_schema::*, *};
use worth_query_installation::facade::*;
mod principal_binding;

#[derive(Debug)]
pub(super) struct Schema;
worth_query_entity!(pub Node for Schema);
worth_query_aspect!(pub NodeData for Schema, Node; identity = AspectIdentity(0x9181_5001), revision = AspectContractRevision(1));
worth_query_field!(pub NodeKey for Schema, Node, NodeData: u64 => U64ApplicationValueBinding, read_only, equality);
pub(super) struct A;
pub(super) struct B;
pub(super) struct Shells;
pub(super) struct OnlyA;
impl A {
    pub(super) fn reference() -> ApplicationRelationRef<Schema, Self, Node, Node> {
        ApplicationRelationRef::from_schema_identifiers(
            "A",
            "Node",
            "Node",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}
impl B {
    pub(super) fn reference() -> ApplicationRelationRef<Schema, Self, Node, Node> {
        ApplicationRelationRef::from_schema_identifiers(
            "B",
            "Node",
            "Node",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}
impl Shells {
    pub(super) fn reference() -> ApplicationComponentSetRef<Schema, Self, Node> {
        ApplicationComponentSetRef::new("Shells", Node::reference())
    }
}
impl OnlyA {
    pub(super) fn reference() -> ApplicationComponentSetRef<Schema, Self, Node> {
        ApplicationComponentSetRef::new("OnlyA", Node::reference())
    }
}
impl ApplicationSchema for Schema {
    const OWNER: &'static str = "worth.tests.component-reader";
    const NAME: &'static str = "Components";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        principal_binding::declare(
            ApplicationSchemaDeclarationBuilder::for_schema()
                .entity(Node::reference())
                .aspect(Node::reference(), NodeData::reference())
                .field(Node::reference(), NodeKey::reference())
                .relation(A::reference(), Node::reference(), Node::reference())
                .relation(B::reference(), Node::reference(), Node::reference())
                .components(
                    ApplicationComponentSetDefinition::new(Shells::reference())
                        .coupled_by(B::reference())
                        .coupled_by(A::reference())
                        .coupled_by(B::reference()),
                )
                .components(
                    ApplicationComponentSetDefinition::new(OnlyA::reference())
                        .coupled_by(A::reference()),
                ),
        )
        .build()
    }
}

pub(super) struct World {
    pub application: WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub invariant: WorthQueryApplicationInvariantProjectionAuthority<Schema>,
    pub other_authority: WorthQueryApplicationInvariantProjectionAuthority<Schema>,
}
pub(super) fn install() -> World {
    with_test_advancement(|phase| {
        let declaration = Schema::declaration().unwrap();
        let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
            Schema::OWNER,
            1,
            0,
        ))
        .application_schema(declaration.clone())
        .validate()
        .unwrap();
        let admitted = WorthQueryInstallationAdmissionProfile::new("components", "local")
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
        let mut bootstrap = authority
            .prepare_primary_graph(
                &phase.bootstrap_for_test(),
                &runtime,
                &schema,
                product_world::test_product_world_resources(),
            )
            .unwrap();
        principal_binding::seed(&mut bootstrap, &schema);
        for key in 0_u64..5 {
            bootstrap
                .bind_entity(
                    WorthQueryApplicationEntitySeed::new(Node::reference(), node_key(key))
                        .field(NodeKey::reference(), key),
                )
                .unwrap();
        }
        seed_edges(&mut bootstrap, A::reference(), &[(0, 1), (1, 0), (3, 3)]);
        // A/B parallel coupling is lawful; this does not duplicate a semantic A edge.
        seed_edges(&mut bootstrap, B::reference(), &[(1, 2), (0, 1)]);
        let invariant = bootstrap.retain_invariant_projection_authority();
        let other_authority = bootstrap.retain_invariant_projection_authority();
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
            invariant,
            other_authority,
        }
    })
}
fn node_key(key: u64) -> WorthQueryApplicationEntityKey<Schema, Node> {
    WorthQueryApplicationEntityKey::new(format!("node-{key}")).unwrap()
}
fn seed_edges<R>(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
    relation: ApplicationRelationRef<Schema, R, Node, Node>,
    edges: &[(u64, u64)],
) {
    for &(from, to) in edges {
        bootstrap
            .bind_relation(WorthQueryApplicationRelationSeed::new(
                relation,
                format!("{}-{from}-{to}", relation.name()),
                node_key(from),
                node_key(to),
            ))
            .unwrap();
    }
}

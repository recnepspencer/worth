//! Fresh installed operation scopes, distinct from relation/entity read markers.
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::*;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::binding::*;
use worth_query_declaration::facade::application_operation::*;
use worth_query_declaration::facade::application_query::ApplicationQueryRootPath;
use worth_query_declaration::facade::application_schema::*;
use worth_query_declaration::*;
pub struct Visible;
impl worth_query_declaration::facade::application_schema::fragment::Relation for Visible {
    const NAME: &'static str = "ComponentVisibility";
    type To = Node;
}
impl Visible {
    pub fn reference<S: OrbitModel>() -> ApplicationRelationRef<S, Self, Node, Node> {
        ApplicationRelationRef::from_schema_identifiers(
            <Self as worth_query_declaration::facade::application_schema::fragment::Relation>::NAME,
            "Node",
            "Node",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}
pub struct Shells;
pub struct Focused;
pub struct Empty;
macro_rules! set_ref {
    ($set:ident) => {
        impl $set {
            pub fn reference<S: OrbitModel>() -> ApplicationComponentSetRef<S, Self, Node> {
                ApplicationComponentSetRef::new(stringify!($set), Node::reference())
            }
        }
    };
}
set_ref!(Shells);
set_ref!(Focused);
set_ref!(Empty);
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct CheckComponentsInput {
    pub root: u64,
}
worth_query_portable_type!(CheckComponentsInput => "worth.query.test.check-components-input.v1");
worth_query_structured_value_binding!(pub CheckComponentsInputBinding for CheckComponentsInput {identity:"worth.query.test.check-components-input.v1"});
worth_query_operation!(pub CheckComponents for S:OrbitModel,input CheckComponentsInputBinding);
worth_query_operation_reads!(CheckComponents => [Node, Next, Shells, Focused, Empty]);
worth_query_mutation_binding!(pub CheckComponentsBinding for CheckComponentsInput,
    schema generic S: OrbitModel, identity "worth.query.test.check-components-mutation.v1",
    input CheckComponentsInputBinding,operation CheckComponents,result CheckOrbitOutcomeBinding,
    idempotency String,identity "worth.query.test.check-components-key.v1",
    decision CheckOrbitDecision,denial CheckOrbitOutcomeBinding,
    handler identity "worth.query.test.check-components-handler.v1",
    outputs NoApplicationMutationOutputs,source NoApplicationMutationSource,
    principal binding IdentityBinding::reference(),binding_type IdentityBinding,
    mapping ExternalMapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
    scope NodeKey,
    field NodeKey::at("NodeKey"),value_field root,
);
pub fn declare<S: OrbitModel>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    schema
        .components(
            ApplicationComponentSetDefinition::new(Shells::reference())
                .coupled_by(Next::reference()),
        )
        .components(
            ApplicationComponentSetDefinition::new(Focused::reference())
                .coupled_by(Next::reference()),
        )
        .components(
            ApplicationComponentSetDefinition::new(Empty::reference())
                .coupled_by(Next::reference()),
        )
        .operation(
            CheckComponents::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_read_entity(CheckComponents::reference(), Node::reference())
        .operation_read_field(CheckComponents::reference(), NodeKey::at("NodeKey"))
        .operation_component_read(ApplicationOperationComponentReadDefinition::new(
            CheckComponents::reference(),
            Shells::reference(),
            ApplicationQueryRootPath::from(Node::reference()).forward_closure(Visible::reference()),
        ))
        .operation_component_read(ApplicationOperationComponentReadDefinition::new(
            CheckComponents::reference(),
            Focused::reference(),
            ApplicationQueryRootPath::from(Node::reference()),
        ))
        .operation_component_read(ApplicationOperationComponentReadDefinition::new(
            CheckComponents::reference(),
            Empty::reference(),
            ApplicationQueryRootPath::from(Node::reference())
                .forward_closure(Visible::reference())
                .where_equal(
                    Marked::at("Marked"),
                    ApplicationEncodedScalarValue::<BoolApplicationValueBinding>::try_new(true)
                        .unwrap(),
                )
                .expect("record field contract"),
        ))
        .application_mutation_binding::<CheckComponentsBinding<S>>()
}

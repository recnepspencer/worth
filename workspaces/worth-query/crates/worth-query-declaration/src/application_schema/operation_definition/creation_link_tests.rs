//! Two effects of one relation derive one membership, not duplicate declarations.
use crate::application_schema::*;
struct Schema;
use crate::application_schema::fragment::{self, *};
struct Node;
impl Entity for Node {
    const NAME: &'static str = "Node";
    type Aspects = ();
    type Relations = (Children,);
}
impl Node {
    fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(Schema, Self)
    }
}
impl Fragment for Schema {
    const NAME: &'static str = "creation";
    type Bindings = NoBindings;
    type Members = (Node,);
}
impl ApplicationSchema for Schema {
    const OWNER: &'static str = "tests";
    const NAME: &'static str = "creation";
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}

struct Children;
impl Relation for Children {
    const NAME: &'static str = "Children";
    type To = Node;
}
impl Children {
    fn reference() -> ApplicationRelationRef<Schema, Self, Node, Node> {
        relation(Schema, Node, Self)
    }
}

crate::worth_query_structured_value_binding!(Input for () { identity: "worth.tests.links-input.v1" });
crate::worth_query_operation!(Spawn for Schema, input Input);
crate::worth_query_operation_reads!(Spawn => [Node]);
crate::worth_query_operation_creates!(Spawn => [Node]);
crate::worth_query_operation_links!(Spawn => [Children]);

#[test]
fn repeated_relation_effects_derive_one_link_membership_and_keep_both_edges() {
    let body = Spawn::reference()
        .body()
        .in_scope(Node::reference())
        .unwrap()
        .create_in_scope(Node::reference(), "child")
        .unwrap()
        .link_from_scope(Children::reference(), "forward")
        .unwrap()
        .link_to_scope(Children::reference(), "backward")
        .unwrap()
        .finish()
        .unwrap();
    let schema = ApplicationSchemaDeclarationBuilder::<Schema>::from_fragment(Schema)
        .unwrap()
        .operation(
            Spawn::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish()
                .with_body(body)
                .unwrap(),
        )
        .operation_projection_work_budget(Spawn::reference(), 16)
        .build()
        .unwrap();
    assert_eq!(
        schema
            .erased()
            .members()
            .iter()
            .filter(|member| matches!(
                member,
                ApplicationSchemaMember::OperationProgram {
                    target: ApplicationOperationProgramTarget::Link { .. },
                    ..
                }
            ))
            .count(),
        1
    );
    let links = schema
        .erased()
        .members()
        .iter()
        .find_map(|member| match member {
            ApplicationSchemaMember::OperationBody { body, .. } => {
                body.creation().map(|creation| creation.links())
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(links.len(), 2);
    assert_ne!(links[0].direction, links[1].direction);
}

//! More than eight outgoing relations compose through the same membership tuples.
use crate::application_schema::{fragment::*, *};
struct Suite;
struct Node;
impl Fragment for Suite {
    const NAME: &'static str = "nested_relations";
    type Bindings = NoBindings;
    type Members = (Node,);
}
impl ApplicationSchema for Suite {
    const OWNER: &'static str = "nested_relations";
    const NAME: &'static str = "nested_relations";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        declare(Self)
    }
}
impl Entity for Node {
    const NAME: &'static str = "node";
    type Aspects = ();
    type Relations = ((A, B, C, D, E, F, G, H), I);
}
struct A;
impl Relation for A {
    const NAME: &'static str = "a";
    type To = Node;
}
struct B;
impl Relation for B {
    const NAME: &'static str = "b";
    type To = Node;
}
struct C;
impl Relation for C {
    const NAME: &'static str = "c";
    type To = Node;
}
struct D;
impl Relation for D {
    const NAME: &'static str = "d";
    type To = Node;
}
struct E;
impl Relation for E {
    const NAME: &'static str = "e";
    type To = Node;
}
struct F;
impl Relation for F {
    const NAME: &'static str = "f";
    type To = Node;
}
struct G;
impl Relation for G {
    const NAME: &'static str = "g";
    type To = Node;
}
struct H;
impl Relation for H {
    const NAME: &'static str = "h";
    type To = Node;
}
struct I;
impl Relation for I {
    const NAME: &'static str = "i";
    type To = Node;
}
#[test]
fn every_nested_outgoing_relation_is_lowered_once() {
    let declaration = Suite::declaration().unwrap();
    let mut names = declaration
        .erased()
        .members()
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::Relation {
                relation, from, to, ..
            } => {
                assert_eq!(from, "node");
                assert_eq!(to, "node");
                Some(relation.as_str())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    names.sort_unstable();
    assert_eq!(names, ["a", "b", "c", "d", "e", "f", "g", "h", "i"]);
}

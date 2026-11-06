//! Scalar record admission uses the owner's codec without requiring Clone.
use super::*;
use worth_query_declaration::facade::application_schema::fragment::{self, *};
pub(super) struct TreeSchema;
impl Fragment for TreeSchema {
    const NAME: &'static str = "TreeSchema";
    type Bindings = Bindings;
    type Members = (Node,);
}
impl ApplicationSchema for TreeSchema {
    const OWNER: &'static str = "TreeSchema";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
pub(super) struct Bindings;
impl BindingSet for Bindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<Tree> for Bindings {
    type Binding = TreeBinding;
    const WRITABLE: bool = false;
}
pub(super) struct Node;
impl fragment::Entity for Node {
    const NAME: &'static str = "Node";
    type Aspects = (Facts,);
    type Relations = ();
}
pub(super) struct Facts;
impl fragment::Aspect for Facts {
    const NAME: &'static str = "Facts";
    type Row = Tree;
}
pub(super) type Saved = FieldSelection<TreeSchema, Node, Facts, Tree>;

//! Scalar record admission uses the owner's codec without requiring Clone.
use super::*;
use worth_query_declaration::facade::application_schema::fragment::{self, *};
pub(super) struct IncompleteSchema;
impl Fragment for IncompleteSchema {
    const NAME: &'static str = "IncompleteSchema";
    type Bindings = Bindings;
    type Members = (IncompleteNode,);
}
impl ApplicationSchema for IncompleteSchema {
    const OWNER: &'static str = "IncompleteSchema";
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
impl Scalar<Incomplete> for Bindings {
    type Binding = IncompleteBinding;
    const WRITABLE: bool = false;
}
pub(super) struct IncompleteNode;
impl fragment::Entity for IncompleteNode {
    const NAME: &'static str = "IncompleteNode";
    type Aspects = (IncompleteFacts,);
    type Relations = ();
}
pub(super) struct IncompleteFacts;
impl fragment::Aspect for IncompleteFacts {
    const NAME: &'static str = "IncompleteFacts";
    type Row = Incomplete;
}

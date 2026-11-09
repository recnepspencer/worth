//! Variable authored ownership through the ordinary source-subgraph host.
use super::*;
mod schema;
pub(super) use schema::*;
mod operations;
pub(super) use operations::*;
mod correction;
mod creation;
pub(super) mod links;
mod replacement;
mod retirement;
pub(super) use correction::{UndoCreate, UndoRetire};
use replacement::*;
pub(super) mod query;
#[derive(Clone, Debug, Eq, PartialEq, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Row {
    pub key: String,
    pub value: u64,
    pub next: Option<String>,
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Image {
    pub key: String,
    pub value: u64,
    pub members: Vec<Row>,
}
pub type AnchorTarget = WorthQueryInvariantMutationTarget<Schema, Source>;
pub type MemberTarget = WorthQueryInvariantMutationTarget<Schema, Member>;
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    links::declare(query::declare(operations::declare(schema)))
}
pub fn configure(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    extra_member: bool,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    setup.handler::<links::Binding, _>(links::Handler)?;
    setup.handler::<CreateBinding, _>(creation::Handler)?;
    setup.handler::<RestoreBinding, _>(creation::Handler)?;
    setup.handler::<RetireBinding, _>(retirement::Handler { extra_member })
}

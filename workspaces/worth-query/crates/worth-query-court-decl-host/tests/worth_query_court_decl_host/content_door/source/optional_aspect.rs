//! Optional parent content remains a complete Native Source dependency.
use super::*;
mod courts;
pub mod operation;
mod provider;
mod rule;
pub use rule::{resolve, CheckOptionalContent};
pub mod query;
pub use provider::Provider;
pub struct Index;
impl ApplicationManagedIndexBinding<Schema> for Index {
    type SourceBinding = query::SourceBinding;
    type ProbeBinding = ProbeBinding;
    type Entity = Record;
    const NAME: &'static str = "optional_content_index";
    const PROVIDER: &'static str = "optional-content.provider.v1";
    const PROVIDER_VERSION: u32 = 1;
    const STRUCTURE: ApplicationManagedIndexSourceStructure =
        ApplicationManagedIndexSourceStructure::BinarySourceTree;
    const MEMBERSHIP: ApplicationManagedIndexMembership =
        ApplicationManagedIndexMembership::ExhaustiveSourceTree;
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    operation::declare(schema)
        .application_query(query::source())
        .application_query_binding::<query::SourceBinding>()
        .application_query(query::find())
        .application_query_binding::<query::FindBinding>()
        .managed_index(ApplicationManagedIndexDefinition::<Schema, Index>::new())
        .invariant(rule::definition())
}

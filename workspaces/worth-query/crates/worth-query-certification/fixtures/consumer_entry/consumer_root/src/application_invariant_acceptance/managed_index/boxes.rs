//! A sibling public provider: exact fixture boxes and actual scoped capability reads.
pub(super) mod access;
mod authority;
mod coordinates;
mod field;
mod geometry;
mod provider;
mod query;
pub(super) mod seed;
pub(super) mod status;
use super::schema::{Catalog, Item};
pub(super) use coordinates::Coordinates;
pub(super) use field::ItemBounds;
pub(super) use provider::{BoxIndex, Calls, Provider};
use worth_query_decl::facade::{
    application_query::ApplicationManagedIndexDefinition, application_schema::*,
};
pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    let schema = schema
        .unit(field::FixtureLength::reference())
        .field(Item::reference(), field::ItemBounds::reference())
        .application_query(query::source_definition())
        .application_query_binding::<query::SourceBinding>()
        .application_query(query::public_definition())
        .application_query_binding::<query::PublicBinding>()
        .application_query(query::governed_definition())
        .application_query_binding::<query::GovernedBinding>()
        .managed_index(ApplicationManagedIndexDefinition::<Catalog, BoxIndex>::new());
    status::declare(access::declare(schema))
}
pub(super) fn configure(
    calls: std::sync::Arc<Calls>,
    incompatible: bool,
    setup:&mut worth_query_host::facade::application_contribution::WorthQueryApplicationContributionSetup<'_,Catalog>,
) -> Result<(), worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    if incompatible {
        setup.managed_index::<BoxIndex, _>(provider::IncompatibleVersion(Provider(calls)))?;
    } else {
        setup.managed_index::<BoxIndex, _>(Provider(calls))?;
    }
    setup.handler::<status::Binding, _>(status::Handler)
}

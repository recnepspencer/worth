//! Protected Parent declarations reuse the actual grant and READ-policy owners.
#[path = "../boxes/access.rs"]
pub(super) mod access;
#[cfg(test)]
mod court;
pub(super) mod expectation;
mod page;
mod query;
mod seed;
#[path = "../boxes/status.rs"]
pub(super) mod status;
use super::schema::Catalog;
use worth_query_decl::facade::application_schema::*;
pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    let schema = status::declare(access::declare(schema));
    schema
        .application_query(page::definition())
        .application_query_binding::<page::Binding>()
        .application_query(page::blocked_definition())
        .application_query_binding::<page::BlockedBinding>()
        .application_query(query::exact::definition())
        .application_query_binding::<query::exact::Binding>()
        .application_query(query::exact::blocked_definition())
        .application_query_binding::<query::exact::BlockedBinding>()
        .application_query(query::definition())
        .application_query_binding::<query::Binding>()
        .application_query_binding::<query::AliasBinding>()
        .application_query(query::blocked_definition())
        .application_query_binding::<query::BlockedBinding>()
        .application_mutation_binding::<expectation::Binding>()
}
pub(super) use seed::populate;

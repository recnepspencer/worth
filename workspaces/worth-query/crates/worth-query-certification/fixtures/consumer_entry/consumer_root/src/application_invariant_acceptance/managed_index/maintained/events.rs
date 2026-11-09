//! A separate ordinary event-text composition shares the actual prefix implementation.
#[path = "nested/authoring.rs"]
mod authoring;
pub(super) mod composition;
#[cfg(test)]
mod court;
mod history;
#[path = "../mutation.rs"]
mod mutation;
mod paths;
mod plain;
#[path = "../prefix.rs"]
pub(super) mod prefix;
#[path = "nested/query.rs"]
mod query;
pub(super) mod rule;
pub(super) mod schema;
#[path = "nested/shape.rs"]
mod shape;
const SOURCE_MAX_RESULTS: usize = 8192;
const PREFIX_MEMBERSHIP:worth_query_decl::facade::application_query::ApplicationManagedIndexMembership=worth_query_decl::facade::application_query::ApplicationManagedIndexMembership::CheckedSummarySourceTree;
const PREFIX_REFINEMENT: Option<
    worth_query_decl::facade::application_query::ApplicationManagedIndexRefinement,
> = None;
const PREFIX_MAINTENANCE:
    worth_query_decl::facade::application_query::ApplicationManagedIndexMaintenance =
    worth_query_decl::facade::application_query::ApplicationManagedIndexMaintenance::WhenRequired;
fn source_path() -> worth_query_decl::facade::application_query::ApplicationQueryRootPath<
    schema::Catalog,
    schema::Directory,
    schema::Item,
> {
    worth_query_decl::facade::application_query::ApplicationQueryRootPath::from(
        schema::Directory::reference(),
    )
    .forward_derived(schema::LiveItems::reference())
}

const PREFIX_SOURCE_SCOPE: worth_query_decl::facade::application_query::ApplicationManagedIndexSourceScope = worth_query_decl::facade::application_query::ApplicationManagedIndexSourceScope::OrderedPredecessor;

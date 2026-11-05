//! An explicit nested text capability composition in the existing consumer binary.
mod authoring;
pub(super) mod composition;
#[cfg(test)]
mod court;
pub(in super::super) mod edit;
mod host;
#[path = "../mutation.rs"]
mod mutation;
#[path = "../prefix.rs"]
pub(super) mod prefix;
mod query;
pub(super) mod rule;
pub(super) mod schema;
mod shape;
const SOURCE_MAX_RESULTS: usize = 8192;
const PREFIX_MEMBERSHIP: worth_query_decl::facade::application_query::ApplicationManagedIndexMembership = worth_query_decl::facade::application_query::ApplicationManagedIndexMembership::CheckedSummarySourceTree;
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
    .forward(schema::Contains::reference())
}

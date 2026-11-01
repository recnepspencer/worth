//! A second authored catalog composition opts into the real maintained Source rule.
//! The original explicit-rebuild catalog and its Phase1 courts remain unchanged.
mod authoring;
mod bounds;
mod correction;
#[cfg(test)]
mod court;
mod generated;
mod geometry;
mod handler;
mod host;
#[path = "mutation.rs"]
mod mutation;
#[path = "prefix.rs"]
mod prefix;
mod protected;
#[path = "query.rs"]
mod query;
mod refinement;
mod rule;
mod schema;

const PREFIX_MEMBERSHIP: worth_query_decl::facade::application_query::ApplicationManagedIndexMembership =
    worth_query_decl::facade::application_query::ApplicationManagedIndexMembership::CheckedSummarySourceTree;

const SOURCE_MAX_RESULTS: usize = 8192;

const PREFIX_REFINEMENT: Option<
    worth_query_decl::facade::application_query::ApplicationManagedIndexRefinement,
> = Some(
    worth_query_decl::facade::application_query::ApplicationManagedIndexRefinement::row_local(
        "managed-catalog.exact-name",
        1,
    ),
);

const PREFIX_MAINTENANCE:
    worth_query_decl::facade::application_query::ApplicationManagedIndexMaintenance =
    worth_query_decl::facade::application_query::ApplicationManagedIndexMaintenance::WhenRequired;

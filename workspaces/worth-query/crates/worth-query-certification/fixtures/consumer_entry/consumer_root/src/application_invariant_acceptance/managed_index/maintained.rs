//! A second authored catalog composition opts into the real maintained Source rule.
//! The original explicit-rebuild catalog and its Phase1 courts remain unchanged.
mod authoring;
mod bounds;
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
mod rule;
mod schema;

const PREFIX_MEMBERSHIP: worth_query_decl::facade::application_query::ApplicationManagedIndexMembership =
    worth_query_decl::facade::application_query::ApplicationManagedIndexMembership::CheckedSummarySourceTree;

const SOURCE_MAX_RESULTS: usize = 8192;

//! Fixed refusing catalog: explicit bootstrap initialization, never a policy flip.
#[cfg(test)]
mod court;
#[path = "directory.rs"]
mod directory;
mod host;
#[path = "mutation.rs"]
mod mutation;
#[path = "prefix.rs"]
mod prefix;
#[path = "query.rs"]
mod query;
mod rule;
mod schema;

const PREFIX_MEMBERSHIP: worth_query_decl::facade::application_query::ApplicationManagedIndexMembership =
    worth_query_decl::facade::application_query::ApplicationManagedIndexMembership::CheckedSummarySourceTree;

const SOURCE_MAX_RESULTS: usize = 16;

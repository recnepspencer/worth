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

const PREFIX_REFINEMENT: Option<
    worth_query_decl::facade::application_query::ApplicationManagedIndexRefinement,
> = None;

const PREFIX_MAINTENANCE: worth_query_decl::facade::application_query::ApplicationManagedIndexMaintenance = worth_query_decl::facade::application_query::ApplicationManagedIndexMaintenance::Native { cold: worth_query_decl::facade::application_query::ApplicationManagedIndexColdSource::AdmitRebuild };

//! Legitimate Candidates profile; every slot still comes from the actual public Source.
mod boxes;
mod court;
mod directory;
mod host;
mod lifecycle;
mod mutation;
mod prefix;
mod query;
mod schema;
const PREFIX_MEMBERSHIP:
    worth_query_decl::facade::application_query::ApplicationManagedIndexMembership =
    worth_query_decl::facade::application_query::ApplicationManagedIndexMembership::Candidates;

const SOURCE_MAX_RESULTS: usize = 16;

const PREFIX_REFINEMENT: Option<
    worth_query_decl::facade::application_query::ApplicationManagedIndexRefinement,
> = None;

const PREFIX_MAINTENANCE:
    worth_query_decl::facade::application_query::ApplicationManagedIndexMaintenance =
    worth_query_decl::facade::application_query::ApplicationManagedIndexMaintenance::WhenRequired;

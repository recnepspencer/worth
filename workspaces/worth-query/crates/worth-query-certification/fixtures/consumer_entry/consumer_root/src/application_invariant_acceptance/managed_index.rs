//! Public committed index consumers, separate from the original application schema.
mod boxes;
mod directory;
mod expectations;
mod host;
mod journey;
#[cfg(test)]
pub(super) mod lifecycle;
mod maintained;
mod mutation;
mod prefix;
mod query;
mod refusing;
mod schema;
#[cfg(test)]
mod value_edits;

const PREFIX_MEMBERSHIP: worth_query_decl::facade::application_query::ApplicationManagedIndexMembership =
    worth_query_decl::facade::application_query::ApplicationManagedIndexMembership::ExhaustiveSourceTree;

const SOURCE_MAX_RESULTS: usize = 16;

const PREFIX_REFINEMENT: Option<
    worth_query_decl::facade::application_query::ApplicationManagedIndexRefinement,
> = None;

const PREFIX_MAINTENANCE:
    worth_query_decl::facade::application_query::ApplicationManagedIndexMaintenance =
    worth_query_decl::facade::application_query::ApplicationManagedIndexMaintenance::WhenRequired;

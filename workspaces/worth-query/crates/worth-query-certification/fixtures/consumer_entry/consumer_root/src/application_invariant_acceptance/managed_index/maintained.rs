//! A second authored catalog composition opts into the real maintained Source rule.
//! The original explicit-rebuild catalog and its Phase1 courts remain unchanged.
mod authoring;
mod bounds;
mod correction;
#[cfg(test)]
mod court;
mod events;
mod generated;
mod geometry;
mod handler;
mod host;
mod mutation;
mod nested;
#[cfg(test)]
mod ordered_refusing;
mod prefix;
mod protected;
mod query;
mod refinement;
mod rule;
pub(super) mod rule_observation;
mod schema;
mod source_authorization;
pub(super) mod source_observation;

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

const PREFIX_SOURCE_SCOPE: worth_query_decl::facade::application_query::ApplicationManagedIndexSourceScope = worth_query_decl::facade::application_query::ApplicationManagedIndexSourceScope::SameAdmittedScope;

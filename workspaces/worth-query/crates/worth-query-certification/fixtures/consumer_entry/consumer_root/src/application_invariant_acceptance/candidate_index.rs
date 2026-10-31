//! Legitimate Candidates profile; every slot still comes from the actual public Source.
#[path = "managed_index/boxes/mod.rs"]
mod boxes;
mod court;
#[path = "managed_index/directory.rs"]
mod directory;
#[path = "managed_index/host.rs"]
mod host;
mod lifecycle;
#[path = "managed_index/mutation.rs"]
mod mutation;
#[path = "managed_index/prefix.rs"]
mod prefix;
#[path = "managed_index/query.rs"]
mod query;
#[path = "managed_index/schema.rs"]
mod schema;
const PREFIX_MEMBERSHIP:
    worth_query_decl::facade::application_query::ApplicationManagedIndexMembership =
    worth_query_decl::facade::application_query::ApplicationManagedIndexMembership::Candidates;

const SOURCE_MAX_RESULTS: usize = 16;

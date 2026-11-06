//! Generic owned-model history over two source-scope-owned producer families.
use worth_query_decl::facade::{
    application_history::*, application_operation::*, application_program::*, application_query::*,
    application_schema::*, authentication::*, *,
};
use worth_query_host::facade::{
    application_contribution::*, application_entry::*, application_installation::*,
    primary_graph::*,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    host::host_limits, operator_identity::*,
};
mod schema;
use schema::*;
mod courts;
mod custody;
mod edit;
mod family;
mod historical_source;
mod member_delete;
mod member_edit;
mod navigation;
mod permission;
mod program;
mod query;
mod world;
use family::{
    child, downstream, first, from_first, restricted, restricted_from_first, restricted_owned,
    second, upstream,
};
mod generated_source;

pub struct History;
worth_query_portable_type!(History => "worth.owned-outputs.history.v1");
impl ApplicationHistoryBinding<Schema> for History {
    type Access = query::model::Read;
    const IDENTITY: &'static str = "OwnedOutputsHistory";
}
pub struct Completion;
impl<B: ApplicationMutationBinding<Schema>> ApplicationActionDelivery<Schema, B> for Completion {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "owned-outputs.completion.v1";
}

mod forest;

mod incomplete_forest;

mod dependent_forest;

mod healthy_verification;
mod source_projection;

mod mixed_verification;

mod build_change;

mod cold_selection;

mod hierarchy;

mod authored_member;

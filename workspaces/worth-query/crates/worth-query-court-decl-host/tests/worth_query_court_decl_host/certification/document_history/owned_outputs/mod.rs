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
mod member_edit;
mod navigation;
mod program;
mod query;
mod world;
use family::{first, second};

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

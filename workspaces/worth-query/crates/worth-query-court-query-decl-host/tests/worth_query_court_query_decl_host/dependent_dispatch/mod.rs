//! Record-only producer graph: each generated source retains its immediate parent.
use std::sync::{Arc, Mutex};
use worth_query_decl::facade::{
    application_operation::*, application_program::*, application_query::*, application_schema::*,
    *,
};
use worth_query_host::facade::{
    application_contribution::*, application_entry::*, application_installation as installation,
    primary_graph::*,
};
mod courts;
#[macro_use]
mod readiness;
mod lifecycle_basis;
mod producer;
mod producer_cell;
mod program;
mod query;
mod realization_read;
mod schema;
mod source_control;
mod split_courts;
mod split_interruptions;
mod split_lifecycle;
mod split_model;
mod split_producer;
mod split_program;
mod split_support;
mod world;
use producer::*;
use program::*;
use query::*;
use schema::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    pub key: String,
    pub tier: u8,
    pub preserve: bool,
    pub prior: bool,
}
pub type Selections = Arc<Mutex<Vec<Selection>>>;

//! A fixed source graph and a generated graph with immutable topology.
use std::sync::{Arc, Mutex};
use worth_query_decl::facade::{
    application_aftermath::*, application_operation::*, application_program::*,
    application_query::*, application_schema::*, authentication::*, *,
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
mod graph_read;
use graph_read::{GraphRow, MemberRow};
mod correction;
pub(crate) mod courts;
mod edit;
mod fingerprint;
mod generation;
mod producer;
mod program;
mod readiness;
mod world;
type Hashes = Arc<Mutex<Vec<[u8; 32]>>>;

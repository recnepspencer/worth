//! A relation-only recorded inverse undoes a creation by reading its retained link.
//!
//! Spawn creates one node, optionally a part, and links the spawned node to the
//! part or, without one, to the scope node. Its correction declares only the
//! link's relation pre-image, as a CAD creation declares its body thread: the
//! inverse learns whether the optional part was born from Native output
//! correspondence and checks the retained relation rows.
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
mod correction;
mod courts;
mod handlers;
mod operations;
mod query;
mod schema;
mod world;
use operations::*;
use schema::*;

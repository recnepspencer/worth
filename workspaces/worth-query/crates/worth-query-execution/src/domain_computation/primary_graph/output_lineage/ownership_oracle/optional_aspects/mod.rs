//! Explicit optional-aspect semantics through an installed Query producer.
use super::*;
use std::marker::PhantomData;
use worth_query_decl::facade::{
    application_operation::*, application_program::*, application_schema::*, worth_query_aspect,
    worth_query_entity, worth_query_field, worth_query_operation, worth_query_operation_creates,
    worth_query_operation_links, worth_query_operation_reads, worth_query_operation_writes,
};
mod schema;
use schema::*;
mod outputs;
use outputs::*;
mod field_patch_cases;
mod operation;
mod ordinary;
mod own_effect;
mod required_content_read;
use operation::*;
mod archive;
mod court;
mod installation;
mod journey;
mod producer;
use producer::*;
mod cases;
mod read_commit;
mod readiness;
mod retained_recovery;
mod tracked_reads;

//! Explicit optional-aspect semantics through an installed Query producer.
use super::*;
use std::marker::PhantomData;
use worth_query_decl::facade::{
    application_operation::*, application_program::*, application_schema::*, worth_query_operation,
    worth_query_operation_creates, worth_query_operation_links, worth_query_operation_reads,
    worth_query_operation_writes,
};
mod schema;
mod source_text;
pub(super) use schema::SourceNotes;
use schema::*;
mod outputs;
use outputs::*;
mod field_patch_cases;
mod foreign_admission;
mod operation;
mod ordinary;
mod own_effect;
mod required_content_read;
mod source_admission;
use operation::*;
mod archive;
mod court;
mod installation;
mod journey;
mod producer;
mod producer_observation;
use producer::*;
mod cases;
mod read_commit;
mod readiness;
mod retained_recovery;
mod tracked_reads;

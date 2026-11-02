//! Generic ordered numerical content through only the public Decl and Host doors.
use worth_foundational::facade::{CanonicalDigestId, ContentRefId};
use worth_query_decl::facade::{
    application_operation::*, application_program::*, application_schema::*, authentication::*, *,
};
use worth_query_host::facade::{
    application_contribution::*, application_entry::*, application_installation::*,
    primary_graph::*,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    host::host_limits, operator_identity::*,
};
mod codec;
mod controls;
mod courts;
mod created_content;
mod operation;
mod revision;
mod revision_courts;
mod schema;
mod source;
mod world;
use schema::*;

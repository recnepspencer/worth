//! A distinct, statically declared application proves a nonempty workflow output inventory.
use super::document;
use document::schema::{
    Document, DocumentIdentityField, DocumentRetentionField, ExternalIdentityField,
    ExternalMapping, MappingStatusField, MappingTarget, Principal, PrincipalIdentityField,
    RetentionModel, SetRetentionInput, SetRetentionInputBinding,
};
use document::workflow::declaration::*;
use document::workflow::{WorkflowDocumentOwner, WorkflowGrantGrantee, WorkflowGrantGrantor};
use worth_query_decl::facade::{
    application_capability::*, application_operation::*, application_program::*,
    application_schema::*,
};
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::primary_graph::*;
mod binding;
mod control;
mod journey;
mod seed;
mod storage;
use binding::*;
use control::*;
use storage::*;

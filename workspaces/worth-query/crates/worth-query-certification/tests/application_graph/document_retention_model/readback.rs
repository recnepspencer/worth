//! What one occurrence actually says a document's retention is.
//!
//! Every verdict in this court is checked against a committed read rather than
//! an outcome variant, so a denial that silently landed an effect, or a
//! performed commit that wrote nothing, fails here.

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationReadObservation, WorthQueryApplicationRequestExt,
};
use worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_host::facade::product::WorthQueryProductBranch;

use super::operator_identity::{authenticate_operator, request_scope};
use super::retention_entry::{DocumentRetentionRead, DOCUMENT_IDENTITY};
use super::schema::DocumentRetentionSchema;

/// Reads the document's committed retention on one exact occurrence.
pub fn read_retention(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<DocumentRetentionSchema>,
    branch: WorthQueryProductBranch,
) -> u64 {
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .query(DocumentRetentionRead {
            identity: DOCUMENT_IDENTITY.to_owned(),
        })
        .execute()
        .expect("the ordinary read must execute on this occurrence")
        .rows()[0]
        .retention_days
}

/// Retains this occurrence's published head, which is what a denial before
/// effects must leave exactly where it found it.
pub fn observe_head(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<DocumentRetentionSchema>,
    branch: WorthQueryProductBranch,
) -> WorthQueryApplicationReadObservation {
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .retain_read()
        .expect("the occurrence must remain selectable")
}

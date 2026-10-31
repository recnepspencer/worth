//! Public model attachment uses declared READ, actual heads and full native identity.
mod assessment;
mod attachment;
mod membership;
pub(crate) mod recording;

use worth_query_host::facade::{
    application_entry::{WorthQueryApplicationHistoryDocument, WorthQueryApplicationRequestExt},
    primary_graph::WorthQueryPrimaryGraphApplicationRuntime,
    product::WorthQueryOwnedProductHead,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    document_history::RetentionHistory,
    host::{
        publish_on_first_program, DocumentRetentionRuntime, DocumentRetentionSchema,
        RetentionProgramP0, SEED_RETENTION,
    },
    operator_identity::{authenticate_operator, request_scope},
    retention_entry::{DocumentRetentionRead, DOCUMENT_IDENTITY, RELATED_DOCUMENT_IDENTITY},
};

pub(crate) type App = DocumentRetentionRuntime<RetentionProgramP0>;
pub(crate) type Runtime = WorthQueryPrimaryGraphApplicationRuntime<DocumentRetentionSchema>;
pub(crate) type Document<'a, H = RetentionHistory> =
    WorthQueryApplicationHistoryDocument<'a, DocumentRetentionSchema, H>;

pub(crate) fn head(host: &Runtime) -> WorthQueryOwnedProductHead<'_, DocumentRetentionSchema> {
    head_from(host, host.current_world())
}

fn head_from(
    host: &Runtime,
    source: worth_query_host::facade::product::WorthQueryProductBranch,
) -> WorthQueryOwnedProductHead<'_, DocumentRetentionSchema> {
    host.with_host_advancement(|phase| {
        let selected = host.on_branch(source).select(&phase).unwrap();
        host.branches()
            .fork_expected(selected)
            .unwrap()
            .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
            .create(&phase)
            .unwrap()
            .into_owned_head()
    })
    .unwrap()
}

pub(crate) fn attach<'a>(
    host: &'a Runtime,
    head: WorthQueryOwnedProductHead<'a, DocumentRetentionSchema>,
    identity: &str,
) -> Document<'a> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.with_host_advancement(|phase| {
        host.request(&principal, &scope)
            .on_branch(head.branch())
            .attach_history::<RetentionHistory>(
                &phase,
                head,
                DocumentRetentionRead {
                    identity: identity.to_owned(),
                },
            )
            .unwrap()
    })
    .unwrap()
}

pub(crate) fn value(host: &Runtime, document: &Document<'_>) -> u64 {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    document
        .read(&host.request(&principal, &scope))
        .unwrap()
        .rows()[0]
        .retention_days
}

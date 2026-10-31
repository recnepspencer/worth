//! Public model attachment uses declared READ, actual heads and full native identity.
mod assessment;
mod attachment;
mod membership;
mod recording;

use worth_query_host::facade::{
    application_entry::{WorthQueryApplicationHistoryDocument, WorthQueryApplicationRequestExt},
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

type App = DocumentRetentionRuntime<RetentionProgramP0>;
type Document<'a, H = RetentionHistory> =
    WorthQueryApplicationHistoryDocument<'a, DocumentRetentionSchema, H>;

fn head(host: &App) -> WorthQueryOwnedProductHead<'_, DocumentRetentionSchema> {
    host.with_host_advancement(|phase| {
        let selected = host
            .runtime()
            .on_branch(host.current_world())
            .select(&phase)
            .unwrap();
        host.runtime()
            .branches()
            .fork_expected(selected)
            .unwrap()
            .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
            .create(&phase)
            .unwrap()
            .into_owned_head()
    })
    .unwrap()
}

fn attach<'a>(
    host: &'a App,
    head: WorthQueryOwnedProductHead<'a, DocumentRetentionSchema>,
    identity: &str,
) -> Document<'a> {
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    host.with_host_advancement(|phase| {
        host.runtime()
            .request(&principal, &scope)
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

fn value(host: &App, document: &Document<'_>) -> u64 {
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    document
        .read(&host.runtime().request(&principal, &scope))
        .unwrap()
        .rows()[0]
        .retention_days
}

pub use super::super::{
    retention_entry::DOCUMENT_IDENTITY, schema::DocumentRetentionSchema,
    workflow::governed_read::seed_grant,
};
pub use worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphBootstrap;

pub fn seed(graph: &mut WorthQueryPrimaryGraphBootstrap<DocumentRetentionSchema>) {
    for (grant, action) in [
        (super::super::document_history::retention_policy::WINDOW_GRANT, <super::super::document_history::retention_policy::WindowCapability as super::capability::Role>::ACTION),
        (super::super::document_history::retention_policy::BOOKMARK_GRANT, <super::super::document_history::retention_policy::BookmarkCapability as super::capability::Role>::ACTION),
        (
            super::super::document_history::navigation::GRANT,
            <super::super::document_history::navigation::NavigationCapability as super::capability::Role>::ACTION,
        ),
        (
            "record-history-payload-grant",
            "read-record-history-subject",
        ),
        (
            "record-history-metadata-grant",
            "read-record-history-metadata",
        ),
    ] {
        seed_grant(
            graph,
            grant,
            action,
            "document-row-1",
            DOCUMENT_IDENTITY,
            &["document-retention-operator"],
        );
    }
    super::super::document_history::control_roles::seed(graph);
}

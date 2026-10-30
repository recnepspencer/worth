pub use super::super::{
    retention_entry::DOCUMENT_IDENTITY, schema::DocumentRetentionSchema,
    workflow::governed_read::seed_grant,
};
pub use worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphBootstrap;

pub fn seed(graph: &mut WorthQueryPrimaryGraphBootstrap<DocumentRetentionSchema>) {
    for (grant, action) in [
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
}

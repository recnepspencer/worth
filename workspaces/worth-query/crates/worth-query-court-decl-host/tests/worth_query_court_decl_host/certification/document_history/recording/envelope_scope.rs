//! A history that owns no relation admits envelope stages on its own model only.
use super::*;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryDocumentHistoryDenial, WorthQueryHistoryEditEnvelopeDenial,
        WorthQueryHistoryStepMetadata,
    },
    declaration::application_program::ApplicationNoOutputGraph,
    runtime::ExecutionAllocationPolicy,
};
use worth_relational::facade::inspection::observe_maintenance_work;

fn set_retention(identity: &str, days: u64) -> SetRetentionIntent {
    SetRetentionIntent {
        input: SetRetentionInput {
            identity: identity.to_owned(),
            retention_days: days,
        },
    }
}

/// Stage 0 edits the document's own model. Stage 1 is the same enrolled
/// operation on another model instance, which this history can never record.
#[test]
fn an_envelope_stage_on_another_model_refuses_before_any_commit() {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let root = document.current_step(&request).unwrap();
    let branch = document
        .selection(&request)
        .unwrap()
        .physical_endpoint()
        .product_branch();
    let on_document = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch);
    let (executed, published) = observe_maintenance_work(8, || {
        let mut envelope = document
            .edit_envelope(&host, &on_document, 0x9182_0e01_u64)
            .unwrap_or_else(|failure| panic!("envelope construction: {:?}", failure.denial()));
        envelope
            .push_stage::<ApplicationNoOutputGraph, _>(set_retention(DOCUMENT_IDENTITY, 3))
            .unwrap();
        envelope
            .push_stage::<ApplicationNoOutputGraph, _>(set_retention(RELATED_DOCUMENT_IDENTITY, 3))
            .unwrap();
        envelope.execute_recorded(
            WorthQueryHistoryStepMetadata::from_bytes(Vec::new()),
            ExecutionAllocationPolicy::SystemAllocation,
        )
    });
    let (document, denial) = match executed {
        Err(failure) => failure.into_parts(),
        Ok(_) => panic!("a stage on another model must refuse"),
    };
    assert!(
        matches!(
            denial,
            WorthQueryHistoryEditEnvelopeDenial::Stage {
                ordinal: 1,
                denial: WorthQueryDocumentHistoryDenial::ForeignModel
            }
        ),
        "{denial:?}"
    );
    assert_eq!(
        published.published_rows.len() + published.omitted_publications,
        0,
        "no stage may commit before the refusal"
    );
    assert_eq!(document.current_step(&request).unwrap().id(), root.id());
    assert_eq!(value(&host, &document), 7);
    close(&host, document, 1);
}

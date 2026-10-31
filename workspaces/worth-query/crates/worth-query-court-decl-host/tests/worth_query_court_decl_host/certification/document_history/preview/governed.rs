//! Current metadata authority is required even when the original source remains held.
use super::super::metadata::{governed, grant};
use super::*;
use worth_query_testkit_document_retention::facade::document_retention_model::record_history::capability;
fn input() -> capability::Input {
    capability::Input {
        identity: DOCUMENT_IDENTITY.to_owned(),
    }
}

#[test]
fn metadata_refusal_precedes_foreign_preview_node_lookup_with_a_lawful_twin() {
    let host = publish_on_first_program();
    let public = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let mut document = governed(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let foreign = public.current_step(&request).unwrap().id();
    let observed = document.observe_history_enrollment_for_test();
    let denied = document.preview(&request, foreign).unwrap_err();
    assert!(
        matches!(denied, Failure::Document(Native::Metadata(_))),
        "metadata must precede source lookup: {denied:?}"
    );
    assert_eq!(observed.snapshot().node_probes, 0);
    drop(observed);
    let node = document
        .current_step_governed(
            &request,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
        )
        .unwrap()
        .id();
    let preview = document
        .preview_governed(
            &request,
            node,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
        )
        .unwrap();
    assert_eq!(
        preview
            .read_governed(
                &document,
                &request,
                capability::MetadataCapability::reference(),
                capability::MetadataRead::reference(),
                input()
            )
            .unwrap()
            .rows()[0]
            .retention_days,
        SEED_RETENTION
    );
    preview.close();
    close(&host, document, 1);
    close(&host, public, 1);
}

#[test]
fn a_retained_preview_rechecks_current_metadata_without_rebinding_a_diverged_head() {
    let host = publish_on_first_program();
    let mut document = governed(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let step = document
        .current_step_governed(
            &request,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
        )
        .unwrap();
    let preview = document
        .preview_governed(
            &request,
            step.id(),
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
        )
        .unwrap();
    let observed = document.observe_history_enrollment_for_test();
    assert!(matches!(
        preview.read(&document, &request).unwrap_err(),
        Failure::Document(Native::Metadata(_))
    ));
    assert_eq!(observed.snapshot().node_probes, 0);
    grant(
        &host,
        step.endpoint().product_branch(),
        "revoked",
        0x9182_a020,
    );
    let denied = preview
        .read_governed(
            &document,
            &request,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
        )
        .unwrap_err();
    assert!(matches!(denied, Failure::Document(Native::Metadata(worth_query_host::facade::primary_graph::WorthQueryApplicationHistoryDenial::Authorization(_)))), "old source grants cannot replace current authority: {denied:?}");
    assert_eq!(observed.snapshot().node_probes, 0);
    grant(
        &host,
        step.endpoint().product_branch(),
        "active",
        0x9182_a021,
    );
    assert!(matches!(
        preview
            .read_governed(
                &document,
                &request,
                capability::MetadataCapability::reference(),
                capability::MetadataRead::reference(),
                input()
            )
            .unwrap_err(),
        Failure::Document(Native::HeadDiverged)
    ));
    drop(observed);
    preview.close();
    close(&host, document, 1);
}

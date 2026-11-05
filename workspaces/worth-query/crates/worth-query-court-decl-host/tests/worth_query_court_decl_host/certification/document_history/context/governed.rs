//! Actual revocation and restoration preserve an old holder and require fresh authority.
use super::super::metadata::{governed, grant};
use super::*;
use worth_query_host::facade::application_entry::WorthQueryHistoryEditAdvance as Advance;
use worth_query_testkit_document_retention::facade::document_retention_model::record_history::capability;

fn input() -> capability::Input {
    capability::Input {
        identity: DOCUMENT_IDENTITY.to_owned(),
    }
}

#[test]
fn retained_preview_survives_real_grant_restoration_via_explicit_context_refresh() {
    let host = publish_on_first_program();
    let mut document = governed(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let original = document
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
            original.id(),
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
        )
        .unwrap();
    let observation = document.observe_history_enrollment_for_test();
    grant(
        &host,
        original.endpoint().product_branch(),
        "revoked",
        0x9182_b000,
    );
    for denied in [
        preview
            .read_governed(
                &document,
                &request,
                capability::MetadataCapability::reference(),
                capability::MetadataRead::reference(),
                input(),
            )
            .unwrap_err(),
        document
            .refresh_context_governed(
                &request,
                capability::MetadataCapability::reference(),
                capability::MetadataRead::reference(),
                input(),
                limits(2),
            )
            .unwrap_err(),
    ] {
        assert!(matches!(denied, Failure::Document(Native::Metadata(worth_query_host::facade::primary_graph::WorthQueryApplicationHistoryDenial::Authorization(_)))), "fresh grant denial: {denied:?}");
    }
    assert_eq!(observation.snapshot().node_probes, 0);
    grant(
        &host,
        original.endpoint().product_branch(),
        "active",
        0x9182_b001,
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
    document
        .refresh_context_governed(
            &request,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
            limits(2),
        )
        .unwrap();
    assert_eq!(
        document
            .current_step_governed(
                &request,
                capability::MetadataCapability::reference(),
                capability::MetadataRead::reference(),
                input()
            )
            .unwrap(),
        original
    );
    let result = preview
        .read_governed(
            &document,
            &request,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
        )
        .unwrap();
    assert_eq!(result.rows()[0].retention_days, 7);
    assert_eq!(result.source(), original.endpoint());
    drop(result);
    drop(observation);
    let edit = begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_b002);
    let Advance::Progress(edit) = edit.advance_with_metadata(
        &request,
        capability::MetadataCapability::reference(),
        capability::MetadataRead::reference(),
        input(),
    ) else {
        panic!("fresh current authority admits the next edit");
    };
    let (document, child) = finish(&host, ready(&host, edit), 3);
    assert_eq!(child.parent(), Some(original.id()));
    preview.close();
    close(&host, document, 2);
}

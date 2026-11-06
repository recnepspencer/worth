//! Metadata disclosure, window control and bookmark control remain distinct decisions.
use super::super::metadata::{governed, grant};
use super::*;
use worth_query_host::facade::{
    application_entry::{WorthQueryHistoryContextRefreshLimits, WorthQueryHistoryEditAdvance},
    primary_graph::{
        WorthQueryApplicationHistoryDenial as Metadata,
        WorthQueryApplicationQueryAdmissionDenialKind as QueryKind,
        WorthQueryOperationAuthorizationDenialKind as Authorization,
        WorthQueryOperationAuthorizationRefusalKind as Refusal,
    },
};
use worth_query_testkit_document_retention::facade::document_retention_model::record_history::capability;

fn input() -> capability::Input {
    capability::Input {
        identity: DOCUMENT_IDENTITY.to_owned(),
    }
}
fn governance_required(cause: Failure) {
    let Failure::Document(Native::Metadata(Metadata::Query(cause))) = cause else {
        panic!("metadata admission must precede policy work: {cause:?}");
    };
    assert_eq!(cause.kind(), QueryKind::DisclosureGovernanceRequired);
}
fn metadata_revoked(cause: Failure) {
    let Failure::Document(Native::Metadata(Metadata::Authorization(cause))) = cause else {
        panic!("the actual metadata grant must refuse: {cause:?}");
    };
    assert_eq!(
        cause.kind(),
        Authorization::Refused(Refusal::CapabilityAuthorizationMissing)
    );
    assert_eq!(cause.subject(), "MetadataCapability");
}

#[test]
fn governed_retention_rechecks_metadata_before_independent_role_changes_and_resumes_after_restoration(
) {
    let host = publish_on_first_program();
    let mut document = governed(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let root = document
        .current_step_governed(
            &request,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
        )
        .unwrap();
    let mut first = None;
    for (offset, days) in [3, 5, 6].into_iter().enumerate() {
        let edit = begin(
            &host,
            document,
            DOCUMENT_IDENTITY,
            days,
            0x9182_6700 + offset as u64,
        );
        let WorthQueryHistoryEditAdvance::Progress(edit) = edit.advance_with_metadata(
            &request,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
        ) else {
            panic!("real metadata admits enrollment");
        };
        let (next, step) = finish(&host, ready(&host, edit), days);
        first.get_or_insert(step.id());
        document = next;
    }
    let first = first.unwrap();
    governance_required(document.history_retention_status(&request).unwrap_err());
    governance_required(
        document
            .set_history_bookmark::<Bookmark>(&request, root.id(), true)
            .unwrap_err(),
    );
    let target = ApplicationHistoryRecentSteps::new(NonZeroUsize::MIN);
    governance_required(
        document
            .set_history_window(target)
            .advance::<Window>(&request)
            .unwrap_err(),
    );
    let wrong = document
        .history_retention_status_governed(
            &request,
            capability::PayloadCapability::reference(),
            capability::PayloadRead::reference(),
            input(),
        )
        .unwrap_err();
    let Failure::Document(Native::Metadata(Metadata::Query(wrong))) = wrong else {
        panic!("a granted payload purpose is not metadata authority");
    };
    assert_eq!(wrong.kind(), QueryKind::DisclosureAuthorizationMismatch);
    let before = document
        .history_retention_status_governed(
            &request,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
        )
        .unwrap();
    assert_eq!(
        (
            before.recent_steps.get(),
            before.bookmarks,
            before.unprotected_steps,
            before.pending_window
        ),
        (200, 0, 3, None)
    );
    let pinned = document
        .set_history_bookmark_governed::<Bookmark, _, _, _>(
            &request,
            root.id(),
            true,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
        )
        .unwrap();
    assert_eq!(pinned.bookmarks, 1);
    let changed = document
        .set_history_window(target)
        .advance_governed::<Window, _, _, _>(
            &request,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
        )
        .unwrap();
    assert_eq!(
        (
            changed.recent_steps,
            changed.pending_window,
            changed.unprotected_steps,
            changed.bookmarks
        ),
        (target, None, 1, 1)
    );
    let preview = document
        .preview_governed(
            &request,
            root.id(),
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
        7
    );
    let denied = document
        .preview_governed(
            &request,
            first,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
        )
        .unwrap_err();
    assert!(
        matches!(denied, Failure::Document(Native::HistoryNodeExcluded)),
        "{denied:?}"
    );
    document
        .set_history_bookmark_governed::<Bookmark, _, _, _>(
            &request,
            root.id(),
            false,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
        )
        .unwrap();
    preview.close();

    let branch = document
        .selection_governed(
            &request,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
        )
        .unwrap()
        .physical_endpoint()
        .product_branch();
    grant(&host, branch, "revoked", 0x9182_6710);
    metadata_revoked(
        document
            .history_retention_status_governed(
                &request,
                capability::MetadataCapability::reference(),
                capability::MetadataRead::reference(),
                input(),
            )
            .unwrap_err(),
    );
    metadata_revoked(
        document
            .set_history_bookmark_governed::<Bookmark, _, _, _>(
                &request,
                first,
                true,
                capability::MetadataCapability::reference(),
                capability::MetadataRead::reference(),
                input(),
            )
            .unwrap_err(),
    );
    let larger = ApplicationHistoryRecentSteps::new(NonZeroUsize::new(2).unwrap());
    metadata_revoked(
        document
            .set_history_window(larger)
            .advance_governed::<Window, _, _, _>(
                &request,
                capability::MetadataCapability::reference(),
                capability::MetadataRead::reference(),
                input(),
            )
            .unwrap_err(),
    );
    let rows = host
        .request(&principal, &scope)
        .on_branch(branch)
        .query(DocumentRetentionRead {
            identity: DOCUMENT_IDENTITY.to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(rows.rows()[0].retention_days, 6);
    drop(rows);
    grant(&host, branch, "active", 0x9182_6711);
    document
        .refresh_context_governed(
            &request,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
            WorthQueryHistoryContextRefreshLimits {
                maximum_commits: NonZeroUsize::new(4).unwrap(),
                maximum_records: NonZeroUsize::new(256).unwrap(),
                maximum_evidence_bytes: std::num::NonZeroU64::new(8 * 1024 * 1024).unwrap(),
            },
        )
        .unwrap();
    let unchanged = document
        .history_retention_status_governed(
            &request,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
        )
        .unwrap();
    assert_eq!(
        (
            unchanged.recent_steps,
            unchanged.bookmarks,
            unchanged.unprotected_steps
        ),
        (target, 0, 1)
    );
    let grown = document
        .set_history_window(larger)
        .advance_governed::<Window, _, _, _>(
            &request,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
        )
        .unwrap();
    assert_eq!((grown.recent_steps, grown.pending_window), (larger, None));
    let denied = document
        .preview_governed(
            &request,
            first,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
        )
        .unwrap_err();
    assert!(
        matches!(denied, Failure::Document(Native::HistoryNodeExcluded)),
        "growing must not reconstruct excluded sources: {denied:?}"
    );
    close(&host, document, 4);
}

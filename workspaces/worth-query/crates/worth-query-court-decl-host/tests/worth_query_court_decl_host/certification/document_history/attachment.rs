use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationHistoryCloseFailure, WorthQueryApplicationHistoryDenial,
    WorthQueryDocumentHistoryDenial,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    presented_request::set_retention,
    readback::{observe_head, read_retention},
    settled_verdict::{settle, RetentionVerdict},
};

#[test]
fn attached_documents_read_their_actual_independent_model_heads_and_refuse_divergence() {
    let host = publish_on_first_program();
    let first = head(&host);
    let first_branch = first.branch();
    let second = head(&host);
    let second_branch = second.branch();
    assert_eq!(
        settle(set_retention(&host, first_branch, 3, 0x9182_0200)),
        RetentionVerdict::Performed(3)
    );
    let document_a = attach(&host, first, DOCUMENT_IDENTITY);
    let document_b = attach(&host, second, RELATED_DOCUMENT_IDENTITY);
    assert_eq!(value(&host, &document_a), 3);
    assert_eq!(value(&host, &document_b), SEED_RETENTION);
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    assert_eq!(
        document_a
            .read(
                &host
                    .runtime()
                    .request(&principal, &scope)
                    .on_branch(second_branch)
            )
            .unwrap()
            .rows()[0]
            .retention_days,
        3
    );
    assert_eq!(
        settle(set_retention(&host, first_branch, 7, 0x9182_0201)),
        RetentionVerdict::Performed(7)
    );
    assert_eq!(read_retention(host.runtime(), first_branch), 7);
    assert!(matches!(
        document_a.read(&host.runtime().request(&principal, &scope)),
        Err(WorthQueryApplicationHistoryDenial::Document(
            WorthQueryDocumentHistoryDenial::HeadDiverged
        ))
    ));
    assert_eq!(
        settle(set_retention(&host, first_branch, 3, 0x9182_0203)),
        RetentionVerdict::Performed(3)
    );
    assert!(matches!(
        document_a.read(&host.runtime().request(&principal, &scope)),
        Err(WorthQueryApplicationHistoryDenial::Document(
            WorthQueryDocumentHistoryDenial::HeadDiverged
        ))
    ));
    assert_eq!(value(&host, &document_b), SEED_RETENTION);
    assert_eq!(
        read_retention(host.runtime(), host.current_world()),
        SEED_RETENTION
    );
    host.with_host_advancement(|phase| document_a.close(&phase).unwrap())
        .unwrap();
    host.with_host_advancement(|phase| document_b.close(&phase).unwrap())
        .unwrap();
    assert!(host.runtime().branches().pending_cleanup().is_empty());
}

#[test]
fn attached_document_foreign_read_and_close_preserve_its_original_live_custody() {
    let host = publish_on_first_program();
    let foreign = publish_on_first_program();
    let original = head(&host);
    let branch = original.branch();
    let scope = request_scope();
    let principal = authenticate_operator(foreign.runtime().installed_schema(), &scope);
    let failure = foreign
        .with_host_advancement(|phase| {
            foreign
                .runtime()
                .request(&principal, &scope)
                .on_branch(branch)
                .attach_history::<RetentionHistory>(
                    &phase,
                    original,
                    DocumentRetentionRead {
                        identity: DOCUMENT_IDENTITY.to_owned(),
                    },
                )
                .unwrap_err()
        })
        .unwrap();
    assert!(matches!(failure.denial(), WorthQueryApplicationHistoryDenial::Query(
        worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial::ProductSelection(_))));
    let (original, _) = failure.into_parts();
    assert_eq!(original.branch(), branch);
    let document = attach(&host, original, DOCUMENT_IDENTITY);
    assert_eq!(value(&host, &document), SEED_RETENTION);
    let cancellation = worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource::new();
    let stopped_scope =
        worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope::new(
            None,
            cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let stopped_principal =
        authenticate_operator(host.runtime().installed_schema(), &stopped_scope);
    cancellation.cancel();
    assert!(matches!(document.read(&host.runtime().request(&stopped_principal, &stopped_scope)),
        Err(WorthQueryApplicationHistoryDenial::ExecutionRequest(
            worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial::Interrupted(
                worth_query_host::facade::application_contribution::WorthQueryManagedComputationInterruption::Cancelled)))));
    assert_eq!(value(&host, &document), SEED_RETENTION);
    assert!(matches!(
        document.read(&foreign.runtime().request(&principal, &scope)),
        Err(WorthQueryApplicationHistoryDenial::ForeignRuntime)
    ));
    let failure = foreign
        .with_host_advancement(|phase| document.close(&phase).unwrap_err())
        .unwrap();
    let WorthQueryApplicationHistoryCloseFailure::Retained { document, denial } = failure else {
        panic!("a foreign phase cannot retire the document")
    };
    assert!(matches!(denial, worth_query_host::facade::product::WorthQueryApplicationProductBranchCloseDenial::ForeignPhase));
    assert_eq!(value(&host, &document), SEED_RETENTION);
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
    assert!(host.runtime().branches().pending_cleanup().is_empty());
}

#[test]
fn attached_document_close_transfers_actual_retired_cleanup_until_its_reader_releases() {
    let host = publish_on_first_program();
    let head = head(&host);
    let branch = head.branch();
    assert_eq!(
        settle(set_retention(&host, branch, 3, 0x9182_0202)),
        RetentionVerdict::Performed(3)
    );
    let document = attach(&host, head, DOCUMENT_IDENTITY);
    assert_eq!(value(&host, &document), 3);
    let retained = observe_head(host.runtime(), branch);
    let failure = host
        .with_host_advancement(|phase| document.close(&phase).unwrap_err())
        .unwrap();
    let WorthQueryApplicationHistoryCloseFailure::Retired {
        branch: retired,
        failure,
    } = failure
    else {
        panic!("a held reader retains actual retired cleanup")
    };
    assert_eq!(retired, branch);
    assert!(matches!(failure.denial(), worth_query_host::facade::product::WorthQueryApplicationProductBranchCleanupDenial::Product(
        worth_query_host::facade::product::WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryStillRetained)));
    let blocked = failure.into_cleanup().retry().unwrap_err();
    assert!(matches!(blocked.denial(), worth_query_host::facade::product::WorthQueryApplicationProductBranchCleanupDenial::Product(
        worth_query_host::facade::product::WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryStillRetained)));
    drop(retained);
    blocked.into_cleanup().retry().unwrap();
    assert!(host.runtime().branches().pending_cleanup().is_empty());
}

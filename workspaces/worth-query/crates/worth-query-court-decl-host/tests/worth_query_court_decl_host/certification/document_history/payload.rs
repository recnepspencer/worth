//! Retained truth grants no payload authority; current READ and metadata stay independent.
use super::*;
use std::num::{NonZeroU64, NonZeroUsize};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationHistoryDenial as Failure,
    WorthQueryApplicationRequestQueryDenial as QueryFailure,
    WorthQueryDocumentHistoryDenial as Native, WorthQueryGovernedHistoryPayload,
    WorthQueryHistoryContextRefreshLimits, WorthQueryHistoryEdit,
    WorthQueryHistoryEditAdvance as Advance, WorthQueryHistoryEditDenial as EditDenial,
    WorthQueryHistoryEditOutcome, WorthQueryHistoryPageLimits, WorthQueryHistoryStep,
    WorthQuerySourceCompletion,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    document_history::GovernedPayloadRetentionHistory as History,
    record_history::{capability, query},
};
mod authoring;
use authoring::{begin, finish, ready};

type Payload = WorthQueryGovernedHistoryPayload<
    DocumentRetentionSchema,
    capability::PayloadCapability,
    capability::PayloadRead,
    capability::Input,
>;
type GovernedDocument<'a> =
    WorthQueryApplicationHistoryDocument<'a, DocumentRetentionSchema, History, Payload>;
fn input() -> capability::Input {
    capability::Input {
        identity: DOCUMENT_IDENTITY.to_owned(),
    }
}
fn access() -> query::Input {
    query::Input {
        identity: DOCUMENT_IDENTITY.to_owned(),
    }
}
fn page_limits() -> WorthQueryHistoryPageLimits {
    WorthQueryHistoryPageLimits {
        maximum_entries: NonZeroUsize::new(8).unwrap(),
        maximum_examined_entries: NonZeroUsize::new(8).unwrap(),
        maximum_owned_bytes: NonZeroUsize::new(16 * 1024).unwrap(),
    }
}
fn refresh_limits() -> WorthQueryHistoryContextRefreshLimits {
    WorthQueryHistoryContextRefreshLimits {
        maximum_commits: NonZeroUsize::new(2).unwrap(),
        maximum_records: NonZeroUsize::new(256).unwrap(),
        maximum_evidence_bytes: NonZeroU64::new(8 * 1024 * 1024).unwrap(),
    }
}
fn payload_denied(cause: Failure) {
    assert!(
        matches!(cause, Failure::Query(QueryFailure::CapabilityAdmission(_))),
        "actual payload capability must refuse: {cause:?}"
    );
}

#[test]
fn payload_revocation_after_paging_blocks_detail_and_old_holder_until_real_restoration() {
    let host = publish_on_first_program();
    let owned = head(&host);
    let branch = owned.branch();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    // A lawful metadata purpose cannot substitute for payload governance at attachment.
    let failure = host
        .with_host_advancement(|phase| {
            host.request(&principal, &scope)
                .on_branch(branch)
                .attach_history_with_payload::<History, _, _, _>(
                    &phase,
                    owned,
                    access(),
                    capability::MetadataCapability::reference(),
                    capability::MetadataRead::reference(),
                    input(),
                )
                .unwrap_err()
        })
        .unwrap();
    let (owned, cause) = failure.into_parts();
    assert!(
        matches!(cause, Failure::Query(QueryFailure::Admission(_))),
        "wrong purpose: {cause:?}"
    );
    assert_eq!(owned.branch(), branch);
    let document = host
        .with_host_advancement(|phase| {
            host.request(&principal, &scope)
                .on_branch(branch)
                .attach_history_with_payload::<History, _, _, _>(
                    &phase,
                    owned,
                    access(),
                    capability::PayloadCapability::reference(),
                    capability::PayloadRead::reference(),
                    input(),
                )
                .unwrap()
        })
        .unwrap();
    assert_eq!(
        document.read(&request).unwrap().rows()[0].retention,
        SEED_RETENTION
    );
    assert!(matches!(
        document.current_step(&request).unwrap_err(),
        Failure::Document(Native::Metadata(_))
    ));
    let (document, old) = finish(
        &host,
        ready(&host, begin(&host, document, 3, 0x9182_c000)),
        3,
    );
    let (mut document, selected) = finish(
        &host,
        ready(&host, begin(&host, document, 5, 0x9182_c001)),
        5,
    );
    let page = document
        .history_page_governed(
            &request,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
            page_limits(),
            None,
        )
        .unwrap();
    assert_eq!(page.rows().len(), 3);
    assert_page_entry(page.rows()[1].step(), &old);
    assert_eq!(page.current_step_id(), selected.id());
    let preview = document
        .preview_governed(
            &request,
            old.id(),
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
        )
        .unwrap();
    let before = preview
        .read_governed(
            &document,
            &request,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
        )
        .unwrap();
    assert_eq!(before.rows()[0].retention, 3);
    assert_eq!(before.source(), old.endpoint());
    drop(before);
    let observed = document.observe_history_enrollment_for_test();
    super::metadata::grant_named(
        &host,
        selected.endpoint().product_branch(),
        "record-history-payload-grant",
        "revoked",
        0x9182_c002,
    );
    // Metadata remains independently lawful on the same current branch.
    host.with_host_advancement(|phase| {
        let current = host.on_branch(selected.endpoint().product_branch()).select(&phase).unwrap();
        let binding = host.installed_schema().installed_query_binding::<query::Binding>().unwrap();
        let current_principal = current.resolve_authenticated_principal(&phase, binding.principal_binding(), &principal, &scope, worth_query_host::facade::primary_graph::WorthQueryPrincipalResolutionMode::Ordinary).unwrap();
        let metadata = host.installed_schema().capability(capability::MetadataCapability::reference(), capability::MetadataRead::reference()).unwrap();
        current.admit_capability_access(&phase, &current_principal, &metadata, input(), &scope).unwrap();
    }).unwrap();
    for cause in [
        document.read(&request).err().expect("revoked payload read"),
        document
            .current_step_governed(
                &request,
                capability::MetadataCapability::reference(),
                capability::MetadataRead::reference(),
                input(),
            )
            .unwrap_err(),
        document
            .history_page_governed(
                &request,
                capability::MetadataCapability::reference(),
                capability::MetadataRead::reference(),
                input(),
                page_limits(),
                None,
            )
            .unwrap_err(),
        document
            .redo_choices_governed(
                &request,
                capability::MetadataCapability::reference(),
                capability::MetadataRead::reference(),
                input(),
                page_limits(),
                None,
            )
            .unwrap_err(),
        document
            .preview_governed(
                &request,
                old.id(),
                capability::MetadataCapability::reference(),
                capability::MetadataRead::reference(),
                input(),
            )
            .unwrap_err(),
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
                refresh_limits(),
            )
            .unwrap_err(),
    ] {
        payload_denied(cause);
    }
    assert_eq!(observed.snapshot().node_probes, 0);
    assert_eq!(observed.snapshot().child_index_probes, 0);
    let edit = begin(&host, document, 9, 0x9182_c003);
    let Advance::Refused {
        edit,
        denial: EditDenial::History(cause),
    } = edit.advance_with_metadata(
        &request,
        capability::MetadataCapability::reference(),
        capability::MetadataRead::reference(),
        input(),
    )
    else {
        panic!("internal READ must recheck payload authority")
    };
    payload_denied(cause);
    assert_eq!(observed.snapshot().node_probes, 0);
    document = edit.discard(&request).unwrap();
    super::metadata::grant_named(
        &host,
        selected.endpoint().product_branch(),
        "record-history-payload-grant",
        "active",
        0x9182_c004,
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
            refresh_limits(),
        )
        .unwrap();
    let old_read = preview
        .read_governed(
            &document,
            &request,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
        )
        .unwrap();
    assert_eq!(old_read.rows()[0].retention, 3);
    assert_eq!(old_read.source(), old.endpoint());
    assert_eq!(
        document
            .redo_choices_governed(
                &request,
                capability::MetadataCapability::reference(),
                capability::MetadataRead::reference(),
                input(),
                page_limits(),
                None,
            )
            .unwrap()
            .choice(),
        worth_query_host::facade::application_entry::WorthQueryHistoryRedoChoice::NoChild
    );
    assert_eq!(document.read(&request).unwrap().rows()[0].retention, 5);
    assert_eq!(
        document
            .current_step_governed(
                &request,
                capability::MetadataCapability::reference(),
                capability::MetadataRead::reference(),
                input()
            )
            .unwrap(),
        selected
    );
    assert_page_entry(page.rows()[1].step(), &old);
    drop(old_read);
    drop(observed);
    preview.close();
    let receipt = host
        .with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
    assert_eq!(receipt.closed_heads(), 3);
    assert!(host.branches().pending_cleanup().is_empty());
}

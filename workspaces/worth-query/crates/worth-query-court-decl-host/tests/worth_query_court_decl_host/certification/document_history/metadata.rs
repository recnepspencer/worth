//! Real metadata grants do not follow from payload READ, write authority or cached attempts.
use super::recording::{begin, close, finish, ready};
use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationHistoryDenial as HistoryDenial,
    WorthQueryDocumentHistoryDenial as DocumentDenial, WorthQueryHistoryEditAdvance as Advance,
    WorthQueryHistoryEditDenial as EditDenial, WorthQueryHistoryEditOutcome as Outcome,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationHistoryDenial as MetadataDenial,
    WorthQueryApplicationQueryAdmissionDenialKind as QueryDenial,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    document_history::GovernedRetentionHistory as History, record_history::capability,
};

pub(super) fn governed(host: &App) -> Document<'_, History> {
    let owned = head(host);
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    host.with_host_advancement(|phase| {
        host.runtime()
            .request(&principal, &scope)
            .on_branch(owned.branch())
            .attach_history::<History>(
                &phase,
                owned,
                DocumentRetentionRead {
                    identity: DOCUMENT_IDENTITY.to_owned(),
                },
            )
            .unwrap()
    })
    .unwrap()
}

fn input() -> capability::Input {
    capability::Input {
        identity: DOCUMENT_IDENTITY.to_owned(),
    }
}

fn governance_required(cause: HistoryDenial) {
    let HistoryDenial::Document(DocumentDenial::Metadata(MetadataDenial::Query(cause))) = cause
    else {
        panic!("metadata decision must refuse before step lookup: {cause:?}");
    };
    assert_eq!(cause.kind(), QueryDenial::DisclosureGovernanceRequired);
}

#[test]
fn selected_metadata_and_cached_attempts_require_their_own_current_capability() {
    let host = publish_on_first_program();
    let mut document = governed(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    assert_eq!(document.read(&request).unwrap().rows()[0].retention_days, 7);
    let observation = document.observe_history_enrollment_for_test();
    governance_required(document.current_step(&request).unwrap_err());
    let wrong = document
        .current_step_governed(
            &request,
            capability::PayloadCapability::reference(),
            capability::PayloadRead::reference(),
            input(),
        )
        .unwrap_err();
    let HistoryDenial::Document(DocumentDenial::Metadata(MetadataDenial::Query(wrong))) = wrong
    else {
        panic!("the real granted payload purpose cannot govern metadata: {wrong:?}");
    };
    assert_eq!(wrong.kind(), QueryDenial::DisclosureAuthorizationMismatch);
    assert_eq!(observation.snapshot().node_probes, 0);
    drop(observation);
    let root = document
        .current_step_governed(
            &request,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
        )
        .unwrap();
    let edit = begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_8100);
    let Advance::Progress(edit) = edit.advance_with_metadata(
        &request,
        capability::MetadataCapability::reference(),
        capability::MetadataRead::reference(),
        input(),
    ) else {
        panic!("fresh genuine metadata grant admits ordinary enrollment");
    };
    let (mut document, step) = finish(&host, ready(&host, edit), 3);
    assert_eq!(step.parent(), Some(root.id()));
    let observation = document.observe_history_enrollment_for_test();
    governance_required(document.current_step(&request).unwrap_err());
    let retry = begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_8100);
    let Advance::Refused {
        edit: retry,
        denial: EditDenial::History(cause),
    } = retry.advance(&request)
    else {
        panic!("write authority cannot expose cached step metadata");
    };
    governance_required(cause);
    assert_eq!(observation.snapshot().attempt_probes, 0);
    assert_eq!(observation.snapshot().node_probes, 0);
    drop(observation);
    assert!(retry.selected_step().is_none());
    assert_eq!(retry.read(&request).unwrap().rows()[0].retention_days, 3);
    let Advance::Ready(retry) = retry.advance_with_metadata(
        &request,
        capability::MetadataCapability::reference(),
        capability::MetadataRead::reference(),
        input(),
    ) else {
        panic!("same original owner can retry under the real metadata grant");
    };
    let done = retry.finish(&request).unwrap();
    let Outcome::AlreadyRecorded { step: original, .. } = done.outcome else {
        panic!("original completed occurrence");
    };
    assert_eq!(original.id(), step.id());
    assert_eq!(
        done.document
            .current_step_governed(
                &request,
                capability::MetadataCapability::reference(),
                capability::MetadataRead::reference(),
                input()
            )
            .unwrap()
            .id(),
        step.id()
    );
    let branch = step.endpoint().product_branch();
    grant(&host, branch, "revoked", 0x9182_8101);
    let denied = done
        .document
        .current_step_governed(
            &request,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
        )
        .unwrap_err();
    assert!(
        matches!(
            denied,
            HistoryDenial::Document(DocumentDenial::Metadata(MetadataDenial::Authorization(_)))
        ),
        "actual metadata grant revocation must refuse: {denied:?}"
    );
    // Payload READ itself remains lawful; the attached head reports divergence
    // because this control write was made outside its managed edit owner.
    assert_eq!(
        host.runtime()
            .request(&principal, &scope)
            .on_branch(branch)
            .query(DocumentRetentionRead {
                identity: DOCUMENT_IDENTITY.to_owned()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .retention_days,
        3
    );
    grant(&host, branch, "active", 0x9182_8102);
    let restored_head = head_from(&host, branch);
    let restored = host
        .with_host_advancement(|phase| {
            host.runtime()
                .request(&principal, &scope)
                .on_branch(restored_head.branch())
                .attach_history::<History>(
                    &phase,
                    restored_head,
                    DocumentRetentionRead {
                        identity: DOCUMENT_IDENTITY.to_owned(),
                    },
                )
                .unwrap()
        })
        .unwrap();
    assert!(restored
        .current_step_governed(
            &request,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input()
        )
        .is_ok());
    close(&host, restored, 1);
    close(&host, done.document, 2);
}

fn grant(
    host: &App,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
    status: &str,
    key: u64,
) {
    use worth_query_testkit_document_retention::facade::document_retention_model::workflow::{
        WorkflowGrantStatusInput, WorkflowGrantStatusIntent,
    };
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let outcome = host
        .with_application_advancement(&scope, |phase| {
            host.runtime()
                .request(&principal, &scope)
                .on_branch(branch)
                .advanced()
                .mutate(WorkflowGrantStatusIntent {
                    input: WorkflowGrantStatusInput {
                        grant_identity: "record-history-metadata-grant".to_owned(),
                        status: status.to_owned(),
                    },
                })
                .without_source()
                .idempotency(&key)
                .execute_in_program_in_advancement(&phase, host)
        })
        .unwrap()
        .unwrap();
    assert!(matches!(outcome, worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::Committed { .. }));
}

#[test]
fn metadata_disclosure_cannot_skip_the_original_read_body_and_filters() {
    use worth_query_testkit_document_retention::facade::document_retention_model::{
        document_history::matching_read, record_history::stimuli,
    };
    let host = publish_on_first_program();
    let owned = head(&host);
    let branch = owned.branch();
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let mut document = host
        .with_host_advancement(|phase| {
            host.runtime()
                .request(&principal, &scope)
                .on_branch(branch)
                .attach_history::<matching_read::History>(
                    &phase,
                    owned,
                    matching_read::Input {
                        identity: DOCUMENT_IDENTITY.to_owned(),
                    },
                )
                .unwrap()
        })
        .unwrap();
    assert_eq!(document.read(&request).unwrap().rows(), &[true]);
    let root = document
        .current_step_governed(
            &request,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
        )
        .unwrap();
    assert_eq!(root.parent(), None);
    drop(stimuli::retention(&host, branch, 3, 0x9182_8200, false));
    assert!(
        host.runtime()
            .request(&principal, &scope)
            .on_branch(branch)
            .query(matching_read::Input {
                identity: DOCUMENT_IDENTITY.to_owned()
            })
            .execute()
            .is_err(),
        "the original ExactlyOne filter must refuse at3"
    );
    let observation = document.observe_history_enrollment_for_test();
    let refused = document
        .current_step_governed(
            &request,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
        )
        .unwrap_err();
    assert!(matches!(refused, HistoryDenial::Document(DocumentDenial::Read(_))), "ordinary body denial must precede metadata lookup and head-divergence classification: {refused:?}");
    assert_eq!(observation.snapshot().node_probes, 0);
    drop(observation);
    close(&host, document, 1);
}

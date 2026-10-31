use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationHistoryDenial as HistoryDenial, WorthQueryApplicationMutationOutcome,
    WorthQueryDocumentHistoryDenial as NativeDenial, WorthQueryHistoryEditDenial,
};

#[test]
fn changed_input_refuses_before_effects_and_preserves_the_selected_document() {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let (document, step) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_0410),
        ),
        3,
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let Advance::Refused { edit, denial } =
        begin(&host, document, DOCUMENT_IDENTITY, 7, 0x9182_0410).advance(&request)
    else {
        panic!("same key with changed input refuses")
    };
    assert!(
        matches!(
            denial,
            WorthQueryHistoryEditDenial::History(HistoryDenial::Document(
                NativeDenial::IntentDrift
            ))
        ),
        "{denial:?}"
    );
    assert_eq!(edit.posture(), Posture::AwaitingAdmission);
    assert_eq!(edit.read(&request).unwrap().rows()[0].retention_days, 3);
    let document = edit.discard(&request).unwrap();
    assert_eq!(document.current_step(&request).unwrap().id(), step.id());
    close(&host, document, 2);
}

#[test]
fn wrong_model_and_foreign_capture_preserve_the_original_document_for_a_lawful_twin() {
    let host = publish_on_first_program();
    let foreign = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let root = document.current_step(&request).unwrap();
    let Advance::Refused { edit, denial } =
        begin(&host, document, RELATED_DOCUMENT_IDENTITY, 3, 0x9182_0411).advance(&request)
    else {
        panic!("a different model instance cannot enroll")
    };
    assert!(
        matches!(
            denial,
            WorthQueryHistoryEditDenial::History(HistoryDenial::Document(
                NativeDenial::ForeignModel
            ))
        ),
        "{denial:?}"
    );
    assert_eq!(edit.read(&request).unwrap().rows()[0].retention_days, 7);
    let document = edit.discard(&request).unwrap();
    assert_eq!(document.current_step(&request).unwrap().id(), root.id());
    let foreign_scope = request_scope();
    let foreign_principal =
        authenticate_operator(foreign.runtime().installed_schema(), &foreign_scope);
    let failure = document
        .edit(SetRetentionIntent {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: 3,
            },
        })
        .idempotency_owned(
            &foreign
                .runtime()
                .request(&foreign_principal, &foreign_scope),
            0x9182_0412,
        );
    let failure = match failure {
        Err(failure) => failure,
        Ok(_) => panic!("foreign capture cannot retarget the document"),
    };
    let (builder, key, denial) = failure.into_parts();
    assert!(matches!(denial, worth_query_host::facade::application_entry::WorthQueryHistoryEditCaptureDenial::ForeignRuntime));
    let edit = builder
        .idempotency_owned(&request, key)
        .unwrap()
        .retain_in_program::<RetentionProgramP0, Action>(&host)
        .unwrap_or_else(|(_, cause)| panic!("lawful action: {cause:?}"));
    let (document, step) = finish(&host, ready(&host, edit), 3);
    assert_eq!(step.parent(), Some(root.id()));
    close(&host, document, 2);
}

#[test]
fn domain_denial_keeps_its_native_outcome_and_does_not_record_a_user_step() {
    use worth_query_testkit_document_retention::facade::document_retention_model::retention_entry::SetRetentionDenial;
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let root = document.current_step(&request).unwrap();
    let edit = ready(
        &host,
        begin(&host, document, DOCUMENT_IDENTITY, 7, 0x9182_0413),
    );
    assert_eq!(edit.posture(), Posture::Terminal);
    assert!(matches!(
        edit.terminal_source().unwrap().native_outcome(),
        Some(WorthQueryApplicationMutationOutcome::DomainDenied(
            SetRetentionDenial::RetentionUnchanged
        ))
    ));
    let finished = edit.finish(&request).unwrap();
    let Outcome::NotRecorded { terminal, .. } = finished.outcome else {
        panic!("domain denial remains native evidence")
    };
    assert!(matches!(
        terminal.native_outcome(),
        Some(WorthQueryApplicationMutationOutcome::DomainDenied(
            SetRetentionDenial::RetentionUnchanged
        ))
    ));
    drop(terminal);
    assert_eq!(
        finished.document.current_step(&request).unwrap().id(),
        root.id()
    );
    assert_eq!(value(&host, &finished.document), 7);
    let (document, step) = finish(
        &host,
        ready(
            &host,
            begin(&host, finished.document, DOCUMENT_IDENTITY, 3, 0x9182_0413),
        ),
        3,
    );
    assert_eq!(step.parent(), Some(root.id()));
    close(&host, document, 2);
}

#[test]
fn a_changed_candidate_receipt_never_silently_becomes_a_recorded_step() {
    use worth_query_testkit_document_retention::facade::document_retention_model::{
        presented_request::set_retention,
        settled_verdict::{settle, RetentionVerdict},
    };
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let root = document.current_step(&request).unwrap();
    let mut edit = begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_0414);
    for _ in 0..3 {
        let Advance::Progress(next) = edit.advance(&request) else {
            panic!("ordinary private progression")
        };
        edit = next;
    }
    let candidate = match edit.completed_source().unwrap().evidence() {
        WorthQuerySourceCompletion::Fresh { receipt, .. } => receipt.product_branch(),
        _ => panic!("original fresh completion"),
    };
    assert_eq!(
        settle(set_retention(&host, candidate, 7, 0x9182_0415)),
        RetentionVerdict::Performed(7)
    );
    let Advance::Refused { edit, denial } = edit.advance(&request) else {
        panic!("a different candidate head cannot satisfy the original receipt")
    };
    assert!(
        matches!(
            denial,
            WorthQueryHistoryEditDenial::History(HistoryDenial::Document(
                NativeDenial::ReceiptMismatch
            ))
        ),
        "{denial:?}"
    );
    assert_eq!(edit.posture(), Posture::AwaitingSelection);
    assert_eq!(edit.read(&request).unwrap().rows()[0].retention_days, 7);
    assert!(
        matches!(edit.completed_source().unwrap().evidence(), WorthQuerySourceCompletion::Fresh { result, .. } if result.retention_days == 3)
    );
    let document = edit.discard(&request).unwrap();
    assert_eq!(document.current_step(&request).unwrap().id(), root.id());
    close(&host, document, 1);
}

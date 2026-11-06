use super::*;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationHistoryDenial, WorthQueryDocumentHistoryDenial,
        WorthQueryHistoryEditDenial,
    },
    declaration::application_schema::{ApplicationEncodedScalarValue, TypedMutationPreconditions},
};
use worth_query_testkit_document_retention::facade::document_retention_model::schema::DocumentRetentionField;

fn conditioned<'a>(host: &App, document: Document<'a>, expected: u64) -> Edit<'a> {
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    document
        .edit(SetRetentionIntent {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: 3,
            },
        })
        .preconditions(TypedMutationPreconditions::new().expect_fact(
            DocumentRetentionField::reference(),
            ApplicationEncodedScalarValue::try_new(expected).unwrap(),
        ))
        .idempotency_owned(&host.runtime().request(&principal, &scope), 0x9182_0450)
        .unwrap()
        .retain_in_program::<RetentionProgramP0, Action>(host)
        .unwrap_or_else(|(_, cause)| panic!("declared action: {cause:?}"))
}

#[test]
fn recorded_retry_binds_original_typed_preconditions_without_rechecking_the_old_fact() {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let (document, original) = finish(&host, ready(&host, conditioned(&host, document, 7)), 3);
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let Advance::Ready(edit) = conditioned(&host, document, 7).advance(&request) else {
        panic!("equivalent retry returns the original step although current retention is now 3")
    };
    assert_eq!(edit.posture(), Posture::AlreadyRecorded);
    let finished = edit.finish(&request).unwrap();
    assert!(
        matches!(finished.outcome, Outcome::AlreadyRecorded { step, .. } if step.id() == original.id())
    );
    let Advance::Refused { edit, denial } =
        conditioned(&host, finished.document, 3).advance(&request)
    else {
        panic!("changing a declared expected fact changes the original intent")
    };
    assert!(
        matches!(
            denial,
            WorthQueryHistoryEditDenial::History(WorthQueryApplicationHistoryDenial::Document(
                WorthQueryDocumentHistoryDenial::IntentDrift
            ))
        ),
        "{denial:?}"
    );
    assert_eq!(edit.posture(), Posture::AwaitingAdmission);
    assert_eq!(edit.read(&request).unwrap().rows()[0].retention_days, 3);
    let document = edit.discard(&request).unwrap();
    assert_eq!(document.current_step(&request).unwrap().id(), original.id());
    close(&host, document, 2);
}

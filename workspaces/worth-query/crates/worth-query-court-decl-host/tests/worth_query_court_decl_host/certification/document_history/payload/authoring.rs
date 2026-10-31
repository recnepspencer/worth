//! Use the original retention action with the document's governed payload recipe.
use super::*;
use worth_query_host::facade::declaration::application_program::{
    ApplicationActionReference, ApplicationRootComposition,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    programs::{completion::RetentionCompletion, DocumentRetentionFeature},
    retention_entry::{SetRetentionBinding, SetRetentionInput, SetRetentionIntent},
};

type Action = ApplicationActionReference<
    DocumentRetentionSchema,
    ApplicationRootComposition,
    DocumentRetentionFeature,
    SetRetentionBinding,
    RetentionCompletion,
>;
pub(super) type Edit<'a> = WorthQueryHistoryEdit<
    'a,
    DocumentRetentionSchema,
    History,
    SetRetentionIntent,
    RetentionProgramP0,
    Action,
    Payload,
>;

pub(super) fn begin<'a>(
    host: &App,
    document: GovernedDocument<'a>,
    days: u64,
    key: u64,
) -> Edit<'a> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    document
        .edit(SetRetentionIntent {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: days,
            },
        })
        .idempotency_owned(&host.request(&principal, &scope), key)
        .unwrap()
        .retain_in_program::<RetentionProgramP0, Action>(host)
        .unwrap_or_else(|(_, cause)| panic!("original action constructs: {cause:?}"))
}

pub(super) fn ready<'a>(host: &App, mut edit: Edit<'a>) -> Edit<'a> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    for _ in 0..16 {
        edit = match edit.advance_with_metadata(
            &host.request(&principal, &scope),
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            input(),
        ) {
            Advance::Progress(edit) => edit,
            Advance::Ready(edit) => return edit,
            Advance::Refused { denial, .. } => panic!("lawful original edit: {denial:?}"),
        };
    }
    panic!("bounded source journey did not settle")
}

pub(super) fn finish<'a>(
    host: &App,
    edit: Edit<'a>,
    days: u64,
) -> (GovernedDocument<'a>, WorthQueryHistoryStep) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let done = edit.finish(&host.request(&principal, &scope)).unwrap();
    let WorthQueryHistoryEditOutcome::Recorded {
        step, completion, ..
    } = done.outcome
    else {
        panic!("fresh key records")
    };
    let WorthQuerySourceCompletion::Fresh {
        receipt, result, ..
    } = completion
    else {
        panic!("fresh original source")
    };
    assert_eq!(result.retention_days, days);
    assert_eq!(
        step.endpoint().selected_commit(),
        receipt.committed_product_publication().composite_commit()
    );
    assert_eq!(
        done.document
            .read(&host.request(&principal, &scope))
            .unwrap()
            .rows()[0]
            .retention,
        days
    );
    (done.document, step)
}

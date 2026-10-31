//! Ordinary public edits own candidate work, exact step bindings and original outcomes.
use super::*;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryHistoryEdit, WorthQueryHistoryEditAdvance as Advance,
        WorthQueryHistoryEditOutcome as Outcome, WorthQueryHistoryEditPosture as Posture,
        WorthQueryHistoryStep, WorthQuerySourceCompletion,
    },
    declaration::{
        application_history::ApplicationHistoryBinding,
        application_program::ApplicationProgramDefinition,
        application_program::{ApplicationActionReference, ApplicationRootComposition},
    },
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    programs::{completion::RetentionCompletion, DocumentRetentionFeature},
    readback::read_retention,
    retention_entry::{SetRetentionBinding, SetRetentionInput, SetRetentionIntent},
};

mod allocation;
mod capture_race;
mod closing;
mod handoff;
mod journeys;
mod model_lifecycle;
mod preconditions;
mod recovery;
mod refusals;

type Action = ApplicationActionReference<
    DocumentRetentionSchema,
    ApplicationRootComposition,
    DocumentRetentionFeature,
    SetRetentionBinding,
    RetentionCompletion,
>;
type Edit<'a, H = RetentionHistory, P = RetentionProgramP0> =
    WorthQueryHistoryEdit<'a, DocumentRetentionSchema, H, SetRetentionIntent, P, Action>;

pub(crate) fn begin<'a, H>(
    host: &App,
    document: Document<'a, H>,
    identity: &str,
    days: u64,
    key: u64,
) -> Edit<'a, H>
where
    H: ApplicationHistoryBinding<DocumentRetentionSchema, Access = DocumentRetentionRead>,
{
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    document
        .edit(SetRetentionIntent {
            input: SetRetentionInput {
                identity: identity.to_owned(),
                retention_days: days,
            },
        })
        .idempotency_owned(&host.runtime().request(&principal, &scope), key)
        .unwrap()
        .retain_in_program::<RetentionProgramP0, Action>(host)
        .unwrap_or_else(|(_, cause)| panic!("declared action constructs: {cause:?}"))
}

pub(crate) fn ready<'a, H, P>(host: &Runtime, mut edit: Edit<'a, H, P>) -> Edit<'a, H, P>
where
    H: ApplicationHistoryBinding<DocumentRetentionSchema, Access = DocumentRetentionRead>,
    P: ApplicationProgramDefinition<DocumentRetentionSchema>,
{
    for _ in 0..16 {
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        edit = match edit.advance(&host.request(&principal, &scope)) {
            Advance::Progress(edit) => edit,
            Advance::Ready(edit) => return edit,
            Advance::Refused { denial, .. } => panic!("lawful edit refused: {denial:?}"),
        };
    }
    panic!("bounded ordinary source edit did not become ready")
}

pub(crate) fn finish<'a, H, P>(
    host: &Runtime,
    edit: Edit<'a, H, P>,
    days: u64,
) -> (Document<'a, H>, WorthQueryHistoryStep)
where
    H: ApplicationHistoryBinding<DocumentRetentionSchema, Access = DocumentRetentionRead>,
    P: ApplicationProgramDefinition<DocumentRetentionSchema>,
{
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let finished = edit.finish(&request).unwrap();
    let Outcome::Recorded {
        step, completion, ..
    } = finished.outcome
    else {
        panic!("new key records its actual performed step")
    };
    let WorthQuerySourceCompletion::Fresh {
        receipt, result, ..
    } = completion
    else {
        panic!("fresh journey preserves its fresh result")
    };
    assert_eq!(result.retention_days, days);
    assert_eq!(
        step.endpoint().selected_commit(),
        receipt.committed_product_publication().composite_commit()
    );
    let read = finished.document.read(&request).unwrap();
    assert_eq!(read.rows()[0].retention_days, days);
    assert_eq!(
        read.receipt().inspect().basis().branch(),
        receipt.commit_reference().branch_id.0
    );
    assert_eq!(
        read.receipt().inspect().basis().version(),
        receipt.commit_reference().version_id.0
    );
    drop(read);
    (finished.document, step)
}

pub(crate) fn close<H: ApplicationHistoryBinding<DocumentRetentionSchema>>(
    host: &Runtime,
    document: Document<'_, H>,
    heads: usize,
) {
    let receipt = host
        .with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
    assert_eq!(receipt.closed_heads(), heads);
    assert!(host.branches().pending_cleanup().is_empty());
}

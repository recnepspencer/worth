use super::*;
use model::{
    programs::{completion::RetentionCompletion, DocumentRetentionFeature},
    retention_entry::{
        DocumentRetentionRead, SetRetentionBinding, SetRetentionInput, SetRetentionIntent,
        DOCUMENT_IDENTITY,
    },
};
use worth_query_host::facade::{application_entry::*, declaration::application_program::*};

type Action = ApplicationActionReference<
    Schema,
    ApplicationRootComposition,
    DocumentRetentionFeature,
    SetRetentionBinding,
    RetentionCompletion,
>;

pub fn attach(app: &App) -> Document<'_> {
    let scope = request_scope();
    let principal = authenticate_operator(app.installed_schema(), &scope);
    app.with_host_advancement(|phase| {
        let selected = app
            .runtime()
            .on_branch(app.current_world())
            .select(&phase)
            .unwrap();
        let head = app
            .branches()
            .fork_expected(selected)
            .unwrap()
            .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
            .create(&phase)
            .unwrap()
            .into_owned_head();
        app.request(&principal, &scope)
            .on_branch(head.branch())
            .attach_history::<RetentionHistory>(
                &phase,
                head,
                DocumentRetentionRead {
                    identity: DOCUMENT_IDENTITY.into(),
                },
            )
            .unwrap()
    })
    .unwrap()
}

pub fn edit<'a>(
    app: &App,
    document: Document<'a>,
    days: u64,
    key: u64,
) -> (Document<'a>, WorthQueryHistoryStep, usize) {
    let scope = request_scope();
    let principal = authenticate_operator(app.installed_schema(), &scope);
    let request = app.request(&principal, &scope);
    let mut edit = document
        .edit(SetRetentionIntent {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.into(),
                retention_days: days,
            },
        })
        .idempotency_owned(&request, key)
        .unwrap()
        .retain_in_program::<RetentionProgramP0, Action>(app)
        .unwrap_or_else(|(_, cause)| panic!("history edit installs: {cause:?}"));
    for _ in 0..16 {
        edit = match edit.advance(&request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(ready) => {
                let finished = ready.finish(&request).unwrap();
                let WorthQueryHistoryEditOutcome::Recorded {
                    step, completion, ..
                } = finished.outcome
                else {
                    panic!("fresh key must record")
                };
                let WorthQuerySourceCompletion::Fresh {
                    receipt, result, ..
                } = completion
                else {
                    panic!("real source required")
                };
                assert_eq!(result.retention_days, days);
                return (finished.document, step, receipt.changed_record_count());
            }
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => {
                panic!("history edit refused: {denial:?}")
            }
        };
    }
    panic!("history edit failed to finish bounded progression")
}

pub fn undo(app: &App, document: &Document<'_>, key: u128) {
    navigate(app, document, None, key)
}

pub fn redo(app: &App, document: &Document<'_>, node: WorthQueryHistoryNodeId, key: u128) {
    navigate(app, document, Some(node), key)
}

fn navigate(app: &App, document: &Document<'_>, redo: Option<WorthQueryHistoryNodeId>, key: u128) {
    use model::document_history::navigation::Navigation;
    let scope = request_scope();
    let principal = authenticate_operator(app.installed_schema(), &scope);
    let request = app.request(&principal, &scope);
    let builder = document.navigate::<Navigation>();
    let target = match redo {
        Some(node) => builder.redo_to(node),
        None => builder.undo_step(),
    };
    let mut owner = target
        .expected_selection(document.selection(&request).unwrap())
        .idempotency(key)
        .prepare(&request)
        .unwrap();
    for _ in 0..16 {
        owner = match owner.advance(&request) {
            WorthQueryHistoryNavigationAdvance::Pending(next) => next,
            WorthQueryHistoryNavigationAdvance::Selected(_) => return,
            WorthQueryHistoryNavigationAdvance::Refused { cause, .. } => {
                panic!("undo refused: {cause:?}")
            }
        };
    }
    panic!("undo failed to finish bounded progression")
}

pub fn unrelated(app: &App, step: usize) {
    let scope = request_scope();
    let principal = authenticate_operator(app.installed_schema(), &scope);
    let result = app
        .request(&principal, &scope)
        .advanced()
        .mutate(SetRetentionIntent {
            input: SetRetentionInput {
                identity: model::retention_entry::RELATED_DOCUMENT_IDENTITY.into(),
                retention_days: if step % 16 == 7 { 3 } else { 7 },
            },
        })
        .without_source()
        .idempotency(&(20000 + step as u64))
        .execute_in_program(
            app,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(
        matches!(
            result,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "unrelated history stimulus must commit: {result:?}"
    );
}

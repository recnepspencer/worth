//! Consuming commands return the original SDK construction and group owners.
use super::*;
type Edit<'r> = WorthQueryHistoryEdit<
    'r,
    ConsumerSchema,
    History,
    Original,
    FixedProgram,
    FixedAction,
    OrdinaryHistoryPayload,
    Outputs,
>;
type Group<'r> = WorthQueryGroupedHistoryDocument<
    'r,
    ConsumerSchema,
    History,
    Original,
    FixedProgram,
    FixedAction,
    OrdinaryHistoryPayload,
    Outputs,
>;

pub(super) fn edit<'r>(
    document: Configured<'r>,
    context: &WorthQueryHistoryDescriptionContext,
    y: u64,
) -> Result<
    WorthQueryHistoryEditBuilder<
        'r,
        ConsumerSchema,
        History,
        Original,
        OrdinaryHistoryPayload,
        Outputs,
    >,
    (Configured<'r>, WorthQueryHistoryCommandContextDenial),
> {
    if let Err(cause) = document.check_history_command_context(context) {
        return Err((document, cause));
    }
    Ok(document.edit(Original(PlanarInitialAdjustment {
        scope_key: "anchor-a".into(),
        replacement_y: length(y),
    })))
}

pub(super) fn record<'r>(
    document: Configured<'r>,
    app: &Runtime,
    context: &WorthQueryHistoryDescriptionContext,
    y: u64,
    key: u64,
) -> (Configured<'r>, WorthQueryHistoryStep) {
    let edit: Edit<'r> = model::with_request(app, |request| {
        edit(document, context, y)
            .unwrap_or_else(|(_, cause)| panic!("edit description context: {cause:?}"))
            .idempotency_owned(request, key)
            .unwrap_or_else(|_| panic!("original edit capture"))
            .retain_in_program::<FixedProgram, FixedAction>(app)
            .unwrap_or_else(|(_, cause)| panic!("original program construction: {cause:?}"))
    });
    finish(edit, app)
}

fn finish<'r>(mut edit: Edit<'r>, app: &Runtime) -> (Configured<'r>, WorthQueryHistoryStep) {
    for _ in 0..96 {
        if edit.posture() == WorthQueryHistoryEditPosture::Selected {
            let finished = model::with_request(app, |request| edit.finish(request))
                .unwrap_or_else(|(_, cause)| panic!("original edit finish: {cause:?}"));
            let WorthQueryFixedHistoryEditOutcome::Recorded { step, delivery } = finished.outcome
            else {
                panic!("one new authored occurrence");
            };
            assert_eq!(
                delivery.settlement().output_count(),
                1,
                "genuine required child"
            );
            assert_eq!(
                delivery.settlement().observation().selected_commit(),
                step.endpoint().selected_commit()
            );
            drop(delivery);
            return (finished.document, step);
        }
        edit = match model::with_request(app, |request| edit.advance(request)) {
            WorthQueryHistoryEditAdvance::Progress(next)
            | WorthQueryHistoryEditAdvance::Ready(next) => next,
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => {
                panic!("original edit refused: {denial:?}")
            }
        };
    }
    panic!("original fixed source/required graph did not settle");
}

pub(super) enum GroupStartDenial {
    Context(WorthQueryHistoryCommandContextDenial),
    Group(WorthQueryHistoryGroupDenial),
}

impl std::fmt::Debug for GroupStartDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Context(cause) => formatter.debug_tuple("Context").field(cause).finish(),
            Self::Group(cause) => formatter.debug_tuple("Group").field(cause).finish(),
        }
    }
}

pub(super) fn group<'r>(
    document: Configured<'r>,
    app: &Runtime,
    request: &Request<'_, '_, '_>,
    context: &WorthQueryHistoryDescriptionContext,
    key: u128,
    members: [(Original, u64); 2],
) -> Result<Group<'r>, (Configured<'r>, GroupStartDenial)> {
    if let Err(cause) = document.check_history_command_context(context) {
        return Err((document, GroupStartDenial::Context(cause)));
    }
    document
        .group_edits::<Original, FixedProgram, FixedAction, 2>(app, request, key, members)
        .map_err(|(document, cause)| (document, GroupStartDenial::Group(cause)))
}

pub(super) fn complete_group<'r>(
    mut group: Group<'r>,
    app: &Runtime,
) -> (Configured<'r>, WorthQueryHistoryStep) {
    for _ in 0..128 {
        if group.posture() == WorthQueryHistoryGroupPosture::ReadyToComplete {
            let step = model::with_request(app, |request| {
                let (document, execution) = group.parts();
                execution.complete(document, request).unwrap()
            });
            let finished = model::with_request(app, |request| group.finish(request))
                .unwrap_or_else(|(_, cause)| panic!("same grouped document finish: {cause:?}"));
            assert_eq!(finished.completions.len(), 2);
            drop(finished.completions);
            return (finished.document, step);
        }
        model::with_request(app, |request| {
            let (document, execution) = group.parts();
            execution.advance(document, request).unwrap();
        });
    }
    panic!("finite original group did not complete");
}

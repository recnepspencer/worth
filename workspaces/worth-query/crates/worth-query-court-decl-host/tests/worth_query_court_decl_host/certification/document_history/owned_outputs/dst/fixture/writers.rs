//! Writers that hold the move-only document (model `WriterBegin`,
//! `WriterCommit`, `WriterDiscard`). An `.edit` takes its intent when it
//! starts, and the model's begin has none, so a begun writer holds the
//! document and starts at commit or discard, each under one fresh request.
//! Any other refusal fails the seed with it: the model has no refused writer.
use super::super::reference::Writer;
use super::reads::{with_request, Forest, Host};
use super::*;

/// Edits and gestures end well inside this many advances.
const ADVANCES: usize = 512;

/// A begun writer: the document it holds until it commits or discards.
pub struct Held<'h> {
    pub class: Writer,
    pub document: Forest<'h>,
}

/// One edit of `intent` through program action `$action`. With `$give_up`,
/// it is discarded once it awaits selection; otherwise it records.
macro_rules! edit {
    ($host:expr, $document:expr, $request:expr, $key:expr, $intent:expr, $action:ty, $give_up:expr) => {{
        let (host, request, key, give_up): (&Host, &family::Request<'_, '_, '_>, String, bool) =
            ($host, $request, $key, $give_up);
        let mut edit = $document
            .edit($intent)
            .idempotency_owned(request, key.clone())
            .unwrap_or_else(|cause| panic!("{key}: the edit must take its key: {cause:?}"))
            .retain_in_program::<program::Program, $action>(host)
            .unwrap_or_else(|(_, cause)| panic!("{key}: the edit must start: {cause:?}"));
        let mut ended = None;
        for _ in 0..ADVANCES {
            if give_up && edit.posture() == WorthQueryHistoryEditPosture::AwaitingSelection {
                let document = edit
                    .discard(request)
                    .unwrap_or_else(|(_, cause)| panic!("{key}: the edit must discard: {cause:?}"));
                ended = Some((document, None, None));
                break;
            }
            edit = match edit.advance(request) {
                WorthQueryHistoryEditAdvance::Progress(next) => next,
                WorthQueryHistoryEditAdvance::Ready(next) => {
                    assert!(!give_up, "{key}: the edit must await selection before it is ready");
                    let WorthQueryHistoryEditFinish { document, outcome } = next
                        .finish(request)
                        .unwrap_or_else(|(_, cause)| panic!("{key}: the edit must finish: {cause:?}"));
                    let WorthQueryHistoryEditOutcome::Recorded { step, completion, .. } = &outcome else {
                        panic!("{key}: the edit must record")
                    };
                    let step = step.clone();
                    ended = Some((document, Some(step), completion.live_receipt().cloned()));
                    break;
                }
                // The held writer starts here, after any write outside since
                // its begin: on a diverged head it is refused at its read and
                // given up as the model's discard (traced, seed 0x7b020008).
                WorthQueryHistoryEditAdvance::Refused {
                    edit,
                    denial:
                        WorthQueryHistoryEditDenial::History(WorthQueryApplicationHistoryDenial::Document(
                            WorthQueryDocumentHistoryDenial::HeadDiverged,
                        )),
                } if give_up => {
                    let document = edit
                        .discard(request)
                        .unwrap_or_else(|(_, cause)| panic!("{key}: the refused edit must discard: {cause:?}"));
                    ended = Some((document, None, None));
                    break;
                }
                WorthQueryHistoryEditAdvance::Refused { denial, .. } => {
                    panic!("{key}: the edit must proceed: {denial:?}")
                }
            };
        }
        ended.unwrap_or_else(|| panic!("{key}: the edit must end within {ADVANCES} advances"))
    }};
}

/// A gesture member's commit receipt, from its typed completion.
macro_rules! member_receipt {
    ($completion:expr, $call:expr) => {
        match $call {
            Call::Create(E1, _) => $completion.original::<create_e1::Intent, Action<create_e1::Binding>>().and_then(|(_, done)| done.live_receipt().cloned()),
            Call::Create(_, _) => $completion.original::<create_e2::Intent, Action<create_e2::Binding>>().and_then(|(_, done)| done.live_receipt().cloned()),
            Call::Set(E1, _) => $completion.original::<set_e1::Intent, Action<set_e1::Binding>>().and_then(|(_, done)| done.live_receipt().cloned()),
            Call::Set(_, _) => $completion.original::<set_e2::Intent, Action<set_e2::Binding>>().and_then(|(_, done)| done.live_receipt().cloned()),
            Call::Retire(E1) => $completion.original::<retire_e1::Intent, Action<retire_e1::Binding>>().and_then(|(_, done)| done.live_receipt().cloned()),
            Call::Retire(_) => $completion.original::<retire_e2::Intent, Action<retire_e2::Binding>>().and_then(|(_, done)| done.live_receipt().cloned()),
        }
    };
}

/// The writer's posture at which a gesture is given up.
fn gives_up(posture: WorthQueryHistoryGroupPosture) -> bool {
    matches!(
        posture,
        WorthQueryHistoryGroupPosture::QualifyingOutputs | WorthQueryHistoryGroupPosture::ReadyToComplete
    )
}

/// Record the held writer's step: an edit runs its one call, a gesture
/// pushes every call in order. `key` names the edit; `gesture` the group.
pub fn commit<'h>(
    host: &'h Host,
    held: Held<'h>,
    key: String,
    gesture: u128,
    calls: &[Call],
) -> (Forest<'h>, WorthQueryHistoryStep, Vec<Option<super::envelope::Receipt>>) {
    let Held { class, document } = held;
    with_request(host, |request| match class {
        Writer::Edit => {
            let [call] = calls else { panic!("{key}: an edit commits one call: {calls:?}") };
            let (document, step, receipt) =
                dispatch!(*call, intent, Act => edit!(host, document, request, key.clone(), intent, Act, false));
            (document, step.expect("a recording edit has its step"), vec![receipt])
        }
        Writer::Gesture => {
            let mut group = document
                .gesture(host, request, gesture)
                .unwrap_or_else(|(_, cause)| panic!("{key}: the gesture must open: {cause:?}"));
            for (index, call) in calls.iter().enumerate() {
                let member = format!("{key}:{index}");
                dispatch!(*call, intent, Act => group.push::<_, Act>(request, intent, member))
                    .unwrap_or_else(|cause| panic!("{key}: {call:?} must join the gesture: {cause:?}"));
            }
            for _ in 0..ADVANCES {
                let (document, execution) = group.parts();
                let posture = execution
                    .advance(document, request)
                    .unwrap_or_else(|cause| panic!("{key}: the gesture must proceed: {cause:?}"));
                if posture == WorthQueryHistoryGroupPosture::ReadyToComplete {
                    let step = execution
                        .complete(document, request)
                        .unwrap_or_else(|cause| panic!("{key}: the gesture must complete: {cause:?}"));
                    let finished = group
                        .finish(request)
                        .unwrap_or_else(|(_, cause)| panic!("{key}: the gesture must finish: {cause:?}"));
                    let receipts = finished
                        .completions
                        .iter()
                        .zip(calls)
                        .map(|(completion, call)| member_receipt!(completion, *call))
                        .collect();
                    return (finished.document, step, receipts);
                }
            }
            panic!("{key}: the gesture must end within {ADVANCES} advances")
        }
    })
}

/// Start the held writer on an edit of the owned model and give it up
/// before it records: the document comes back unchanged.
pub fn discard<'h>(host: &'h Host, held: Held<'h>, gesture: u128) -> Forest<'h> {
    let Held { class, document } = held;
    let intent = || edit::Intent { key: "a".into(), value: 1 };
    let key = format!("discard:{gesture:#x}");
    with_request(host, |request| match class {
        Writer::Edit => {
            let (document, step, _) =
                edit!(host, document, request, key.clone(), intent(), super::super::super::program::EditAction, true);
            assert!(step.is_none(), "{key}: a discarded edit records nothing");
            document
        }
        Writer::Gesture => {
            let mut group = document
                .gesture(host, request, gesture)
                .unwrap_or_else(|(_, cause)| panic!("{key}: the gesture must open: {cause:?}"));
            group
                .push::<_, super::super::super::program::EditAction>(request, intent(), "model".into())
                .unwrap_or_else(|cause| panic!("{key}: the edit must join the gesture: {cause:?}"));
            for _ in 0..ADVANCES {
                let (document, execution) = group.parts();
                let given_up = match execution.advance(document, request) {
                    Ok(posture) => gives_up(posture),
                    // As an `.edit`'s: on a head diverged since its begin
                    // the gesture is refused at its read and given up as the
                    // model's discard (traced, seed 0x7b02010e).
                    Err(WorthQueryHistoryGroupDenial::History(WorthQueryApplicationHistoryDenial::Document(
                        WorthQueryDocumentHistoryDenial::HeadDiverged,
                    ))) => true,
                    Err(cause) => panic!("{key}: the gesture must proceed: {cause:?}"),
                };
                if given_up {
                    return group
                        .abort(request)
                        .unwrap_or_else(|(_, cause)| panic!("{key}: the gesture must abort: {cause:?}"));
                }
            }
            panic!("{key}: the gesture must reach its outputs within {ADVANCES} advances")
        }
    })
}

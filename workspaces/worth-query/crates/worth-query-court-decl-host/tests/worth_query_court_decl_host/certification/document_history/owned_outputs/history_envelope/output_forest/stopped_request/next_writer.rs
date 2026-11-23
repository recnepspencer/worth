//! A single edit or a gesture that starts right after a stopped envelope,
//! before any maintenance, meets the trees the stopped request left bound.
use super::*;

/// How the writer that follows the stopped envelope ends.
#[derive(Clone, Copy)]
enum End {
    /// It records its step.
    Record,
    /// The caller gives it up before it records any output tree.
    GiveUp,
}

/// A fresh request per step keeps the stopped request out of what follows.
fn with_request<T>(host: &world::Host, run: impl FnOnce(&family::Request<'_, '_, '_>) -> T) -> T {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    run(&host.request(&principal, &scope))
}

/// Stop an envelope while its output trees record. The request ends before it
/// can release them, so they stay bound to the candidate it leaves parked.
fn stop_while_the_trees_record<'d>(
    host: &world::Host,
    document: Forest<'d>,
    stages: Stages,
    stop_at: usize,
) -> Forest<'d> {
    let before = observe(host, &document);
    let (execution, _) = record_stopping_at(host, document, before.branch, stages, stop_at);
    let (document, denial) = refused(execution);
    assert!(
        matches!(denial, Denial::Outputs(_)),
        "the request must stop while the trees record: {denial:?}"
    );
    drop(denial);
    assert_eq!(observe(host, &document), before);
    document
}

/// One model edit on the document, started and ended under fresh requests.
fn edit_model<'d>(
    host: &world::Host,
    document: Forest<'d>,
    key: &str,
    value: u64,
    end: End,
) -> (Forest<'d>, Option<WorthQueryHistoryStep>) {
    with_request(host, |request| {
        let mut edit = document
            .edit(edit::Intent {
                key: "a".into(),
                value,
            })
            .idempotency_owned(request, key.to_owned())
            .unwrap()
            .retain_in_program::<program::Program, program::EditAction>(host)
            .unwrap_or_else(|(_, cause)| panic!("{key}: {cause:?}"));
        for _ in 0..512 {
            if matches!(end, End::GiveUp)
                && edit.posture() == WorthQueryHistoryEditPosture::AwaitingSelection
            {
                let document = edit
                    .discard(request)
                    .unwrap_or_else(|(_, cause)| panic!("{key}: the edit must discard: {cause:?}"));
                return (document, None);
            }
            edit = match edit.advance(request) {
                WorthQueryHistoryEditAdvance::Progress(next) => next,
                WorthQueryHistoryEditAdvance::Ready(next) => {
                    let WorthQueryHistoryEditFinish { document, outcome } = next
                        .finish(request)
                        .unwrap_or_else(|(_, cause)| panic!("{key}: {cause:?}"));
                    let WorthQueryHistoryEditOutcome::Recorded { step, .. } = &outcome else {
                        panic!("{key}: the edit must record")
                    };
                    return (document, Some(step.clone()));
                }
                WorthQueryHistoryEditAdvance::Refused { denial, .. } => {
                    panic!("{key}: the edit must proceed: {denial:?}")
                }
            };
        }
        panic!("{key}: the edit must end")
    })
}

/// One gesture of a single model edit on the document.
fn gesture_model<'d>(
    host: &world::Host,
    document: Forest<'d>,
    key: u128,
    value: u64,
    end: End,
) -> (Forest<'d>, Option<WorthQueryHistoryStep>) {
    with_request(host, |request| {
        let mut group = document
            .gesture(host, request, key)
            .unwrap_or_else(|(_, cause)| panic!("{key:#x}: {cause:?}"));
        group
            .push::<_, program::EditAction>(
                request,
                edit::Intent {
                    key: "a".into(),
                    value,
                },
                "model".into(),
            )
            .unwrap_or_else(|cause| panic!("{key:#x}: {cause:?}"));
        for _ in 0..512 {
            let (document, execution) = group.parts();
            let posture = execution
                .advance(document, request)
                .unwrap_or_else(|cause| panic!("{key:#x}: the gesture must proceed: {cause:?}"));
            match (posture, end) {
                (WorthQueryHistoryGroupPosture::QualifyingOutputs, End::GiveUp) => {
                    let document = group.abort(request).unwrap_or_else(|(_, cause)| {
                        panic!("{key:#x}: the gesture must abort: {cause:?}")
                    });
                    return (document, None);
                }
                (WorthQueryHistoryGroupPosture::ReadyToComplete, _) => {
                    let step = execution
                        .complete(document, request)
                        .unwrap_or_else(|cause| panic!("{key:#x}: {cause:?}"));
                    let finished = group
                        .finish(request)
                        .unwrap_or_else(|(_, cause)| panic!("{key:#x}: {cause:?}"));
                    return (finished.document, Some(step));
                }
                _ => {}
            }
        }
        panic!("{key:#x}: the gesture must end")
    })
}

/// After each stopped envelope the next writer runs before any maintenance.
/// Given up, it returns the unchanged document. Recording, it adds its one
/// step with both trees. Either way maintenance then closes the candidate the
/// stopped request left, and the stopped envelope's own key records.
fn the_next_writer_is_not_wedged(
    write: for<'d> fn(&world::Host, Forest<'d>, End, u64) -> (Forest<'d>, Option<WorthQueryHistoryStep>),
) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let document = attach_with_both_trees(&host);
    let branch = observe(&host, &document).branch;
    let unstopped = Stages {
        key: "unstopped",
        model: 2,
        member: 6,
    };
    let (execution, consults) = record_stopping_at(&host, document, branch, unstopped, usize::MAX);
    let (mut document, _) = recorded(execution);
    // Well inside the part of the execution in which the trees record.
    let stop_at = consults - consults / 8;
    for (round, end) in [End::GiveUp, End::Record].into_iter().enumerate() {
        let round = round as u64;
        let stopped = Stages {
            key: ["stopped-then-given-up", "stopped-then-recorded"][round as usize],
            model: 3 + round,
            member: 7 + round,
        };
        document = stop_while_the_trees_record(&host, document, stopped, stop_at);
        let before = observe(&host, &document);
        let original = snapshot(&host, &document);
        let value = 40 + round;
        let (next, step) = write(&host, document, end, value);
        document = next;
        let head = match (end, step) {
            (End::GiveUp, None) => {
                assert_eq!(observe(&host, &document), before);
                assert_eq!(snapshot(&host, &document), original);
                before.current
            }
            (End::Record, Some(step)) => {
                assert_eq!(step.parent(), Some(before.current));
                assert_eq!(observe(&host, &document).current, step.id());
                let now = snapshot(&host, &document);
                assert_eq!(
                    (now.model, now.member, now.outputs.0.value, now.outputs.1.value),
                    (value, original.member, value * 10, 1000 + value)
                );
                step.id()
            }
            (_, step) => panic!("round {round}: {step:?}"),
        };
        let passes = maintain(&host, &mut document);
        assert!(
            !passes.is_empty()
                && passes
                    .iter()
                    .all(|pass| *pass == WorthQueryHistoryReleaseOutcome::Closed),
            "round {round}: the parked candidate must close: {passes:?}"
        );
        let (next, step) =
            recorded(record(&host, document, &request_scope(), stopped, metadata(b"")));
        document = next;
        assert_eq!(step.parent(), Some(head), "round {round}");
        let now = snapshot(&host, &document);
        assert_eq!(
            (now.model, now.member, now.outputs.0.value, now.outputs.1.value),
            (stopped.model, stopped.member, stopped.model * 10, 1000 + stopped.model),
            "round {round}"
        );
    }
    close(&host, document);
}

#[test]
fn an_edit_right_after_a_stopped_envelope_discards_or_records_without_maintenance() {
    the_next_writer_is_not_wedged(|host, document, end, value| {
        let key = match end {
            End::GiveUp => "edit-given-up",
            End::Record => "edit-recorded",
        };
        edit_model(host, document, key, value, end)
    });
}

#[test]
fn a_gesture_right_after_a_stopped_envelope_aborts_or_records_without_maintenance() {
    the_next_writer_is_not_wedged(|host, document, end, value| {
        let key = match end {
            End::GiveUp => 0x57_b001,
            End::Record => 0x57_b002,
        };
        gesture_model(host, document, key, value, end)
    });
}

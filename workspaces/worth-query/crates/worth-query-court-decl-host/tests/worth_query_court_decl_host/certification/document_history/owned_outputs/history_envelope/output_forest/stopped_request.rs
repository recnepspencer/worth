//! A request that stops while an envelope records its output trees leaves a
//! document the next envelope records on.
use super::*;
use worth_execution::with_request_consult_observer_for_test;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAdmissionClock, WorthQueryCancellationSource,
};

mod next_writer;

/// Stop points spread evenly over one envelope's execution, last consult first.
/// Every recorded step keeps its previous head alive, and the installed world
/// admits 32 live branches.
const STOP_POINTS: usize = 16;

/// Run history maintenance under a fresh request until nothing is pending;
/// return what each pass did.
fn maintain(host: &world::Host, document: &mut Forest<'_>) -> Vec<WorthQueryHistoryReleaseOutcome> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut passes = Vec::new();
    for _ in 0..4 {
        match document.maintain_history(&request).unwrap() {
            WorthQueryHistoryReleaseOutcome::NoPendingRelease => break,
            other => passes.push(other),
        }
    }
    passes
}

/// Execute `stages` under a request that stops at consult `stop_at` of the
/// execution; also return the consults the execution made.
fn record_stopping_at<'d>(
    host: &world::Host,
    document: Forest<'d>,
    branch: WorthQueryProductBranch,
    stages: Stages,
    stop_at: usize,
) -> (Execution<'d, WorthQueryHistoryWithOutputTrees<Schema>>, usize) {
    let cancellation = WorthQueryCancellationSource::new();
    let scope =
        WorthQueryRequestScope::new(None, cancellation.token(), WorthQueryAdmissionClock::system());
    let mut consults = 0;
    let execution = record_within(
        host,
        document,
        &scope,
        branch,
        stages,
        metadata(b""),
        |execute| {
            let (execution, count) = with_request_consult_observer_for_test(
                move |at| {
                    if at == stop_at {
                        cancellation.cancel();
                    }
                },
                execute,
            );
            consults = count;
            execution
        },
    );
    (execution, consults)
}

/// Wherever the request stops, the envelope either recorded its one step or
/// rolled back: the document is unchanged, one maintenance pass closes the
/// candidate the stopped request could not, and the same key then records the
/// step with both trees. Stops after a producer has recorded on the private
/// candidate are the ones that leave bound trees for the refusal to release.
/// Every other stop records again before maintenance runs, so the next
/// envelope itself releases the trees the stopped request left bound.
#[test]
fn a_request_stopped_anywhere_in_an_envelope_rolls_back_and_the_same_key_then_records() {
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
    assert!(consults >= STOP_POINTS, "{consults} consults");
    let stride = consults / STOP_POINTS;
    let (mut refusals, mut refusals_after_a_producer, mut parked) = (0, 0, 0);
    let mut recorded_before_maintenance = 0;
    for point in 0..STOP_POINTS {
        let stop_at = consults - point * stride;
        let before = observe(&host, &document);
        let original = snapshot(&host, &document);
        let stages = Stages {
            key: Box::leak(format!("stopped-{point}").into_boxed_str()),
            model: 3 + (point as u64 % 7),
            member: 7 + point as u64,
        };
        let contacts = host.producer_contacts_on_this_thread_for_test();
        let (execution, _) = record_stopping_at(&host, document, before.branch, stages, stop_at);
        let (next, step) = match execution {
            Ok(Outcome::Recorded { document, step, .. }) => (document, step),
            Ok(other) => panic!("stop at {stop_at}: {other:?}"),
            Err(failure) => {
                let (mut document, denial) = failure.into_parts();
                // The denial may hold receipts that keep the candidate's history.
                let denial = format!("{denial:?}");
                refusals += 1;
                let after_a_producer = host.producer_contacts_on_this_thread_for_test() > contacts;
                refusals_after_a_producer += usize::from(after_a_producer);
                assert_eq!(observe(&host, &document), before, "stop at {stop_at}: {denial}");
                assert_eq!(snapshot(&host, &document), original, "stop at {stop_at}");
                let mut step_before_maintenance = None;
                if point % 2 == 1 {
                    let (next, step) =
                        recorded(record(&host, document, &request_scope(), stages, metadata(b"")));
                    document = next;
                    step_before_maintenance = Some(step);
                    recorded_before_maintenance += usize::from(after_a_producer);
                }
                let passes = maintain(&host, &mut document);
                eprintln!("stop at {stop_at}: {denial}; maintenance {passes:?}");
                assert!(
                    passes.len() <= 1
                        && passes
                            .iter()
                            .all(|pass| *pass == WorthQueryHistoryReleaseOutcome::Closed),
                    "stop at {stop_at}: the parked candidate must close: {passes:?}"
                );
                parked += passes.len();
                match step_before_maintenance {
                    Some(step) => (document, step),
                    None => {
                        assert_eq!(snapshot(&host, &document), original, "stop at {stop_at}");
                        recorded(record(&host, document, &request_scope(), stages, metadata(b"")))
                    }
                }
            }
        };
        document = next;
        assert_eq!(step.parent(), Some(before.current), "stop at {stop_at}");
        assert_eq!(observe(&host, &document).current, step.id());
        let now = snapshot(&host, &document);
        assert_eq!(
            (now.model, now.member, now.outputs.0.value, now.outputs.1.value),
            (stages.model, stages.member, stages.model * 10, 1000 + stages.model),
            "stop at {stop_at}"
        );
    }
    eprintln!(
        "stopped-request sweep: consults={consults} stride={stride} refusals={refusals} \
         after_a_producer={refusals_after_a_producer} parked={parked} \
         recorded_before_maintenance={recorded_before_maintenance}"
    );
    assert!(parked > 0, "no stop left a candidate for maintenance to close");
    assert!(
        refusals_after_a_producer > 0,
        "no stop refused an envelope whose trees were already bound to its candidate"
    );
    assert!(
        recorded_before_maintenance > 0,
        "no envelope recorded over trees a stopped request had left bound"
    );
    close(&host, document);
}

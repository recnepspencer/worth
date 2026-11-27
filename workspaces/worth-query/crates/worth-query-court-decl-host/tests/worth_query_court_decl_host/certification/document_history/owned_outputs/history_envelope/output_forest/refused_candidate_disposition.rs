//! A refused envelope parks its private candidate for retention maintenance
//! to close; a refused edit keeps it, and the output trees it recorded on
//! it, for its retry.
use super::*;
use std::time::Duration;
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryAdmissionClock, WorthQueryCancellationSource},
    application_entry::{WorthQueryDocumentHistoryDenial, WorthQueryHistoryEditDenial},
    product::WorthQueryExpectedProductBranchAdoptionDenial,
};

/// The envelope releases its bound tree before it parks its candidate, so
/// the next maintenance closes the parked candidate and the one after finds
/// nothing pending; the same key then refuses the same way. The edit's
/// adoption, stopped after every tree recorded, keeps the candidate and the
/// recorded trees: its retry adopts them without contacting a producer
/// again.
#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "certification-harness thread-scope=1 scoped-thread-spawn=1: a_refused_envelope_parks_and_a_refused_edit_keeps_its_candidate stops the edit's adoption at its real capture rendezvous from a second actor."
)]
fn a_refused_envelope_parks_and_a_refused_edit_keeps_its_candidate() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    a_refused_envelope_parks_its_candidate_for_maintenance();
    a_refused_edit_keeps_its_recorded_trees();
}

fn a_refused_envelope_parks_its_candidate_for_maintenance() {
    let host = world::install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    first::settle(&host, &request, "a");
    second::settle(&host, &request, "a");
    // The second family is owned by the model but bound to no tree, so the
    // envelope refuses once its bound tree has recorded.
    let document = world::attach(&host);
    let document = host
        .with_host_advancement(|phase| {
            document
                .with_output_trees::<program::Program, program::FirstTree>(
                    &phase,
                    &request,
                    &host,
                    first::Demand("a".into()),
                    WorthQueryOutputDemandControls::default(),
                )
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
        })
        .unwrap();
    let before = observe(&host, &document);
    let stages = Stages {
        key: "refused-envelope",
        model: 2,
        member: 6,
    };
    let incomplete = |denial: &Denial| {
        matches!(
            denial,
            Denial::Outputs(WorthQueryHistoryOutputDenial::RecordingQualification(
                WorthQueryRecordingOutputQualificationDenial::Incomplete { .. }
            ))
        )
    };
    let (mut document, denial) = refused(record(&host, document, &scope, stages, metadata(b"")));
    assert!(incomplete(&denial), "{denial:?}");
    assert_eq!(observe(&host, &document), before);
    assert_eq!(
        document.maintain_history(&request).unwrap(),
        WorthQueryHistoryReleaseOutcome::Closed,
        "maintenance closes the refused envelope's parked candidate"
    );
    assert_eq!(
        document.maintain_history(&request).unwrap(),
        WorthQueryHistoryReleaseOutcome::NoPendingRelease,
        "the parked candidate was the only pending release"
    );
    let (document, again) = refused(record(&host, document, &scope, stages, metadata(b"")));
    assert!(incomplete(&again), "{again:?}");
    assert_eq!(observe(&host, &document), before);
    close(&host, document);
}

fn a_refused_edit_keeps_its_recorded_trees() {
    let host = world::install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    first::settle(&host, &request, "a");
    second::settle(&host, &request, "a");
    let document = courts::configure_all_trees(&host, world::attach(&host));
    let before = document.current_step(&request).unwrap().id();
    let mut edit = document
        .edit(edit::Intent {
            key: "a".into(),
            value: 5,
        })
        .idempotency_owned(&request, "refused-edit-keeps-its-candidate".into())
        .unwrap()
        .retain_in_program::<program::Program, program::EditAction>(&host)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    let pause = edit.pause_after_history_capture_for_test();
    let cancellation = WorthQueryCancellationSource::new();
    let stopped = WorthQueryRequestScope::new(
        None,
        cancellation.token(),
        WorthQueryAdmissionClock::system(),
    );
    let stopped_principal = authenticate_operator(host.installed_schema(), &stopped);
    let stopped_request = host.request(&stopped_principal, &stopped);
    // The edit records every tree under a request that a second actor
    // cancels at the adoption's capture, after the trees recorded. The edit
    // holds its output trees, so it advances on this thread.
    let (reached, result) = std::thread::scope(|threads| {
        let control = threads.spawn(|| {
            let reached = pause.wait_until_reached(Duration::from_secs(60));
            if reached {
                cancellation.cancel();
            }
            pause.release();
            reached
        });
        let mut edit = edit;
        let mut advances = 0;
        let result = loop {
            advances += 1;
            assert!(advances < 512, "the edit never reached adoption");
            edit = match edit.advance(&stopped_request) {
                WorthQueryHistoryEditAdvance::Progress(next) => next,
                other => break other,
            };
        };
        (control.join().unwrap(), result)
    });
    assert!(reached, "the edit's adoption must reach its capture");
    let WorthQueryHistoryEditAdvance::Refused { edit, denial } = result else {
        panic!("the stopped adoption must refuse")
    };
    assert!(
        matches!(
            denial,
            WorthQueryHistoryEditDenial::History(WorthQueryApplicationHistoryDenial::Document(
                WorthQueryDocumentHistoryDenial::Adoption(
                    WorthQueryExpectedProductBranchAdoptionDenial::Stopped(_)
                )
            ))
        ),
        "{denial:?}"
    );
    assert_eq!(
        edit.posture(),
        WorthQueryHistoryEditPosture::AwaitingSelection
    );
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let mut edit = edit;
    let mut advances = 0;
    let edit = loop {
        advances += 1;
        assert!(advances < 512, "the retry did not adopt");
        edit = match edit.advance(&request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(next) => break next,
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => panic!("retry: {denial:?}"),
        };
    };
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts,
        "the retry adopts the trees the refused adoption kept"
    );
    let finish = edit
        .finish(&request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    assert!(matches!(
        finish.outcome,
        WorthQueryHistoryEditOutcome::Recorded { .. }
    ));
    let document = finish.document;
    drop(finish.outcome);
    let future = document.selection(&request).unwrap();
    assert_ne!(future.step().id(), before);
    let recorded = world::outputs(&host, future.physical_endpoint().product_branch());
    assert_eq!(
        (recorded.0.value, recorded.1.value),
        (50, 1005),
        "the recorded step includes both families at the edited source"
    );
    close(&host, document);
}

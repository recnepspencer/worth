//! The caller key names one recorded step for one complete intent.
use super::*;

#[test]
fn a_replayed_envelope_names_its_recorded_step_and_a_changed_intent_drifts() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let scope = request_scope();
    let stages = Stages {
        key: "once",
        model: 2,
        member: 6,
    };
    let (document, step) = recorded(record(
        &host,
        world::attach(&host),
        &scope,
        stages,
        metadata(b"first call"),
    ));
    let after = observe(&host, &document);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let replayed = match record(&host, document, &scope, stages, metadata(b"second call")) {
        Ok(Outcome::AlreadyRecorded { document, step }) => (document, step),
        other => panic!("the same key and intent must name the recorded step: {other:?}"),
    };
    let (document, again) = replayed;
    assert_eq!(again.id(), step.id());
    assert_eq!(again.metadata().bytes(), b"first call");
    assert_eq!(observe(&host, &document), after);
    let changed = Stages { model: 3, ..stages };
    let (document, denial) = refused(record(&host, document, &scope, changed, metadata(b"")));
    assert!(
        matches!(
            denial,
            Denial::History(WorthQueryApplicationHistoryDenial::Document(
                WorthQueryDocumentHistoryDenial::IntentDrift
            ))
        ),
        "{denial:?}"
    );
    assert_eq!(observe(&host, &document), after);
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    close(&host, document);
}

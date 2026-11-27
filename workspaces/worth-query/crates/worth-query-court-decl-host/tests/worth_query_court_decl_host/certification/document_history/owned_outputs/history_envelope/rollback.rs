//! An envelope that records nothing returns the document at its prior head.
use super::*;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAdmissionClock, WorthQueryCancellationSource,
};
use worth_query_host::facade::application_entry::{
    WorthQueryEditEnvelopeModeDenial, WorthQueryEditEnvelopeOutcome as Bare,
};

const VALID: Stages = Stages {
    key: "retried",
    model: 2,
    member: 6,
};

/// The same key commits the next step on the unchanged head.
fn retry_records_the_next_step<'a>(
    host: &'a world::Host,
    document: world::Document<'a>,
    before: &Observed,
) -> world::Document<'a> {
    let scope = request_scope();
    let (document, step) = recorded(record(host, document, &scope, VALID, metadata(b"")));
    assert_eq!(step.parent(), Some(before.current));
    let after = observe(host, &document);
    assert_eq!((after.model, after.member), (2, 6));
    assert_eq!(after.nodes.len(), before.nodes.len() + 1);
    document
}

#[test]
fn a_cancelled_builder_returns_the_untouched_document() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let document = world::attach(&host);
    let before = observe(&host, &document);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope).on_branch(before.branch);
    let mut envelope = document
        .edit_envelope(&host, &request, "never-run".to_owned())
        .unwrap_or_else(|failure| panic!("{failure:?}"));
    envelope
        .push_stage::<ApplicationNoOutputGraph, _>(edit::Intent {
            key: "a".into(),
            value: 2,
        })
        .unwrap();
    let (envelope, denial) = envelope
        .ordered_authored()
        .expect_err("the order is chosen before the first stage");
    assert_eq!(
        denial,
        WorthQueryEditEnvelopeModeDenial::StagesAlreadyCaptured
    );
    let document = envelope.cancel();
    assert_eq!(observe(&host, &document), before);
    close(&host, document);
}

#[test]
fn a_domain_denied_stage_rolls_back_and_the_same_key_then_records() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let document = world::attach(&host);
    let before = observe(&host, &document);
    let scope = request_scope();
    let denied = Stages {
        member: 2000,
        ..VALID
    };
    let (document, denial) = refused(record(&host, document, &scope, denied, metadata(b"")));
    assert!(
        matches!(denial, Denial::Envelope(Bare::DomainDenied(_))),
        "{denial:?}"
    );
    assert_eq!(observe(&host, &document), before);
    let document = retry_records_the_next_step(&host, document, &before);
    close(&host, document);
}

/// The member stage stops its own request while deciding, so the rollback
/// cannot advance on that request either. The private candidate is left for
/// the document's own disposal, and the record is still released.
#[test]
fn a_request_cancelled_during_a_stage_rolls_back_and_the_same_key_then_records() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let document = world::attach(&host);
    let before = observe(&host, &document);
    let index = document.history_source_index_for_test();
    let cancellation = WorthQueryCancellationSource::new();
    let stopped =
        WorthQueryRequestScope::new(None, cancellation.token(), WorthQueryAdmissionClock::system());
    member_edit::stop_request_during_next_decide(cancellation);
    let (document, denial) = refused(record(&host, document, &stopped, VALID, metadata(b"")));
    assert!(
        matches!(denial, Denial::Envelope(Bare::StageExecutionDenied { .. })),
        "{denial:?}"
    );
    assert!(
        host.runtime()
            .with_application_advancement(&stopped, |_| ())
            .is_err(),
        "the stopped request cannot open the phase a rollback would close in"
    );
    assert_eq!(observe(&host, &document), before);
    assert_eq!(
        document.history_source_index_for_test(),
        index,
        "the cancel releases its index entries"
    );
    let document = retry_records_the_next_step(&host, document, &before);
    let indexed = document.history_source_index_for_test();
    assert_eq!((indexed.sources, indexed.endpoints), (index.sources + 1, index.endpoints + 1));
    close(&host, document);
}

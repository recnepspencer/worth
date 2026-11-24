//! One envelope is one Native commit and one history step.
use super::*;
use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Bare;
use worth_relational::facade::inspection::observe_maintenance_work;

/// The same stages committed on the document branch outside its history move
/// the head behind the document: its next step refuses and records nothing.
#[test]
fn an_envelope_on_the_document_branch_diverges_the_next_history_edit() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let document = world::attach(&host);
    let before = observe(&host, &document);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope).on_branch(before.branch);
    let mut bare = request
        .edit_envelope()
        .idempotency("out-of-history".to_owned())
        .runtime_stages(&host);
    bare.push_stage::<ApplicationNoOutputGraph, _>(edit::Intent {
        key: "a".into(),
        value: 2,
    })
    .unwrap();
    let outcome = bare
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(outcome, Bare::Committed(_)), "{outcome:?}");
    drop(outcome);
    assert_eq!(world::model(&host, before.branch).value, 2);
    assert!(matches!(
        document.current_step(&host.request(&principal, &scope)),
        Err(WorthQueryApplicationHistoryDenial::Document(
            WorthQueryDocumentHistoryDenial::HeadDiverged
        ))
    ));
    let stages = Stages {
        key: "after-divergence",
        model: 3,
        member: 6,
    };
    let (document, denial) = refused(record_on(
        &host,
        document,
        &scope,
        before.branch,
        stages,
        metadata(b""),
    ));
    assert!(
        matches!(
            denial,
            Denial::History(WorthQueryApplicationHistoryDenial::Document(
                WorthQueryDocumentHistoryDenial::HeadDiverged
            ))
        ),
        "{denial:?}"
    );
    assert_eq!(world::model(&host, before.branch).value, 2);
    close(&host, document);
}

#[test]
fn a_two_stage_envelope_records_one_step() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let document = world::attach(&host);
    let before = observe(&host, &document);
    assert_eq!((before.model, before.member), (1, 5));
    let scope = request_scope();
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let stages = Stages {
        key: "two-stages",
        model: 2,
        member: 6,
    };
    let (execution, published) = observe_maintenance_work(8, || {
        record(&host, document, &scope, stages, metadata(b"step summary"))
    });
    let (document, step) = recorded(execution);
    assert_eq!(
        published.published_rows.len() + published.omitted_publications,
        1,
        "both stages are one Native commit"
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts,
        "stages without output roots contact no producer"
    );
    assert_eq!(step.parent(), Some(before.current));
    assert_eq!(step.label(), Some("Edit model"));
    assert_eq!(step.metadata().bytes(), b"step summary");
    let after = observe(&host, &document);
    assert_eq!((after.model, after.member), (2, 6));
    assert_eq!(after.current, step.id());
    assert_eq!(after.nodes.len(), before.nodes.len() + 1);
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let page = document
        .history_page(&host.request(&principal, &scope), limits(), None)
        .unwrap();
    let entry = page
        .rows()
        .iter()
        .map(|row| row.step())
        .find(|entry| entry.id() == step.id())
        .expect("the page shows the recorded step");
    assert_eq!(
        entry.parent(),
        WorthQueryHistoryParentBoundary::Retained(before.current)
    );
    assert_eq!(entry.label(), Some("Edit model"));
    assert_eq!(entry.metadata().bytes(), b"step summary");
    close(&host, document);
}

/// A recorded envelope leaves the head where the document expects it: the
/// next single edit and the next envelope both extend the same line.
#[test]
fn a_recorded_envelope_keeps_the_head_for_the_next_edit_and_envelope() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let first = Stages {
        key: "first",
        model: 2,
        member: 6,
    };
    let (document, envelope_step) = recorded(record(
        &host,
        world::attach(&host),
        &scope,
        first,
        metadata(b""),
    ));
    let mut edit = document
        .edit(edit::Intent {
            key: "a".into(),
            value: 3,
        })
        .idempotency_owned(&request, "single-edit".to_owned())
        .unwrap()
        .retain_in_program::<program::Program, program::EditAction>(&host)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    loop {
        edit = match edit.advance(&request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(next) => {
                edit = next;
                break;
            }
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => panic!("{denial:?}"),
        };
    }
    let WorthQueryHistoryEditFinish { document, outcome } = edit
        .finish(&request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    let edit_step = match &outcome {
        WorthQueryHistoryEditOutcome::Recorded { step, .. } => step.clone(),
        _ => panic!("the single edit must record"),
    };
    // The edit's live results retain its branch history until they are dropped.
    drop(outcome);
    assert_eq!(edit_step.parent(), Some(envelope_step.id()));
    let second = Stages {
        key: "second",
        model: 4,
        member: 7,
    };
    let (document, step) = recorded(record(&host, document, &scope, second, metadata(b"")));
    assert_eq!(step.parent(), Some(edit_step.id()));
    let after = observe(&host, &document);
    assert_eq!((after.model, after.member), (4, 7));
    close(&host, document);
}

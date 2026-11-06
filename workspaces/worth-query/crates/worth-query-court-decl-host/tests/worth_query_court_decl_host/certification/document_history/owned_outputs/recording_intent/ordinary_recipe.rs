//! A real ordinary producer fallback records the exact discovery it executes.
use super::{non_leaf::settle, non_leaf_program as installed, *};
fn exercise_ordinary(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host: WorthQueryProgramApplicationRuntime<Schema, installed::Program> =
        world::install_program_with_value(limits, 3);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    settle(&host, &request, false);
    let document = world::attach(&host);
    let before = document.current_step(&request).unwrap().id();
    let selection = document.selection(&request).unwrap();
    let at = host
        .request(&principal, &scope)
        .on_branch(selection.physical_endpoint().product_branch());
    let original_first = at
        .query(query::first_output::Read("first:a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    let original_child = at
        .query(query::second_output::Read("second:a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    let drift = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let document = host
        .with_host_advancement(|phase| {
            document
                .with_output_trees::<installed::Program, installed::Tree>(
                    &phase,
                    &request,
                    &host,
                    installed::Demand {
                        source: "a".into(),
                        discovery: std::sync::Arc::clone(&drift),
                    },
                    WorthQueryOutputDemandControls::default(),
                )
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
        })
        .unwrap();
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let mut edit = document
        .edit(edit::Intent {
            key: "a".into(),
            value: 5,
        })
        .idempotency_owned(&request, "ordinary-source-fallback-with-recipe".into())
        .unwrap()
        .retain_in_program::<installed::Program, program::EditAction>(&host)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    for count in 0..512 {
        assert!(count < 511, "ordinary recording must settle");
        edit = match edit.advance(&request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(done) => {
                edit = done;
                break;
            }
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => panic!("{denial:?}"),
        };
    }
    let document = edit
        .finish(&request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
        .document;
    let after = document.current_step(&request).unwrap().id();
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2,
        "both changed families take the real ordinary fallback"
    );
    drift.store(true, std::sync::atomic::Ordering::Relaxed);
    let reads = super::super::source_projection::Observation::begin();
    let discovery = installed::observation::Observation::begin();
    let selected = super::super::courts::checkout(
        &document,
        &request,
        before,
        document.selection(&request).unwrap(),
        0x1255_a1,
    )
    .unwrap();
    let selected =
        super::super::courts::checkout(&document, &request, after, selected, 0x1255_a2).unwrap();
    assert!(discovery.saw("a") && !discovery.saw("b"));
    assert_eq!(reads.reads(), 0, "selected healthy roots remain unopened");
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2
    );
    let at = host
        .request(&principal, &scope)
        .on_branch(selected.physical_endpoint().product_branch());
    let mut expected_first = original_first;
    expected_first.value = 50;
    let mut expected_child = original_child;
    expected_child.value = 1005;
    assert_eq!(
        at.query(query::first_output::Read("first:a".into()))
            .execute()
            .unwrap()
            .rows(),
        &[expected_first]
    );
    assert_eq!(
        at.query(query::second_output::Read("second:a".into()))
            .execute()
            .unwrap()
            .rows(),
        &[expected_child]
    );
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
#[test]
fn ordinary_recording_freezes_its_actual_discovery_request() {
    exercise_ordinary(false);
}
#[test]
fn worker_only_ordinary_recording_freezes_its_actual_discovery_request() {
    exercise_ordinary(true);
}

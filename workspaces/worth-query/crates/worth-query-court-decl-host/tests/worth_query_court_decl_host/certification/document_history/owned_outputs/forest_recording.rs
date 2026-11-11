//! One source edit records both configured roots before adopting its single history step.
use super::*;
fn exercise(worker: bool) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = if worker {
        world::install_worker_only()
    } else {
        world::install()
    };
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    first::settle(&host, &request, "a");
    second::settle(&host, &request, "a");
    let original = world::outputs(&host, host.current_world());
    let document = courts::configure_all_trees(&host, world::attach(&host));
    let before = document.current_step(&request).unwrap().id();
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let mut edit = document
        .edit(edit::Intent {
            key: "a".into(),
            value: 5,
        })
        .idempotency_owned(&request, "record-both-original-trees".into())
        .unwrap()
        .retain_in_program::<program::Program, program::EditAction>(&host)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    for count in 0..512 {
        assert!(
            count < 511,
            "recording did not finish: {:?}",
            edit.posture()
        );
        edit = match edit.advance(&request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(next) => {
                edit = next;
                break;
            }
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => {
                panic!("forest edit: {denial:?}")
            }
        };
    }
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
    let recorded = world::outputs(&host, future.physical_endpoint().product_branch());
    assert_eq!(
        (recorded.0.value, recorded.1.value),
        (50, 1005),
        "the recorded step includes both families at the edited source"
    );
    assert_eq!(
        (recorded.0.native_id, recorded.1.native_id),
        (original.0.native_id, original.1.native_id)
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2
    );
    let after_edit = host.producer_contacts_on_this_thread_for_test();
    let selected = courts::checkout(&document, &request, before, future.clone(), 0x57_f001)
        .unwrap_or_else(|cause| panic!("select original two-family snapshot: {cause:?}"));
    assert_eq!(
        world::outputs(&host, selected.physical_endpoint().product_branch()),
        original
    );
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), after_edit);
    let redone = courts::checkout(&document, &request, future.step().id(), selected, 0x57_f002)
        .unwrap_or_else(|cause| panic!("select recorded two-family snapshot: {cause:?}"));
    assert_eq!(
        world::outputs(&host, redone.physical_endpoint().product_branch()),
        recorded
    );
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), after_edit);
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
#[test]
fn source_edit_records_every_configured_tree_before_its_single_adoption() {
    exercise(false);
}
#[test]
fn worker_only_source_edit_records_every_configured_tree_before_its_single_adoption() {
    exercise(true);
}

fn omitted_family(worker: bool) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = if worker {
        world::install_worker_only()
    } else {
        world::install()
    };
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    first::settle(&host, &request, "a");
    second::settle(&host, &request, "a");
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
    let selection = document.selection(&request).unwrap();
    let branch = selection.physical_endpoint().product_branch();
    let model = world::model(&host, branch);
    let outputs = world::outputs(&host, branch);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let mut edit = document
        .edit(edit::Intent {
            key: "a".into(),
            value: 5,
        })
        .idempotency_owned(&request, "record-incomplete-forest".into())
        .unwrap()
        .retain_in_program::<program::Program, program::EditAction>(&host)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    for count in 0..512 {
        assert!(count < 511, "incomplete recording did not end");
        edit = match edit.advance(&request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(_) => panic!("omitted stale family was adopted"),
            WorthQueryHistoryEditAdvance::Refused {
                denial,
                edit: owner,
            } => {
                assert!(
                    matches!(denial, WorthQueryHistoryEditDenial::Outputs(
                    WorthQueryHistoryOutputDenial::RecordingQualification(
                        WorthQueryRecordingOutputQualificationDenial::Incomplete {model: subject}
                    )) if subject == model.native_id),
                    "{denial:?}"
                );
                let document = owner
                    .discard(&request)
                    .unwrap_or_else(|(_, cause)| panic!("discard: {cause:?}"));
                assert_eq!(document.selection(&request).unwrap(), selection);
                assert_eq!(world::model(&host, branch), model);
                assert_eq!(world::outputs(&host, branch), outputs);
                assert_eq!(
                    host.producer_contacts_on_this_thread_for_test(),
                    contacts + 1
                );
                host.with_host_advancement(|phase| document.close(&phase).unwrap())
                    .unwrap();
                return;
            }
        };
    }
}
#[test]
fn recording_refuses_an_omitted_stale_native_family_without_adopting_partial_outputs() {
    omitted_family(false);
}
#[test]
fn worker_only_recording_refuses_an_omitted_stale_native_family_without_adopting_partial_outputs() {
    omitted_family(true);
}

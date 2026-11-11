//! Every complete set is checked again at the one final recording endpoint.
use super::*;
fn exercise(worker: bool, upstream_first: bool, earlier_stale: bool) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = if worker {
        world::install_worker_only()
    } else {
        world::install()
    };
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    upstream::settle(&host, &request, "a");
    downstream::settle(&host, &request, "upstream:a");
    if earlier_stale {
        dependent_forest::advance_model_before_attachment(&host, &request);
        upstream::settle(&host, &request, "a");
    }
    let document = dependent_forest::configured(&host, world::attach(&host), upstream_first);
    let selection = document.selection(&request).unwrap();
    let branch = selection.physical_endpoint().product_branch();
    let before = dependent_forest::values(&host, branch);
    let model = world::model(&host, branch);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let mut edit = document
        .edit(edit::Intent {
            key: "a".into(),
            value: 3,
        })
        .idempotency_owned(&request, "record-dependent-forest".into())
        .unwrap()
        .retain_in_program::<program::Program, program::EditAction>(&host)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    for count in 0..512 {
        assert!(count < 511, "dependent recording did not finish");
        edit = match edit.advance(&request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(done) => {
                assert!(
                    upstream_first,
                    "a later tree invalidated the earlier family"
                );
                let finish = done
                    .finish(&request)
                    .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
                let document = finish.document;
                drop(finish.outcome);
                let selected = document.selection(&request).unwrap();
                let after =
                    dependent_forest::values(&host, selected.physical_endpoint().product_branch());
                assert_eq!((after.0.value, after.1.value), (15, 1500));
                assert_eq!(
                    (after.0.native_id, after.1.native_id),
                    (before.0.native_id, before.1.native_id)
                );
                assert_eq!(
                    host.producer_contacts_on_this_thread_for_test(),
                    contacts + 2
                );
                host.with_host_advancement(|phase| document.close(&phase).unwrap())
                    .unwrap();
                return;
            }
            WorthQueryHistoryEditAdvance::Refused {
                edit: owner,
                denial,
            } => {
                assert!(!upstream_first, "ordered recording refused: {denial:?}");
                let expected = if earlier_stale {
                    matches!(
                        &denial,
                        WorthQueryHistoryEditDenial::Outputs(
                            WorthQueryHistoryOutputDenial::RecordingQualification(
                                WorthQueryRecordingOutputQualificationDenial::Original(
                                    WorthQueryOriginalReadOutputDenial::Currentness(_)
                                )
                            )
                        )
                    )
                } else {
                    matches!(&denial, WorthQueryHistoryEditDenial::Outputs(WorthQueryHistoryOutputDenial::RecordingQualification(
                        WorthQueryRecordingOutputQualificationDenial::TreeChanged{source,family})) if *source==before.0.native_id && *family=="downstream")
                };
                assert!(expected, "{denial:?}");
                let document = owner
                    .discard(&request)
                    .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
                assert_eq!(document.selection(&request).unwrap(), selection);
                assert_eq!(world::model(&host, branch), model);
                assert_eq!(dependent_forest::values(&host, branch), before);
                assert_eq!(
                    host.producer_contacts_on_this_thread_for_test(),
                    contacts + if earlier_stale { 2 } else { 1 }
                );
                host.with_host_advancement(|phase| document.close(&phase).unwrap())
                    .unwrap();
                return;
            }
        };
    }
}
#[test]
fn recording_later_roots_read_the_progressed_candidate() {
    exercise(false, true, true);
}
#[test]
fn recording_refuses_a_later_tree_invalidating_an_earlier_family() {
    exercise(false, false, true);
}
#[test]
fn worker_only_recording_later_roots_read_the_progressed_candidate() {
    exercise(true, true, true);
}
#[test]
fn worker_only_recording_refuses_a_later_tree_invalidating_an_earlier_family() {
    exercise(true, false, true);
}

#[test]
fn recording_rechecks_a_skipped_healthy_leaf_after_a_later_tree_changes_its_source() {
    exercise(false, false, false);
}
#[test]
fn worker_only_recording_rechecks_a_skipped_healthy_leaf_after_a_later_tree_changes_its_source() {
    exercise(true, false, false);
}

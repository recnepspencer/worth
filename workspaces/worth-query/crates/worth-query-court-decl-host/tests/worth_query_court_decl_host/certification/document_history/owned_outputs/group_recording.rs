//! One homogeneous or heterogeneous gesture records both real output families.
use super::*;
type Mode = WorthQueryHistoryWithOutputTrees<Schema>;
fn finish<'r, M>(
    host: &world::Host,
    request: &family::Request<'_, '_, '_>,
    mut group: WorthQueryGroupedHistoryDocument<
        'r,
        Schema,
        History,
        program::Program,
        M,
        OrdinaryHistoryPayload,
        Mode,
    >,
    original: (query::Row, query::Row),
    initial: WorthQueryHistorySelection,
    contacts: u64,
) where
    M: WorthQueryHistoryGroupMembers<Schema, History, program::Program>,
{
    let mut qualified = false;
    for count in 0..512 {
        assert!(
            count < 511,
            "finite group must qualify: {:?}",
            group.posture()
        );
        let (document, execution) = group.parts();
        assert_eq!(document.selection(request).unwrap(), initial);
        let posture = execution
            .advance(document, request)
            .unwrap_or_else(|cause| panic!("{cause:?}"));
        if posture == WorthQueryHistoryGroupPosture::QualifyingOutputs {
            qualified = true;
        }
        if posture == WorthQueryHistoryGroupPosture::ReadyToComplete {
            assert_eq!(
                world::outputs(host, initial.physical_endpoint().product_branch()),
                original,
                "no final output is public before the group's one adoption"
            );
            let step = execution
                .complete(document, request)
                .unwrap_or_else(|cause| panic!("{cause:?}"));
            assert_eq!(step.parent(), Some(initial.step().id()));
            break;
        }
    }
    assert!(
        qualified,
        "the opted-in group traverses final output qualification"
    );
    let finished = group
        .finish(request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    assert_eq!(finished.completions.len(), 2);
    let document = finished.document;
    drop(finished.completions);
    let future = document.selection(request).unwrap();
    let outputs = world::outputs(host, future.physical_endpoint().product_branch());
    assert_eq!((outputs.0.value, outputs.1.value), (70, 1007));
    assert_eq!(
        (outputs.0.native_id, outputs.1.native_id),
        (original.0.native_id, original.1.native_id)
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2
    );
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let reads = source_projection::Observation::begin();
    let selected = courts::checkout(
        &document,
        request,
        initial.step().id(),
        future.clone(),
        0x57_a001,
    )
    .unwrap_or_else(|cause| panic!("{cause:?}"));
    assert_eq!(
        world::outputs(host, selected.physical_endpoint().product_branch()),
        original
    );
    let selected = courts::checkout(&document, request, future.step().id(), selected, 0x57_a002)
        .unwrap_or_else(|cause| panic!("{cause:?}"));
    assert_eq!(
        world::outputs(host, selected.physical_endpoint().product_branch()),
        outputs
    );
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    assert_eq!(reads.reads(), 0, "healthy selected roots remain unopened");
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
fn exercise(worker: bool, mixed: bool) {
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
    let initial = document.selection(&request).unwrap();
    let contacts = host.producer_contacts_on_this_thread_for_test();
    if mixed {
        let mut group = document
            .gesture(&host, &request, 0x57_a003)
            .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
        group
            .push::<_, program::EditAction>(
                &request,
                edit::Intent {
                    key: "a".into(),
                    value: 7,
                },
                "model".into(),
            )
            .unwrap_or_else(|cause| panic!("{cause:?}"));
        group
            .push::<_, program::MemberEditAction>(
                &request,
                member_edit::Intent {
                    key: "m".into(),
                    value: 8,
                },
                "member".into(),
            )
            .unwrap_or_else(|cause| panic!("{cause:?}"));
        finish(&host, &request, group, original, initial, contacts);
    } else {
        let group = document
            .group_edits::<_, program::Program, program::EditAction, 2>(
                &host,
                &request,
                0x57_a004,
                [
                    (
                        edit::Intent {
                            key: "a".into(),
                            value: 5,
                        },
                        "first".into(),
                    ),
                    (
                        edit::Intent {
                            key: "a".into(),
                            value: 7,
                        },
                        "last".into(),
                    ),
                ],
            )
            .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
        finish(&host, &request, group, original, initial, contacts);
    }
}
#[test]
fn a_group_records_both_output_families_before_one_adoption() {
    exercise(false, false);
}
#[test]
fn worker_only_a_group_records_both_output_families_before_one_adoption() {
    exercise(true, false);
}
#[test]
fn a_mixed_gesture_records_both_output_families_before_one_adoption() {
    exercise(false, true);
}
#[test]
fn worker_only_a_mixed_gesture_records_both_output_families_before_one_adoption() {
    exercise(true, true);
}

//! Only the stale independent family opens its original source and tree.
use super::*;
fn exercise(host: &world::Host) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    first::settle(host, &request, "a");
    downstream::settle(host, &request, "m");
    let document = world::attach(host);
    let mut edit = document
        .edit(member_edit::Intent {
            key: "m".into(),
            value: 6,
        })
        .idempotency_owned(&request, "mixed-family-source-edit".to_owned())
        .unwrap()
        .retain_in_program::<program::Program, program::MemberEditAction>(host)
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
    assert!(matches!(
        outcome,
        WorthQueryHistoryEditOutcome::Recorded { .. }
    ));
    drop(outcome);
    let step = document.current_step(&request).unwrap().id();
    let current = document.selection(&request).unwrap();
    let before = outputs(host, current.physical_endpoint().product_branch());
    assert_eq!((before.0.value, before.1.value), (10, 500));
    let projection = source_projection::Observation::begin();
    let document = host
        .with_host_advancement(|phase| {
            document
                .with_output_trees::<program::Program, program::FirstTree>(
                    &phase,
                    &request,
                    host,
                    first::Demand("a".into()),
                    WorthQueryOutputDemandControls::default(),
                )
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
                .with_output_tree::<program::Program, program::DownstreamTree>(
                    &phase,
                    &request,
                    host,
                    downstream::Demand("m".into()),
                    WorthQueryOutputDemandControls::default(),
                )
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
        })
        .unwrap();
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let selected = courts::checkout(&document, &request, step, current, 0xf057_09)
        .unwrap_or_else(|cause| panic!("{cause:?}"));
    let after = outputs(host, selected.physical_endpoint().product_branch());
    assert_eq!((after.0.value, after.1.value), (10, 600));
    assert_eq!(
        (after.0.native_id, after.1.native_id),
        (before.0.native_id, before.1.native_id)
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 1
    );
    assert_eq!(
        projection.reads(),
        0,
        "the healthy first family must not open its source or root"
    );
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
fn outputs(
    host: &world::Host,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
) -> (query::Row, query::Row) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope).on_branch(branch);
    (
        request
            .query(query::first_output::Read("first:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone(),
        request
            .query(query::child_output::Read("downstream:m".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone(),
    )
}
#[test]
fn a_stale_family_falls_back_without_opening_a_healthy_sibling_tree() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    exercise(&world::install());
}
#[test]
fn worker_only_mixed_families_keep_healthy_roots_closed() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    exercise(&world::install_worker_only());
}

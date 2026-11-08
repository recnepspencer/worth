//! Public navigation discovers a real dependent without admitting a root demand.
use super::*;
type Host = WorthQueryProgramApplicationRuntime<Schema, program::Program>;
fn exercise(worker_only: bool, initial: u64, stale: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker_only {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host: Host = world::install_program_with_value(limits, initial);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut outputs = request
        .demand(first::Demand("a".into()))
        .retain_program_outputs::<program::Program, program::Tree>(&host)
        .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
    loop {
        outputs = match outputs.advance(&request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
            WorthQueryProgramOutputOperationAdvance::Settled(done) => {
                done.deliver_current(&request).unwrap();
                done.close(&request)
                    .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
                break;
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            WorthQueryProgramOutputOperationAdvance::Terminal(_) => {
                panic!("initial tree terminated")
            }
        };
    }
    if initial == 1 {
        let mut seed = request
            .demand(second::Demand("a".into()))
            .retain_program_outputs::<program::Program, program::SeedTree>(&host)
            .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
        loop {
            seed = match seed.advance(&request) {
                WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
                WorthQueryProgramOutputOperationAdvance::Settled(done) => {
                    done.deliver_current(&request).unwrap();
                    done.close(&request)
                        .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
                    break;
                }
                WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => {
                    panic!("{denial:?}")
                }
                WorthQueryProgramOutputOperationAdvance::Terminal(_) => panic!("seed terminated"),
            };
        }
    }
    assert_eq!(
        request
            .query(query::first_discovery::Read("first:a".into()))
            .execute()
            .unwrap()
            .rows()
            .len(),
        1,
        "the selected D really grants discovery"
    );
    if stale {
        source_edits::model(
            &host,
            &request,
            initial + 2,
            "stale-publications-before-history",
        );
    }
    let document = world::attach(&host);
    let root = document.current_step(&request).unwrap().id();
    let document = host
        .with_host_advancement(|phase| {
            document
                .with_output_trees::<program::Program, program::Tree>(
                    &phase,
                    &request,
                    &host,
                    first::Demand("a".into()),
                    WorthQueryOutputDemandControls::default(),
                )
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
        })
        .unwrap();
    macro_rules! record {
        ($input:expr, $action:ty) => {{
            let mut edit = document
                .edit($input)
                .idempotency_owned(&request, "hierarchy-qualified-edit".into())
                .unwrap()
                .retain_in_program::<program::Program, $action>(&host)
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
            for count in 0..512 {
                assert!(count < 511, "qualified recording must complete");
                edit = match edit.advance(&request) {
                    WorthQueryHistoryEditAdvance::Progress(next) => next,
                    WorthQueryHistoryEditAdvance::Ready(done) => {
                        edit = done;
                        break;
                    }
                    WorthQueryHistoryEditAdvance::Refused { denial, .. } => panic!("{denial:?}"),
                };
            }
            edit.finish(&request)
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
                .document
        }};
    }
    let document = if stale {
        record!(
            member_edit::Intent {
                key: "m".into(),
                value: 6
            },
            super::super::program::MemberEditAction
        )
    } else {
        record!(
            edit::Intent {
                key: "a".into(),
                value: initial + 2
            },
            super::super::program::EditAction
        )
    };
    let future = document.current_step(&request).unwrap().id();
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let observation = source_projection::Observation::begin();
    let selected = super::super::courts::checkout(
        &document,
        &request,
        root,
        document.selection(&request).unwrap(),
        0xd1214_01,
    )
    .unwrap_or_else(|cause| panic!("non-leaf selection: {cause:?}"));
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + if stale { 2 } else { 0 }
    );
    assert!(
        observation.discovery_reads() > 0,
        "installed discovery must run"
    );
    if stale {
        assert!(
            observation.reads() > 0,
            "the stale parent opens its actual source"
        );
    }
    if initial == 1 {
        assert_eq!(
            observation.reads(),
            0,
            "empty discovery never borrows the Native roster's child"
        );
    }
    let restored = if stale { initial + 2 } else { initial };
    let at = host
        .request(&principal, &scope)
        .on_branch(selected.physical_endpoint().product_branch());
    assert_eq!(
        at.query(query::first_output::Read("first:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        restored * 10
    );
    assert_eq!(
        at.query(query::second_output::Read("second:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        restored + 1000
    );
    drop(observation);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let selected =
        super::super::courts::checkout(&document, &request, future, selected, 0xd1214_02).unwrap();
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts,
        "the newly recorded future already has complete outputs"
    );
    let at = host
        .request(&principal, &scope)
        .on_branch(selected.physical_endpoint().product_branch());
    assert_eq!(
        at.query(query::first_output::Read("first:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        (initial + 2) * 10
    );
    assert_eq!(
        at.query(query::second_output::Read("second:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        initial + 1002
    );
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
#[test]
fn healthy_non_leaf_discovers_across_complete_recorded_steps() {
    exercise(false, 3, false);
}
#[test]
fn worker_only_healthy_non_leaf_discovers_across_complete_recorded_steps() {
    exercise(true, 3, false);
}
#[test]
fn a_preexisting_stale_parent_and_child_take_real_navigation_fallback() {
    exercise(false, 3, true);
}
#[test]
fn worker_only_a_preexisting_stale_parent_and_child_take_real_navigation_fallback() {
    exercise(true, 3, true);
}
#[test]
fn empty_discovery_is_not_replaced_by_the_native_roster() {
    exercise(false, 1, false);
}
#[test]
fn worker_only_empty_discovery_is_not_replaced_by_the_native_roster() {
    exercise(true, 1, false);
}

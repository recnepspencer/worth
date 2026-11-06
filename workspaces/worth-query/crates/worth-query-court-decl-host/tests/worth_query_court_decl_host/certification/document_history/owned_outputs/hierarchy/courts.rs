//! Public navigation discovers a real dependent without admitting a root demand.
use super::*;
type Host = WorthQueryProgramApplicationRuntime<Schema, program::Program>;
fn exercise(worker_only: bool, initial: u64, denied: bool) {
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
    let document = world::attach(&host);
    let root = document.current_step(&request).unwrap().id();
    let mut edit = document
        .edit(edit::Intent {
            key: "a".into(),
            value: if denied { 2 } else { initial + 2 },
        })
        .idempotency_owned(&request, "hierarchy-source-edit".into())
        .unwrap()
        .retain_in_program::<program::Program, super::super::program::EditAction>(&host)
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
    let future = document.current_step(&request).unwrap().id();
    let expected = document.selection(&request).unwrap();
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
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let observation = source_projection::Observation::begin();
    if denied {
        let denial =
            super::super::courts::checkout(&document, &request, root, expected.clone(), 0xd1214_03)
                .expect_err("D must not grant discovery when C denies it");
        let (
            "advance",
            WorthQueryHistoryNavigationAdvanceDenial::Outputs(
                WorthQueryHistoryOutputDenial::Preparation(
                    WorthQueryRequiredOutputPreparationDenial::SourceQuery(
                        WorthQueryApplicationRequestQueryDenial::Admission(cause),
                    ),
                ),
            ),
        ) = denial
        else {
            panic!("{denial:?}");
        };
        assert!(
            matches!(
                cause.kind(),
                WorthQueryApplicationQueryAdmissionDenialKind::Authorization(_)
            ),
            "{cause:?}"
        );
        assert_eq!(document.selection(&request).unwrap(), expected);
        assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
        assert_eq!(
            observation.discovery_reads(),
            0,
            "no D disclosure before C authorization"
        );
        drop(observation);
        host.with_host_advancement(|phase| document.close(&phase).unwrap())
            .unwrap();
        return;
    }
    let selected = super::super::courts::checkout(&document, &request, root, expected, 0xd1214_01)
        .unwrap_or_else(|cause| panic!("healthy non-leaf: {cause:?}"));
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    assert!(
        observation.discovery_reads() > 0,
        "the installed discovery must run"
    );
    if initial == 1 {
        assert_eq!(
            observation.reads(),
            0,
            "an empty computed set opens no child source despite the Native second-family route"
        );
    }
    drop(observation);
    let branch = selected.physical_endpoint().product_branch();
    let child = host
        .request(&principal, &scope)
        .on_branch(branch)
        .query(query::second_output::Read("second:a".into()))
        .execute()
        .unwrap();
    assert_eq!(child.rows().len(), 1);
    assert_eq!(child.rows()[0].value, initial + 1000);
    let selected =
        super::super::courts::checkout(&document, &request, future, selected, 0xd1214_02)
            .unwrap_or_else(|cause| panic!("stale non-leaf fallback: {cause:?}"));
    let parent = host
        .request(&principal, &scope)
        .on_branch(selected.physical_endpoint().product_branch())
        .query(query::first_output::Read("first:a".into()))
        .execute()
        .unwrap();
    assert_eq!(
        parent.rows()[0].value,
        (initial + 2) * 10,
        "the stale parent itself must regenerate"
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2
    );
    let child = host
        .request(&principal, &scope)
        .on_branch(selected.physical_endpoint().product_branch())
        .query(query::second_output::Read("second:a".into()))
        .execute()
        .unwrap();
    assert_eq!(child.rows()[0].value, initial + 1002);
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
#[test]
fn healthy_non_leaf_discovers_and_stale_parent_falls_back() {
    exercise(false, 3, false);
}
#[test]
fn worker_only_healthy_non_leaf_discovers_and_stale_parent_falls_back() {
    exercise(true, 3, false);
}

#[test]
fn empty_discovery_is_not_replaced_by_the_native_roster() {
    exercise(false, 1, false);
}
#[test]
fn worker_only_empty_discovery_is_not_replaced_by_the_native_roster() {
    exercise(true, 1, false);
}
#[test]
fn historical_discovery_cannot_borrow_a_d_grant() {
    exercise(false, 1, true);
}
#[test]
fn worker_only_historical_discovery_cannot_borrow_a_d_grant() {
    exercise(true, 1, true);
}

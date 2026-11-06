//! Public navigation discovers a real dependent without admitting a root demand.
use super::*;
type Host = WorthQueryProgramApplicationRuntime<Schema, program::Program>;
fn exercise(worker_only: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker_only {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host: Host = world::install_program_with_value(limits, 3);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut outputs = request
        .demand(program::Demand {
            key: "a".into(),
            expected: "a".into(),
        })
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
            value: 5,
        })
        .idempotency_owned(&request, "hierarchy-source-edit".into())
        .unwrap()
        .retain_in_program::<program::Program, super::super::super::program::EditAction>(&host)
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
    let expected = document.selection(&request).unwrap();
    let document = host
        .with_host_advancement(|phase| {
            document
                .with_output_trees::<program::Program, program::Tree>(
                    &phase,
                    &request,
                    &host,
                    program::Demand {
                        key: "a".into(),
                        expected: "absent".into(),
                    },
                    WorthQueryOutputDemandControls::default(),
                )
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
        })
        .unwrap();
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let denial = super::super::super::courts::checkout(&document, &request, root, expected.clone(), 0xd1214_10)
        .expect_err("the configured source partition has no row at D; another partition cannot certify its parent");
    assert!(
        matches!(
            denial,
            (
                "advance",
                WorthQueryHistoryNavigationAdvanceDenial::Outputs(_)
            )
        ),
        "{denial:?}"
    );
    assert_eq!(document.selection(&request).unwrap(), expected);
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts,
        "a missing source cannot contact a producer"
    );
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
#[test]
fn another_parameter_partition_cannot_certify_a_history_parent() {
    exercise(false);
}
#[test]
fn worker_only_another_parameter_partition_cannot_certify_a_history_parent() {
    exercise(true);
}

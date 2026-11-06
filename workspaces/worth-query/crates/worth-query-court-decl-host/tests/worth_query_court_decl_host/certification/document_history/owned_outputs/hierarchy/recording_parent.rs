//! A historical parent remains a real discovery source while recording a new future.
use super::*;
type Host = WorthQueryProgramApplicationRuntime<Schema, program::Program>;
fn settle(host: &Host, request: &family::Request<'_, '_, '_>) {
    let mut tree = request
        .demand(first::Demand("a".into()))
        .retain_program_outputs::<program::Program, program::Tree>(host)
        .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
    for count in 0..128 {
        assert!(count < 127, "original tree did not settle");
        tree = match tree.advance(request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
            WorthQueryProgramOutputOperationAdvance::Settled(done) => {
                done.deliver_current(request).unwrap();
                done.close(request)
                    .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
                return;
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            WorthQueryProgramOutputOperationAdvance::Terminal(_) => panic!("tree terminated"),
        };
    }
}
fn exercise(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host: Host = world::install_program_with_value(limits, 3);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    settle(&host, &request);
    let original_first = request
        .query(query::first_output::Read("first:a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    let original_child = request
        .query(query::second_output::Read("second:a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    assert_eq!((original_first.value, original_child.value), (30, 1003));
    let document = world::attach(&host);
    let before = document.current_step(&request).unwrap().id();
    macro_rules! record {
        ($doc:expr,$input:expr,$action:ty,$key:literal) => {{
            let mut edit = $doc
                .edit($input)
                .idempotency_owned(&request, $key.into())
                .unwrap()
                .retain_in_program::<program::Program, $action>(&host)
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
            for count in 0..512 {
                assert!(count < 511, "recording did not progress");
                edit = match edit.advance(&request) {
                    WorthQueryHistoryEditAdvance::Progress(next) => next,
                    WorthQueryHistoryEditAdvance::Ready(next) => {
                        edit = next;
                        break;
                    }
                    WorthQueryHistoryEditAdvance::Refused { denial, .. } => {
                        panic!("non-leaf recording: {denial:?}")
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
            let doc = finish.document;
            drop(finish.outcome);
            doc
        }};
    }
    let document = record!(
        document,
        edit::Intent {
            key: "a".into(),
            value: 5
        },
        super::super::program::EditAction,
        "non-leaf-stale-future"
    );
    let selection = document.selection(&request).unwrap();
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
    let undone =
        super::super::courts::checkout(&document, &request, before, selection, 0x57_fa01).unwrap();
    let at_c = host
        .request(&principal, &scope)
        .on_branch(undone.physical_endpoint().product_branch());
    settle(&host, &at_c);
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    let reads = source_projection::Observation::begin();
    let document = record!(
        document,
        member_edit::Intent {
            key: "m".into(),
            value: 6
        },
        super::super::program::MemberEditAction,
        "non-leaf-future-after-undo"
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts,
        "healthy restored parent and child never contact producers"
    );
    assert_eq!(reads.reads(), 0, "healthy original sources remain unopened");
    assert!(
        reads.discovery_reads() > 0,
        "actual installed discovery determines the child set"
    );
    let selected = document.selection(&request).unwrap();
    let at = host
        .request(&principal, &scope)
        .on_branch(selected.physical_endpoint().product_branch());
    assert_eq!(
        at.query(query::first_output::Read("first:a".into()))
            .execute()
            .unwrap()
            .rows(),
        &[original_first]
    );
    assert_eq!(
        at.query(query::second_output::Read("second:a".into()))
            .execute()
            .unwrap()
            .rows(),
        &[original_child]
    );
    assert_eq!(
        at.query(query::member_public::Read("m".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        6
    );
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
#[test]
fn recording_a_future_uses_the_verified_historical_parent_and_real_discovery() {
    exercise(false);
}
#[test]
fn worker_only_recording_a_future_uses_the_verified_historical_parent_and_real_discovery() {
    exercise(true);
}

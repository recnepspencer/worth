//! Discovery runs with each recorded step's immutable canonical request recipe.
use super::non_leaf_program as installed;
use super::*;
type Host = WorthQueryProgramApplicationRuntime<Schema, installed::Program>;
pub(super) fn settle(host: &Host, request: &family::Request<'_, '_, '_>, b: bool) {
    let mut tree = request
        .demand(installed::Demand {
            source: if b { "b" } else { "a" }.into(),
            discovery: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(b)),
        })
        .retain_program_outputs::<installed::Program, installed::Tree>(host)
        .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
    for count in 0..128 {
        assert!(count < 127, "ordinary setup tree must settle");
        tree = match tree.advance(request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
            WorthQueryProgramOutputOperationAdvance::Settled(done) => {
                done.deliver_current(request).unwrap();
                done.close(request)
                    .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
                return;
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            WorthQueryProgramOutputOperationAdvance::Terminal(_) => panic!("setup terminated"),
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
    let host: Host = world::install_program_with_other_model(limits, 3, Some(99));
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    settle(&host, &request, false);
    settle(&host, &request, true);
    let mut change = request
        .mutate(edit::Intent {
            key: "b".into(),
            value: 7,
        })
        .without_source()
        .idempotency_owned("stale-b-before-recording-a".to_owned())
        .unwrap_or_else(|(_, _, cause)| panic!("{cause:?}"))
        .retain_operation::<installed::Program, program::EditAction>(&host)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    for count in 0..128 {
        assert!(count < 127, "ordinary B edit must finish");
        change = match change.advance(&request) {
            WorthQuerySourceOnlyAdvance::Progress(next) => next,
            WorthQuerySourceOnlyAdvance::Completed(done) => {
                done.close(&request)
                    .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
                break;
            }
            WorthQuerySourceOnlyAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            WorthQuerySourceOnlyAdvance::Terminal(_) => panic!("B edit terminated"),
        };
    }
    let document = world::attach(&host);
    let selection = document.selection(&request).unwrap();
    let at_c = host
        .request(&principal, &scope)
        .on_branch(selection.physical_endpoint().product_branch());
    let b_before = at_c
        .query(query::second_output::Read("second:b".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    assert_eq!(b_before.value, 1099);
    let root = at_c
        .query(query::model::Read("a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    let root_id = root.native_id;
    let root_contacts = host.producer_contacts_at_root_on_this_thread_for_test(root_id);
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

    macro_rules! record_member {
        ($document:expr, $value:expr, $key:expr) => {{
            let mut edit = $document
                .edit(member_edit::Intent {
                    key: "m".into(),
                    value: $value,
                })
                .idempotency_owned(&request, $key.into())
                .unwrap()
                .retain_in_program::<installed::Program, program::MemberEditAction>(&host)
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
            for count in 0..512 {
                assert!(count < 511, "recording must finish");
                edit = match edit.advance(&request) {
                    WorthQueryHistoryEditAdvance::Progress(next) => next,
                    WorthQueryHistoryEditAdvance::Ready(done) => {
                        edit = done;
                        break;
                    }
                    WorthQueryHistoryEditAdvance::Refused { denial, .. } => panic!("{denial:?}"),
                };
            }
            let finish = edit
                .finish(&request)
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
            assert!(matches!(
                finish.outcome,
                WorthQueryHistoryEditOutcome::Recorded { .. }
            ));
            finish.document
        }};
    }
    let before = document.current_step(&request).unwrap().id();
    let document = record_member!(document, 6, "record-a-recipe");
    drift.store(true, std::sync::atomic::Ordering::Relaxed);
    let unchanged = installed::Demand {
        source: "a".into(),
        discovery: std::sync::Arc::clone(&drift),
    }
    .source_intent();
    assert_eq!(
        (unchanged.key.as_str(), unchanged.expected.as_str()),
        ("a", "a")
    );
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let reads = super::super::source_projection::Observation::begin();
    let discovered = installed::observation::Observation::begin();
    let selected = super::super::courts::checkout(
        &document,
        &request,
        before,
        document.selection(&request).unwrap(),
        0x1255_01,
    )
    .unwrap();
    assert_eq!(reads.reads(), 0, "the healthy root remains unopened");
    assert!(discovered.saw("a"));
    assert!(
        !discovered.saw("b"),
        "the live shared field must not redirect the selected step"
    );
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    let at = host
        .request(&principal, &scope)
        .on_branch(selected.physical_endpoint().product_branch());
    assert_eq!(
        at.query(query::second_output::Read("second:b".into()))
            .execute()
            .unwrap()
            .rows(),
        &[b_before]
    );
    assert_eq!(
        at.query(query::model::Read("a".into()))
            .execute()
            .unwrap()
            .rows(),
        &[root]
    );
    drop((reads, discovered));

    // A new ordinary recording uses the new live recipe. Forward navigation
    // later must use that step's B recipe even after the caller switches to A.
    let discovered = installed::observation::Observation::begin();
    let document = record_member!(document, 7, "record-new-b-recipe");
    let forward = document.current_step(&request).unwrap().id();
    assert!(discovered.saw("b"));
    assert!(!discovered.saw("a"));
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 1
    );
    drop(discovered);
    drift.store(false, std::sync::atomic::Ordering::Relaxed);
    let selected = super::super::courts::checkout(
        &document,
        &request,
        before,
        document.selection(&request).unwrap(),
        0x1255_02,
    )
    .unwrap();
    let discovered = installed::observation::Observation::begin();
    let selected =
        super::super::courts::checkout(&document, &request, forward, selected, 0x1255_03).unwrap();
    assert!(discovered.saw("b"));
    assert!(!discovered.saw("a"));
    let at = host
        .request(&principal, &scope)
        .on_branch(selected.physical_endpoint().product_branch());
    assert_eq!(
        at.query(query::second_output::Read("second:b".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        1007
    );
    assert_eq!(
        host.producer_contacts_at_root_on_this_thread_for_test(root_id),
        root_contacts,
        "the healthy A roots stay unopened, including the new step's foreign B discovery"
    );
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
#[test]
fn a_selected_non_leaf_step_uses_its_frozen_discovery_recipe() {
    exercise(false);
}
#[test]
fn worker_only_a_selected_non_leaf_step_uses_its_frozen_discovery_recipe() {
    exercise(true);
}

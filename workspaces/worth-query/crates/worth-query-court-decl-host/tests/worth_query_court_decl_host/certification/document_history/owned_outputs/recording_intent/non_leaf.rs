//! A retained A source intent does not freeze the opaque discovery input.
use super::non_leaf_program as installed;
use super::*;
type Host = WorthQueryProgramApplicationRuntime<Schema, installed::Program>;
fn settle(host: &Host, request: &family::Request<'_, '_, '_>, b: bool) {
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
    drift.store(true, std::sync::atomic::Ordering::Relaxed);
    let unchanged_source = installed::Demand {
        source: "a".into(),
        discovery: std::sync::Arc::clone(&drift),
    }
    .source_intent();
    assert_eq!(unchanged_source.key, "a");
    assert_eq!(
        unchanged_source.expected, "a",
        "the complete root source input did not change"
    );

    let contacts = host.producer_contacts_on_this_thread_for_test();
    let mut edit = document
        .edit(member_edit::Intent {
            key: "m".into(),
            value: 6,
        })
        .idempotency_owned(&request, "record-a-with-drifted-parent".into())
        .unwrap()
        .retain_in_program::<installed::Program, program::MemberEditAction>(&host)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    for count in 0..512 {
        assert!(count < 511, "drift must refuse");
        edit = match edit.advance(&request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(done) => {
                let finish = done
                    .finish(&request)
                    .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
                let selected = finish.document.selection(&request).unwrap();
                let at = host
                    .request(&principal, &scope)
                    .on_branch(selected.physical_endpoint().product_branch());
                let after = at
                    .query(query::second_output::Read("second:b".into()))
                    .execute()
                    .unwrap()
                    .rows()[0]
                    .clone();
                panic!("A recorded a drifted B child: before={b_before:?}, after={after:?}, additional_contacts={}",host.producer_contacts_on_this_thread_for_test()-contacts);
            }
            WorthQueryHistoryEditAdvance::Refused {
                edit: owner,
                denial,
            } => {
                let WorthQueryHistoryEditDenial::Outputs(WorthQueryHistoryOutputDenial::Source(
                    WorthQueryHistoryOutputReadDenial::Demand(cause),
                )) = &denial
                else {
                    panic!("wrong frozen root refusal: {denial:?}")
                };
                assert!(
                    matches!(
                        cause.kind(),
                        WorthQueryOutputDemandDenialKind::ForeignSource
                    ),
                    "{cause:?}"
                );
                assert!(
                    cause.subject().contains(&format!("{:?}", root.native_id)),
                    "{cause:?}"
                );
                assert!(cause.subject().contains("first"), "{cause:?}");
                assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
                let document = owner
                    .discard(&request)
                    .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
                assert_eq!(document.selection(&request).unwrap(), selection);
                assert_eq!(
                    at_c.query(query::second_output::Read("second:b".into()))
                        .execute()
                        .unwrap()
                        .rows(),
                    &[b_before]
                );
                host.with_host_advancement(|phase| document.close(&phase).unwrap())
                    .unwrap();
                return;
            }
        };
    }
}
#[test]
fn a_mutated_non_leaf_discovery_cannot_record_another_documents_child() {
    exercise(false);
}
#[test]
fn worker_only_a_mutated_non_leaf_discovery_cannot_record_another_documents_child() {
    exercise(true);
}

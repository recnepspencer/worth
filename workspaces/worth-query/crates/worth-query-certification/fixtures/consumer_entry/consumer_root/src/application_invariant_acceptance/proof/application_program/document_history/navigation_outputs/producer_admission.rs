//! Current admission can resume; an intrinsic producer refusal remains terminal.
use super::*;
use std::sync::atomic::Ordering;
use worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind;

pub(super) fn configured_pair<'a>(
    app: &'a Runtime,
) -> (OutputDocument<'a>, WorthQueryHistoryNodeId) {
    let (document, target) = recorded_pair(app);
    (configure(document, app), target)
}

pub(super) fn recorded_pair<'a>(app: &'a Runtime) -> (Document<'a>, WorthQueryHistoryNodeId) {
    let mut document = attach(app, "anchor-a");
    let mut target = None;
    for (index, y) in [2, 5].into_iter().enumerate() {
        let result = finish(
            to_selection(
                begin(document, app, "anchor-a", y, 0x9182_e600 + index as u64),
                app,
            ),
            app,
        );
        let WorthQueryFixedHistoryEditOutcome::Recorded { step, delivery } = result.outcome else {
            panic!("record required source")
        };
        if index == 0 {
            target = Some(step.id());
        }
        drop(delivery);
        document = result.document;
    }
    (document, target.unwrap())
}

fn configure<'a>(document: Document<'a>, app: &'a Runtime) -> OutputDocument<'a> {
    app.with_host_advancement(|phase| {
        with_request(app, |request| {
            document
                .with_outputs::<HistoryProgram, HistoryRoot>(
                    &phase,
                    request,
                    app,
                    PlanarOutputDemand::new("anchor-a"),
                    Default::default(),
                )
                .unwrap_or_else(|(_, cause)| panic!("history profile: {cause:?}"))
        })
    })
    .unwrap()
}

#[test]
fn rejected_current_producer_scope_resumes_the_same_history_tree_with_lawful_input() {
    let world = install();
    let app = &world.application;
    let (document, target) = configured_pair(app);
    let expected = with_request(app, |request| document.selection(request).unwrap());
    // The installed provider's next actual input names a missing current C
    // scope. This is a scope-admission refusal, not revoked navigation grants.
    world
        .producer_authorization_denials
        .store(1, Ordering::SeqCst);
    let mut navigation = with_request(app, |request| {
        document
            .navigate::<PlanarHistoryNavigation<ConsumerSchema>>()
            .checkout(target)
            .expected_selection(expected.clone())
            .idempotency(0x9182_e610)
            .prepare(request)
            .unwrap()
    });
    let mut refusal_seen = false;
    for _ in 0..96 {
        navigation = match with_request(app, |request| navigation.advance(request)) {
            WorthQueryHistoryNavigationAdvance::Pending(next) => next,
            WorthQueryHistoryNavigationAdvance::Refused { cause, owner } => {
                let WorthQueryHistoryNavigationAdvanceDenial::Outputs(
                    WorthQueryHistoryOutputDenial::Preparation(cause),
                ) = cause
                else {
                    panic!("expected actual producer scope admission")
                };
                let WorthQueryRequiredOutputPreparationDenial::Demand(
                    WorthQueryApplicationOutputDemandDenial::History(cause),
                ) = cause
                else {
                    panic!("expected the original history producer refusal")
                };
                let WorthQueryHistoryOutputAdvanceDenial::Demand(cause) = cause else {
                    panic!("expected the original Native producer admission cause")
                };
                assert!(
                    matches!(
                        cause.kind(),
                        WorthQueryOutputDemandDenialKind::SourceScope(_)
                    ),
                    "{cause:?}"
                );
                assert_eq!(cause.recovery_posture(), worth_query_host::facade::primary_graph::WorthQueryOutputDemandRecoveryPosture::Retryable);
                refusal_seen = true;
                owner
            }
            WorthQueryHistoryNavigationAdvance::Selected(_) => {
                panic!("the rejected producer scope must reach actual C admission")
            }
        };
        if refusal_seen {
            break;
        }
    }
    assert!(refusal_seen);
    assert_eq!(
        world.producer_authorization_denials.load(Ordering::SeqCst),
        0
    );
    assert_eq!(
        with_request(app, |request| document.selection(request).unwrap()),
        expected
    );
    let original_d = navigation
        .candidate_branch_for_test()
        .expect("same actual D survives refusal");
    for _ in 0..96 {
        match with_request(app, |request| navigation.advance(request)) {
            WorthQueryHistoryNavigationAdvance::Pending(next) => {
                assert_eq!(next.candidate_branch_for_test(), Some(original_d));
                navigation = next;
            }
            WorthQueryHistoryNavigationAdvance::Selected(done) => {
                assert_eq!(
                    done.selection().physical_endpoint().product_branch(),
                    original_d
                );
                assert_eq!(done.requested_node(), target);
                assert_eq!(
                    with_request(app, |request| document.read(request).unwrap().rows()[0]
                        .value),
                    length(3)
                );
                app.with_host_advancement(|phase| document.close(&phase).unwrap())
                    .unwrap();
                return;
            }
            WorthQueryHistoryNavigationAdvance::Refused { cause, .. } => {
                panic!("fresh lawful scope must resume original tree: {cause:?}")
            }
        }
    }
    panic!("original history tree did not resume")
}

#[test]
fn producer_domain_denial_keeps_the_same_history_tree_terminal_until_disposal() {
    let _observation = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let world = install();
    let app = &world.application;
    let document = configure(attach(app, "anchor-a"), app);
    let contacts = app.producer_contacts_on_this_thread_for_test();
    let expected = with_request(app, |request| document.selection(request).unwrap());
    let target = expected.step().id();
    // Neither H nor C has had an output-producing edit. The real handler must
    // decide missing output on the actual candidate, then keep its tree terminal.
    world.producer_domain_denial.store(true, Ordering::SeqCst);
    let mut navigation = with_request(app, |request| {
        document
            .navigate::<PlanarHistoryNavigation<ConsumerSchema>>()
            .checkout(target)
            .expected_selection(expected.clone())
            .idempotency(0x9182_e611)
            .prepare(request)
            .unwrap()
    });
    let mut refusal_seen = false;
    for _ in 0..96 {
        navigation = match with_request(app, |request| navigation.advance(request)) {
            WorthQueryHistoryNavigationAdvance::Pending(next) => next,
            WorthQueryHistoryNavigationAdvance::Refused { cause, owner } => {
                let WorthQueryHistoryNavigationAdvanceDenial::Outputs(
                    WorthQueryHistoryOutputDenial::Preparation(
                        WorthQueryRequiredOutputPreparationDenial::Demand(
                            WorthQueryApplicationOutputDemandDenial::History(
                                WorthQueryHistoryOutputAdvanceDenial::Demand(cause),
                            ),
                        ),
                    ),
                ) = cause
                else {
                    panic!("expected the real producer domain refusal: {cause:?}")
                };
                assert_eq!(
                    app.producer_contacts_on_this_thread_for_test(),
                    contacts + 1,
                    "the actual handler must run before its domain refusal: {cause:?}"
                );
                assert_eq!(
                    cause.kind(),
                    WorthQueryOutputDemandDenialKind::ProducerDomainDenied,
                    "{cause:?}"
                );
                assert_eq!(
                    cause.domain_reason(),
                    Some("A current planar output is required before this decision.")
                );
                assert_eq!(cause.recovery_posture(), worth_query_host::facade::primary_graph::WorthQueryOutputDemandRecoveryPosture::Terminal);
                refusal_seen = true;
                owner
            }
            WorthQueryHistoryNavigationAdvance::Selected(_) => {
                panic!("a missing required producer output cannot settle")
            }
        };
        if refusal_seen {
            break;
        }
    }
    assert!(refusal_seen);
    assert_eq!(
        with_request(app, |request| document.selection(request).unwrap()),
        expected
    );
    let original_d = navigation
        .candidate_branch_for_test()
        .expect("failed original D remains owned");
    world.producer_domain_denial.store(false, Ordering::SeqCst);
    let navigation = match with_request(app, |request| navigation.advance(request)) {
        WorthQueryHistoryNavigationAdvance::Refused {
            cause:
                WorthQueryHistoryNavigationAdvanceDenial::Outputs(
                    WorthQueryHistoryOutputDenial::NeedsDisposal,
                ),
            owner,
        } => owner,
        _ => panic!("lawful input cannot resurrect the terminal original tree"),
    };
    assert_eq!(navigation.candidate_branch_for_test(), Some(original_d));
    with_request(app, |request| navigation.discard(request))
        .unwrap_or_else(|(_, cause)| panic!("terminal navigation disposal: {cause:?}"));
    app.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}

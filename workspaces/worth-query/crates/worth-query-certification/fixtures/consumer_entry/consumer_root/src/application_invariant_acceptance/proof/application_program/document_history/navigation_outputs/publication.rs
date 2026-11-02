//! Real producer effects precede the interrupted World publication of private D.
use super::*;
mod recovery_control;
mod stopped_recovery;
use std::num::NonZeroUsize;
use std::time::Duration;
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    product::{
        WorthQueryProductBranch, WorthQueryProductBranchAdmissionDenial,
        WorthQueryProductBranchAdmissionRefusalKind,
    },
};

type Navigation<'d, 'r> = WorthQueryHistoryNavigation<
    'd,
    'r,
    ConsumerSchema,
    History,
    PlanarHistoryNavigation<ConsumerSchema>,
    OrdinaryHistoryPayload,
    Outputs,
>;

pub(super) fn configured(app: &Runtime) -> (OutputDocument<'_>, WorthQueryHistoryNodeId) {
    let document = attach(app, "anchor-a");
    let root = with_request(app, |request| {
        document.selection(request).unwrap().step().id()
    });
    let result = finish(
        to_selection(begin(document, app, "anchor-a", 5, 0x9182_eb00), app),
        app,
    );
    let WorthQueryFixedHistoryEditOutcome::Recorded { delivery, .. } = result.outcome else {
        panic!("record a current model with its derived output")
    };
    drop(delivery);
    let document = app
        .with_host_advancement(|phase| {
            with_request(app, |request| {
                result
                    .document
                    .with_outputs::<HistoryProgram, HistoryRoot>(
                        &phase,
                        request,
                        app,
                        PlanarOutputDemand::new("anchor-a"),
                        Default::default(),
                    )
                    .unwrap_or_else(|(_, cause)| panic!("original required tree: {cause:?}"))
            })
        })
        .unwrap();
    (document, root)
}

pub(super) fn prepare<'d, 'r>(
    document: &'d OutputDocument<'r>,
    app: &Runtime,
    target: WorthQueryHistoryNodeId,
    key: u128,
) -> Navigation<'d, 'r> {
    let expected = with_request(app, |request| document.selection(request).unwrap());
    with_request(app, |request| {
        document
            .navigate::<PlanarHistoryNavigation<ConsumerSchema>>()
            .checkout(target)
            .expected_selection(expected)
            .idempotency(key)
            .prepare(request)
            .unwrap()
    })
}

pub(super) fn awaiting<'d, 'r>(mut owner: Navigation<'d, 'r>, app: &Runtime) -> Navigation<'d, 'r> {
    for _ in 0..96 {
        if owner.posture() == WorthQueryHistoryNavigationPosture::AwaitingOutputs {
            return owner;
        }
        owner = match with_request(app, |request| owner.advance(request)) {
            WorthQueryHistoryNavigationAdvance::Pending(next) => next,
            _ => panic!("initial D must publish before SDK output work"),
        };
    }
    panic!("initial D did not reach its required outputs")
}

fn with_scope<T>(
    app: &Runtime,
    scope: &WorthQueryRequestScope,
    perform: impl FnOnce(&Request<'_, '_, '_>) -> T,
) -> T {
    let adapter = authentication::admit(app.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        scope,
    ))
    .unwrap();
    perform(&app.request(&principal, scope))
}

fn unpublished<'d, 'r>(
    mut owner: Navigation<'d, 'r>,
    app: &Runtime,
) -> (Navigation<'d, 'r>, WorthQueryProductBranch) {
    assert_eq!(
        owner.posture(),
        WorthQueryHistoryNavigationPosture::AwaitingOutputs
    );
    let original_d = owner.candidate_branch_for_test().unwrap();
    // H's cold Length1 differs from the required output2. Before this arm, the
    // initial model-composition publication has already completed on World D.
    assert_eq!(
        with_branch_request(app, original_d, |request| request
            .query(PlanarOutputRead {
                body_key: "anchor-a".into(),
            })
            .execute()
            .unwrap()
            .rows()[0]
            .value),
        length(1)
    );
    let pause = app
        .runtime()
        .world_operation_control_for_test()
        .pause_before_product_compare_for_test(NonZeroUsize::MIN);
    let cancellation = WorthQueryCancellationSource::new();
    let (reached, refused) = std::thread::scope(|threads| {
        let worker = threads.spawn(|| {
            let reached = pause.wait_until_reached(Duration::from_secs(10));
            if reached {
                cancellation.cancel();
            }
            pause.release();
            reached
        });
        let mut refused = None;
        for _ in 0..96 {
            let scope = WorthQueryRequestScope::new(None, cancellation.token());
            match with_scope(app, &scope, |request| owner.advance(request)) {
                WorthQueryHistoryNavigationAdvance::Pending(next) => owner = next,
                WorthQueryHistoryNavigationAdvance::Refused { cause, owner } => {
                    refused = Some((cause, owner));
                    break;
                }
                WorthQueryHistoryNavigationAdvance::Selected(_) => {
                    panic!("interrupted output cannot select")
                }
            }
        }
        (worker.join().unwrap(), refused)
    });
    assert!(
        reached,
        "the actual producer effect must reach World comparison"
    );
    let (cause, owner) = refused.expect("return original private output custody");
    assert!(
        matches!(
            cause,
            WorthQueryHistoryNavigationAdvanceDenial::Outputs(
                WorthQueryHistoryOutputDenial::Preparation(
                    WorthQueryRequiredOutputPreparationDenial::Demand(
                        WorthQueryApplicationOutputDemandDenial::History(
                            WorthQueryHistoryOutputAdvanceDenial::Authority(
                                WorthQueryHistoryNavigationDenial::Stopped(
                                    WorthQueryAdvancementDenial::Interrupted(
                                        WorthQueryManagedComputationInterruption::Cancelled
                                    )
                                )
                            )
                        )
                    )
                )
            )
        ),
        "{cause:?}"
    );
    assert_eq!(owner.candidate_branch_for_test(), Some(original_d));
    assert_eq!(
        owner.posture(),
        WorthQueryHistoryNavigationPosture::AwaitingOutputs
    );
    (owner, original_d)
}

#[test]
fn unpublished_history_producer_resumes_original_effect_once_before_selecting_the_same_candidate() {
    let _observation = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let world = install();
    let app = &world.application;
    let (document, root) = configured(app);
    let before = with_request(app, |request| document.selection(request).unwrap());
    let owner = awaiting(prepare(&document, app, root, 0x9182_eb10), app);
    let contacts = app.producer_contacts_on_this_thread_for_test();
    let (mut owner, original_d) = unpublished(owner, app);
    assert_eq!(
        app.producer_contacts_on_this_thread_for_test(),
        contacts + 1
    );
    assert_eq!(
        with_request(app, |request| document.selection(request).unwrap()),
        before
    );
    assert_eq!(
        with_request(app, |request| document.read(request).unwrap().rows()[0]
            .value),
        length(6)
    );
    let page = app
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap();
    assert_eq!(page.rows().len(), 1);
    let original = page.rows()[0].handle().clone();
    drop(page);
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, cancellation.token());
    owner = match with_scope(app, &scope, |request| {
        cancellation.cancel();
        owner.advance(request)
    }) {
        WorthQueryHistoryNavigationAdvance::Refused { cause, owner } => {
            assert!(
                matches!(
                    cause,
                    WorthQueryHistoryNavigationAdvanceDenial::History(
                        WorthQueryApplicationHistoryDenial::ExecutionRequest(
                            WorthQueryAdvancementDenial::Interrupted(
                                WorthQueryManagedComputationInterruption::Cancelled
                            )
                        )
                    )
                ),
                "{cause:?}"
            );
            owner
        }
        _ => panic!("stopped readmission must preserve original unpublished effect"),
    };
    let page = app
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap();
    assert_eq!(page.rows().len(), 1);
    assert_eq!(page.rows()[0].handle(), &original);
    drop(page);
    for _ in 0..96 {
        match with_request(app, |request| owner.advance(request)) {
            WorthQueryHistoryNavigationAdvance::Pending(next) => {
                assert_eq!(next.candidate_branch_for_test(), Some(original_d));
                owner = next;
            }
            WorthQueryHistoryNavigationAdvance::Selected(done) => {
                assert_eq!(
                    done.selection().physical_endpoint().product_branch(),
                    original_d
                );
                assert_eq!(done.requested_node(), root);
                assert_eq!(
                    app.producer_contacts_on_this_thread_for_test(),
                    contacts + 1,
                    "World recovery cannot execute the producer again"
                );
                assert_eq!(
                    with_request(app, |request| document.read(request).unwrap().rows()[0]
                        .value),
                    length(2)
                );
                app.with_host_advancement(|phase| document.close(&phase).unwrap())
                    .unwrap();
                assert!(app
                    .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
                    .unwrap()
                    .rows()
                    .is_empty());
                return;
            }
            WorthQueryHistoryNavigationAdvance::Refused { cause, .. } => {
                panic!("resume original World effect: {cause:?}")
            }
        }
    }
    panic!("original unpublished effect did not finish")
}

#[test]
fn dropped_unpublished_history_producer_closes_only_its_original_effect_and_candidate() {
    let _observation = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let world = install();
    let app = &world.application;
    let (document, root) = configured(app);
    let before = with_request(app, |request| document.selection(request).unwrap());
    let owner = awaiting(prepare(&document, app, root, 0x9182_eb20), app);
    let contacts = app.producer_contacts_on_this_thread_for_test();
    let (owner, original_d) = unpublished(owner, app);
    assert_eq!(
        app.producer_contacts_on_this_thread_for_test(),
        contacts + 1
    );
    assert_eq!(
        with_request(app, |request| document.selection(request).unwrap()),
        before
    );
    drop(owner);
    let page = app
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap();
    assert_eq!(page.rows().len(), 1);
    let original = page.rows()[0].handle().clone();
    drop(page);
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, cancellation.token());
    let mut retained = None;
    let _outcome = app.runtime().with_application_advancement(&scope, |phase| {
        cancellation.cancel();
        retained = Some(document.close(&phase));
    });
    let Err(WorthQueryApplicationHistoryCloseFailure::ClosingOutputs(closing)) = retained.unwrap()
    else {
        panic!("stopped cleanup keeps live document and original effect")
    };
    assert!(
        matches!(
            closing.denial(),
            WorthQueryHistoryOutputDenial::Execution(WorthQueryAdvancementDenial::Interrupted(
                WorthQueryManagedComputationInterruption::Cancelled
            ))
        ),
        "{:?}",
        closing.denial()
    );
    let page = app
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap();
    assert_eq!(page.rows().len(), 1);
    assert_eq!(page.rows()[0].handle(), &original);
    drop(page);
    app.with_host_advancement(|phase| closing.retry(&phase))
        .unwrap()
        .unwrap_or_else(|cause| panic!("close original unpublished producer: {cause:?}"));
    assert_eq!(
        app.producer_contacts_on_this_thread_for_test(),
        contacts + 1
    );
    let retired = app
        .with_host_advancement(|phase| match app.on_branch(original_d).select(&phase) {
            Err(cause) => cause,
            Ok(_) => panic!("original D must actually retire"),
        })
        .unwrap();
    assert_eq!(
        retired,
        WorthQueryProductBranchAdmissionDenial::Refused(
            WorthQueryProductBranchAdmissionRefusalKind::RetiredBranch
        )
    );
    assert!(app
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .is_empty());
}

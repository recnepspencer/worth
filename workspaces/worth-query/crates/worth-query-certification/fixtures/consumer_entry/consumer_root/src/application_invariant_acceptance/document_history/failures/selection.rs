//! A stale gesture and an already-produced loser are distinct original fates.
use super::*;
use worth_query_host::facade::product::{
    WorthQueryProductBranch, WorthQueryProductBranchAdmissionDenial as Admission,
    WorthQueryProductBranchAdmissionRefusalKind as Refusal,
};

fn candidate(
    document: &mut Configured<'_>,
    app: &Runtime,
    command: &navigation::Command,
) -> WorthQueryProductBranch {
    with_request(app, |request| {
        navigation::prepare(document, request, command)
            .unwrap()
            .candidate_branch_for_test()
    })
    .unwrap()
}

fn awaiting(
    document: &mut Configured<'_>,
    app: &Runtime,
    command: &navigation::Command,
) -> WorthQueryProductBranch {
    use std::time::Duration;
    use worth_query_host::facade::{
        admission::authenticated_principal::WorthQueryCancellationSource,
        primary_graph::{
            WorthQueryOperationAuthorizationDenialKind as Authorization,
            WorthQueryProviderSessionControlStopKind as Stop,
        },
    };
    let pause = with_request(app, |request| {
        let mut owner = navigation::prepare(document, request, command).unwrap();
        owner.pause_before_history_navigation_selection_for_test()
    });
    let cancellation = WorthQueryCancellationSource::new();
    let (reached, refusal) = std::thread::scope(|threads| {
        let worker = threads.spawn(|| {
            let reached = pause.wait_until_reached(Duration::from_secs(10));
            if reached {
                cancellation.cancel();
            }
            pause.release();
            reached
        });
        let mut refusal = None;
        for _ in 0..96 {
            let scope = WorthQueryRequestScope::new(None, cancellation.token());
            match with_scope(app, &scope, |request| {
                navigation::advance(document, request, command)
            }) {
                Ok(navigation::Progress::Pending(_)) => {}
                Ok(navigation::Progress::Selected(_)) => {
                    panic!("guard cancellation precedes actual adoption")
                }
                Err(cause) => {
                    refusal = Some(cause);
                    break;
                }
            }
        }
        (worker.join().unwrap(), refusal)
    });
    assert!(reached, "actual complete-E selection receiver");
    let refusal = refusal.unwrap();
    let navigation::Failure::Advancement {
        cause:
            WorthQueryHistoryNavigationAdvanceDenial::Outputs(
                WorthQueryHistoryOutputDenial::Navigation(
                    WorthQueryHistoryNavigationDenial::Authorization(cause),
                ),
            ),
        posture: WorthQueryHistoryNavigationPosture::AwaitingSelection,
    } = refusal
    else {
        panic!("original complete candidate must remain: {refusal:?}");
    };
    assert_eq!(cause.kind(), Authorization::Stopped(Err(Stop::Cancelled)));
    candidate(document, app, command)
}

#[test]
fn stale_click_and_prepared_loser_preserve_the_actual_adoption_winner_and_original_cleanup() {
    let world = install();
    let app = &world.application;
    let (mut document, context, first) = pair(app);
    let initial = selected(&document, app);
    let stale = capture(
        &document,
        app,
        &context,
        WorthQueryHistoryNavigationIntent::Checkout(first.id()),
        0x9182_7720,
    );
    let loser = capture(
        &document,
        app,
        &context,
        WorthQueryHistoryNavigationIntent::Checkout(first.id()),
        0x9182_7721,
    );
    let winner = capture(
        &document,
        app,
        &context,
        WorthQueryHistoryNavigationIntent::Checkout(initial.step().id()),
        0x9182_7722,
    );
    assert_eq!(loser.expected, winner.expected);
    let lost_d = awaiting(&mut document, app, &loser);
    let won_d = awaiting(&mut document, app, &winner);
    assert_ne!(lost_d, won_d);
    assert_eq!(selected(&document, app), initial);
    for (branch, source, output, child, child_y) in [
        (lost_d, 2, 3, "final:remote-b", 43),
        (won_d, 5, 6, "final:sibling-c", 32),
    ] {
        with_branch_request(app, branch, |request| {
            assert_eq!(
                request
                    .query(PlanarRead {
                        body_key: "anchor-a".into()
                    })
                    .execute()
                    .unwrap()
                    .rows()[0]
                    .y,
                length(source)
            );
            assert_eq!(
                request
                    .query(PlanarOutputRead {
                        body_key: "anchor-a".into()
                    })
                    .execute()
                    .unwrap()
                    .rows()[0]
                    .value,
                length(output)
            );
            assert_eq!(
                request
                    .query(PlanarRead {
                        body_key: child.into()
                    })
                    .execute()
                    .unwrap()
                    .rows()[0]
                    .y,
                length(child_y)
            );
        });
    }
    let won = navigation::complete(&mut document, app, &winner);
    assert_eq!(won.selection().physical_endpoint().product_branch(), won_d);
    assert_ne!(
        won.selection().physical_endpoint(),
        initial.physical_endpoint()
    );
    let old = with_request(app, |request| {
        navigation::advance(&mut document, request, &stale)
    })
    .err()
    .unwrap();
    assert!(
        matches!(
            old,
            navigation::Failure::Preparation(WorthQueryHistoryNavigationAdvanceDenial::Outputs(
                WorthQueryHistoryOutputDenial::Navigation(
                    WorthQueryHistoryNavigationDenial::HeadChanged
                )
            ))
        ),
        "{old:?}"
    );
    let loss = with_request(app, |request| {
        navigation::advance(&mut document, request, &loser)
    })
    .err()
    .unwrap();
    assert!(
        matches!(
            loss,
            navigation::Failure::Advancement {
                cause: WorthQueryHistoryNavigationAdvanceDenial::Outputs(
                    WorthQueryHistoryOutputDenial::Navigation(
                        WorthQueryHistoryNavigationDenial::HeadChanged
                    )
                ),
                posture: WorthQueryHistoryNavigationPosture::AwaitingSelection,
            }
        ),
        "{loss:?}"
    );
    assert_eq!(candidate(&mut document, app, &loser), lost_d);
    assert_eq!(selected(&document, app), *won.selection());
    journey::values(&document, app, 5, "sibling-c", 32);
    with_request(app, |request| {
        navigation::prepare(&mut document, request, &loser)
            .unwrap()
            .discard(request)
            .unwrap_or_else(|(_, cause)| panic!("the same original loser must dispose: {cause:?}"))
    });
    app.with_host_advancement(|phase| {
        assert!(matches!(
            app.runtime().on_branch(lost_d).select(&phase),
            Err(Admission::Refused(Refusal::RetiredBranch))
        ))
    })
    .unwrap();
    assert_eq!(selected(&document, app), *won.selection());
    model::close(document, app);
    assert!(app.branches().pending_cleanup().is_empty());
}

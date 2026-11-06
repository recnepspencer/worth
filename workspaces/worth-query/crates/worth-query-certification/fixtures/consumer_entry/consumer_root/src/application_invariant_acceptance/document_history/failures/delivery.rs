//! Actual delivery cancellation leaves adopted state in its original registry.
use super::*;
use std::time::Duration;
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    primary_graph::{
        WorthQueryOperationAuthorizationDenialKind as Authorization,
        WorthQueryProviderSessionControlStopKind as Stop,
    },
};

#[test]
fn lost_delivery_replays_the_original_completion_without_resetting_a_later_document_selection() {
    let world = install();
    let app = &world.application;
    let (mut document, context, first) = pair(app);
    let before = selected(&document, app);
    let command = capture(
        &document,
        app,
        &context,
        WorthQueryHistoryNavigationIntent::Checkout(first.id()),
        0x9182_7730,
    );
    let pause = with_request(app, |request| {
        let mut owner = navigation::prepare(&mut document, request, &command).unwrap();
        owner.pause_after_history_navigation_adoption_for_test()
    });
    let cancellation = WorthQueryCancellationSource::new();
    let mut original_d = None;
    let (reached, failure) = std::thread::scope(|threads| {
        let worker = threads.spawn(|| {
            let reached = pause.wait_until_reached(Duration::from_secs(10));
            if reached {
                cancellation.cancel();
            }
            pause.release();
            reached
        });
        let mut failure = None;
        for _ in 0..96 {
            let scope = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
            match with_scope(app, &scope, |request| {
                navigation::advance(&mut document, request, &command)
            }) {
                Ok(navigation::Progress::Pending(_)) => {
                    let candidate = with_request(app, |request| {
                        navigation::prepare(&mut document, request, &command)
                            .unwrap()
                            .candidate_branch_for_test()
                    });
                    if let Some(candidate) = candidate {
                        if let Some(original) = original_d {
                            assert_eq!(candidate, original);
                        }
                        original_d = Some(candidate);
                    }
                }
                Ok(navigation::Progress::Selected(_)) => {
                    panic!("cancelled delivery must retain selected posture instead")
                }
                Err(cause) => {
                    failure = Some(cause);
                    break;
                }
            }
        }
        (worker.join().unwrap(), failure)
    });
    assert!(
        reached,
        "the SAME command must reach the actual adoption receiver"
    );
    let failure = failure.expect("original selected delivery refusal");
    let navigation::Failure::Advancement {
        cause:
            WorthQueryHistoryNavigationAdvanceDenial::Outputs(
                WorthQueryHistoryOutputDenial::Navigation(
                    WorthQueryHistoryNavigationDenial::Authorization(cause),
                ),
            ),
        posture: WorthQueryHistoryNavigationPosture::Selected,
    } = failure
    else {
        panic!("actual selected delivery interruption: {failure:?}");
    };
    assert_eq!(cause.kind(), Authorization::Stopped(Err(Stop::Cancelled)));
    let adopted = selected(&document, app);
    assert_eq!(adopted.step().id(), first.id());
    assert_eq!(
        adopted.physical_endpoint().product_branch(),
        original_d.unwrap()
    );
    journey::values(&document, app, 2, "remote-b", 43);
    let later = capture(
        &document,
        app,
        &context,
        WorthQueryHistoryNavigationIntent::Checkout(before.step().id()),
        0x9182_7731,
    );
    let later = navigation::complete(&mut document, app, &later);
    journey::values(&document, app, 5, "sibling-c", 32);
    let contacts = app.producer_contacts_on_this_thread_for_test();
    let replay = navigation::complete(&mut document, app, &command);
    assert_eq!(replay.selection(), &adopted);
    assert_eq!(command.expected, before);
    assert_eq!(selected(&document, app), *later.selection());
    assert_eq!(app.producer_contacts_on_this_thread_for_test(), contacts);
    model::close(document, app);
    assert!(app.branches().pending_cleanup().is_empty());
}

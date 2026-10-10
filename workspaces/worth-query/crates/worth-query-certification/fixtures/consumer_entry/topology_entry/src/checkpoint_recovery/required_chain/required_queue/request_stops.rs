//! A stop that belongs to one advance leaves its required row to a later one.

use super::super::contact_readings::Reading;
use super::*;
use std::time::{Duration, Instant};
use worth_query_host::facade::application_contribution::{
    WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
};
use worth_query_host::facade::primary_graph::WorthQueryOutputDemandRecoveryPosture;

/// Advances the middle consumer on a request that `$cancel` stops, and checks
/// it meets that request's own stop after `$decisions` middle-row decisions.
macro_rules! cancelled_middle_advance {
    ($application:expr, $principal:expr, $demand:expr, $cancel:expr, $decisions:expr) => {{
        let cancellation = authentication::WorthQueryCancellationSource::new();
        let cancelled_scope = authentication::WorthQueryRequestScope::new(
            Instant::now() + Duration::from_secs(120),
            cancellation.token(),
        );
        let cancelled = $application.request(&$principal, &cancelled_scope);
        take_decisions("anchor-b");
        let cancel: fn(authentication::WorthQueryCancellationSource) = $cancel;
        cancel(cancellation);
        let stopped = $demand.advance(&cancelled);
        assert!(
            matches!(
                &stopped,
                Err(worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(denial))
                    if if $decisions == 0 {
                        denial.kind() == WorthQueryOutputDemandDenialKind::ExecutionRequest(
                            WorthQueryAdvancementDenial::Interrupted(
                                WorthQueryManagedComputationInterruption::Cancelled,
                            ),
                        ) && denial.recovery_posture() == WorthQueryOutputDemandRecoveryPosture::Terminal
                            && denial.subject() == "request admission"
                    } else {
                        denial.kind() == WorthQueryOutputDemandDenialKind::Cancelled
                    }
            ),
            "the cancelled request meets its own stop: {:?}",
            stopped.as_ref().err()
        );
        let decisions = take_decisions("anchor-b").len();
        assert_eq!(
            decisions,
            $decisions,
            "the stop interrupted the middle row's refresh where the request was cancelled"
        );
        decisions
    }};
}

/// Advances the middle consumer on a request that is cancelled while the
/// middle row's refresh decides.
macro_rules! cancelled_inside_middle_refresh {
    ($application:expr, $principal:expr, $demand:expr) => {
        cancelled_middle_advance!(
            $application,
            $principal,
            $demand,
            |cancellation| super::super::binding::cancel_during_next_decision(
                "anchor-b",
                cancellation
            ),
            1
        )
    };
}

#[test]
fn a_queue_frame_whose_stage_defers_leaves_its_row_to_a_later_advance() {
    let _guard = checkpoint_recovery_test_guard();
    let application = chain_application();
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (a, b, mut c, mut d) = chain_with_unrelated!(application, request);
    change_root_input!(request, application, 2, 0x9176_3e00_u64);
    // The unrelated caller runs the root's refresh as a queue frame, and
    // that refresh defers its readiness delivery.
    application.delay_next_output_readiness_delivery_for_test();
    settled_in_one_advance!(d, request, "the unrelated required demand");
    // The frame's successor outlived the frame's wave: the last consumer
    // finishes the deferred row and the chain above it.
    settled_in_one_advance!(c, request, "the last consumer");
    drop((a, b, c, d));
}

#[test]
fn a_required_refresh_its_request_stops_is_left_to_a_later_advance() {
    let _guard = checkpoint_recovery_test_guard();
    let application = chain_application();
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (a, mut b, mut c, mut d) = chain_with_unrelated!(application, request);
    change_root_input!(request, application, 2, 0x9176_3f00_u64);
    // A queue frame leaves the root's refresh waiting on its deferred
    // readiness delivery.
    application.delay_next_output_readiness_delivery_for_test();
    settled_in_one_advance!(d, request, "the unrelated required demand");
    // A request cancelled inside the middle row's refresh stops there.
    cancelled_inside_middle_refresh!(application, principal, b);
    // The stop was the request's, not the row's: the dependent of that row
    // redoes its refresh on a later advance.
    settled_in_one_advance!(c, request, "the last consumer");
    drop((a, b, c, d));
}

#[test]
fn a_cancelled_caller_retries_its_own_successor_on_its_next_advance() {
    let _guard = checkpoint_recovery_test_guard();
    let application = chain_application();
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (mut a, mut b, mut c, mut d) = chain_with_unrelated!(application, request);
    let mut root_reading = Reading::default();
    root_reading.contacts(
        settled_in_one_advance!(a, request, "the initial root reading")
            .producer_contacts_in_this_demand(),
    );
    let mut middle_reading = Reading::default();
    middle_reading.contacts(
        settled_in_one_advance!(b, request, "the initial middle reading")
            .producer_contacts_in_this_demand(),
    );
    change_root_input!(request, application, 2, 0x9176_3f80_u64);
    // A queue frame refreshes the root and leaves its readiness delivery
    // deferred, so the middle row's refresh is still to come.
    application.delay_next_output_readiness_delivery_for_test();
    settled_in_one_advance!(d, request, "the unrelated required demand");
    let cancelled_contacts = cancelled_inside_middle_refresh!(application, principal, b);
    // The same caller's next advance resumes its own interrupted row and
    // finishes it, without contacting the root producer that already
    // refreshed.
    crate::producer::reset_provider_contacts();
    let resumed = settled_in_one_advance!(b, request, "the cancelled caller");
    assert_eq!(
        crate::producer::provider_contacts(),
        0,
        "the already refreshed root producer is not contacted again"
    );
    assert_eq!(
        middle_reading.contacts(resumed.producer_contacts_in_this_demand()) - cancelled_contacts,
        1,
        "the cancelled caller refreshes its own row once"
    );
    assert_eq!(take_decisions("anchor-b").len(), 1);
    let last = settled_in_one_advance!(c, request, "the last consumer");
    assert_eq!(
        Reading::default().contacts(last.producer_contacts_in_this_demand()),
        1,
        "the last consumer has one own execution since its handle opened"
    );
    let root = settled_in_one_advance!(a, request, "the open root demand");
    assert_eq!(
        root_reading.contacts(root.producer_contacts_in_this_demand()),
        0
    );
    drop((a, b, c, d));
}

#[test]
fn a_request_cancelled_before_it_advances_leaves_the_row_to_a_later_advance() {
    let _guard = checkpoint_recovery_test_guard();
    let application = chain_application();
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (a, mut b, mut c, mut d) = chain_with_unrelated!(application, request);
    change_root_input!(request, application, 2, 0x9176_3fc0_u64);
    application.delay_next_output_readiness_delivery_for_test();
    settled_in_one_advance!(d, request, "the unrelated required demand");
    // A request already cancelled when the middle consumer advances stops
    // before the middle row's refresh decides anything.
    cancelled_middle_advance!(
        application,
        principal,
        b,
        |cancellation| cancellation.cancel(),
        0
    );
    // The dependent of that row refreshes it on a later advance.
    settled_in_one_advance!(c, request, "the last consumer");
    drop((a, b, c, d));
}

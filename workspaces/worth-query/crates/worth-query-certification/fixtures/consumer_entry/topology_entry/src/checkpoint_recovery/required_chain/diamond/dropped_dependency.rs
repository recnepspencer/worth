//! A changed consumer must disclose its new decision before following obsolete edges.
use super::*;

#[test]
fn changed_own_output_can_drop_unresolved_consumed_dependencies() {
    journey(true, ChildDelivery::Direct, false);
}

#[test]
fn changed_own_decision_still_refreshes_the_dependency_it_reads() {
    journey(false, ChildDelivery::Direct, false);
}

#[cfg(feature = "test-output-delivery-faults")]
#[test]
fn changed_decision_finishes_a_held_dependency_without_advancing_its_caller() {
    journey(false, ChildDelivery::Held, false);
}

#[test]
fn changed_decision_retries_after_cancellation_with_its_refreshed_child() {
    journey(false, ChildDelivery::Direct, true);
}

#[cfg(feature = "test-output-delivery-faults")]
#[test]
fn changed_decision_retries_after_cancellation_with_a_held_child() {
    journey(false, ChildDelivery::Held, true);
}

#[cfg(feature = "test-output-delivery-faults")]
#[test]
fn changed_decision_refreshes_a_child_superseded_before_readiness_delivery() {
    journey(false, ChildDelivery::SupersededWhileHeld, false);
}

#[cfg(feature = "test-output-delivery-faults")]
#[test]
fn superseded_child_refresh_survives_cancellation_of_its_consumer() {
    journey(false, ChildDelivery::SupersededWhileHeld, true);
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ChildDelivery {
    Direct,
    #[cfg(feature = "test-output-delivery-faults")]
    Held,
    #[cfg(feature = "test-output-delivery-faults")]
    SupersededWhileHeld,
}

fn journey(drop_edges: bool, child_delivery: ChildDelivery, cancel_decision: bool) {
    let held_child = child_delivery != ChildDelivery::Direct;
    #[cfg(feature = "test-output-delivery-faults")]
    let superseded_child = child_delivery == ChildDelivery::SupersededWhileHeld;
    #[cfg(not(feature = "test-output-delivery-faults"))]
    let superseded_child = false;
    let _guard = checkpoint_recovery_test_guard();
    take_decisions("diamond-join");
    let profile =
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard();
    let application = support::install_program_with_seed::<program::ChainProgram>(
        None,
        profile,
        4096,
        128 * 1024 * 1024,
        u64::try_from(profile.limits().source_currentness_work()).unwrap(),
        seed,
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut left = request
        .demand(PlanarOutputDemand::new("diamond-left"))
        .start_in_program::<program::ChainProgram, program::ChainRoot>(&application)
        .unwrap();
    let mut right = request
        .demand(PlanarOutputDemand::new("diamond-right"))
        .start_in_program::<program::ChainProgram, program::ChainRoot>(&application)
        .unwrap();
    let mut join = request
        .demand(ChainDemand("diamond-join".into()))
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&application)
        .unwrap();
    settle!(left, request);
    settle!(right, request);
    settle!(join, request);
    assert_eq!(take_decisions("diamond-join"), [[2, 51]]);

    // The old dependency needs a new producer decision. The consumer's own native
    // decision fact changes as well; only its fresh handler decides whether to consume it.
    let mut root_demands = Some((left, right));
    if !held_child {
        drop(root_demands.take());
    }
    change_root!(request, application, "diamond-left", 5, 0x9176_3580_u64);
    #[cfg(feature = "test-output-delivery-faults")]
    if held_child {
        // A clean independent caller drives the dirty left root as a queue frame.
        // Its deferred successor is then owned by the registry, not an open caller.
        application.delay_next_output_readiness_delivery_for_test();
        settled_in_one_advance!(
            root_demands.as_mut().unwrap().1,
            request,
            "the independent right caller"
        );
        assert_eq!(
            output_lengths!(request, ["diamond-left"]),
            [6],
            "the queue actually published the left refresh"
        );
    }
    if superseded_child {
        change_root!(request, application, "diamond-left", 7, 0x9176_3582_u64);
        assert_eq!(
            output_lengths!(request, ["diamond-left"]),
            [6],
            "the second source edit has not run the child producer"
        );
    }
    let _retained_root_interests = root_demands;
    let before = request
        .query(PlanarOutputRead {
            body_key: "diamond-join".into(),
        })
        .execute()
        .unwrap();
    change_root!(
        request,
        application,
        "diamond-join",
        if drop_edges { 105 } else { 104 },
        0x9176_3581_u64
    );
    let after = request
        .query(PlanarOutputRead {
            body_key: "diamond-join".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        before.rows(),
        after.rows(),
        "the producer source stays equal while its own decision facts change"
    );
    assert!(
        take_decisions("diamond-join").is_empty(),
        "the source write cannot refresh the consumer"
    );
    let latest_child = if superseded_child { 8 } else { 6 };
    let prior_join = output_lengths!(request, ["diamond-join"]);
    crate::producer::reset_provider_contacts();
    if cancel_decision {
        let cancellation = authentication::WorthQueryCancellationSource::new();
        let cancelled_scope = authentication::WorthQueryRequestScope::new(
            std::time::Instant::now() + std::time::Duration::from_secs(120),
            cancellation.token(),
        );
        super::super::binding::cancel_during_next_decision("diamond-join", cancellation);
        let cancelled = application.request(&principal, &cancelled_scope);
        let stopped = join.advance(&cancelled);
        assert!(
            matches!(
                &stopped,
                Err(worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(denial))
                    if denial.kind() == WorthQueryOutputDemandDenialKind::Cancelled
            ),
            "the decision meets the actual request cancellation: {:?}",
            stopped.as_ref().err()
        );
        assert_eq!(take_decisions("diamond-join"), [[latest_child, 51]]);
        assert_eq!(output_lengths!(request, ["diamond-left"]), [latest_child]);
        assert_eq!(
            output_lengths!(request, ["diamond-join"]),
            prior_join,
            "the cancelled consumer did not publish"
        );
        if superseded_child {
            assert!(
                crate::producer::provider_contacts() > 0,
                "the newer child source requires production before the consumer can decide"
            );
        }
        crate::producer::reset_provider_contacts();
    }
    let refreshed = if held_child {
        match join.advance(&request) {
            Ok(WorthQueryApplicationOutputDemandProgress::Settled(settled)) => settled,
            Err(worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Superseded)
            | Ok(WorthQueryApplicationOutputDemandProgress::Pending) => {
                settled_in_one_advance!(join, request, "the held child retry")
            }
            Err(worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(denial))
                if denial.kind() == WorthQueryOutputDemandDenialKind::PublicationStale => {
                    assert_eq!(denial.recovery_posture(), worth_query_host::facade::primary_graph::WorthQueryOutputDemandRecoveryPosture::Retryable);
                    assert_eq!(output_lengths!(request, ["diamond-join"]), prior_join, "the stale attempt did not publish the join");
                    settled_in_one_advance!(join, request, "the refreshed publication retry")
                }
            Err(error) => panic!("the exact held dependency failed: {error:?}"),
        }
    } else {
        settled_in_one_advance!(join, request, "the changed decision")
    };
    if cancel_decision && !held_child {
        assert_eq!(
            refreshed.producer_contacts_in_this_demand(),
            1,
            "the cancelled decision did not publish a reusable join output"
        );
    }
    if drop_edges {
        assert_eq!(take_decisions("diamond-join"), [Vec::<u64>::new()]);
        assert_eq!(refreshed.producer_contacts_in_this_demand(), 1);
        assert_eq!(
            crate::producer::provider_contacts(),
            0,
            "the fresh consumer dropped both old edges; neither root should execute for it"
        );
        assert_eq!(
            output_lengths!(request, ["diamond-left"]),
            [2],
            "the old upstream remains unresolved, rather than being falsely certified"
        );
    } else {
        assert_eq!(
            take_decisions("diamond-join"),
            if superseded_child {
                vec![vec![8, 51]]
            } else if held_child {
                vec![vec![6, 51], vec![6, 51]]
            } else {
                vec![vec![6, 51]]
            },
            "the completed decision retries after readiness changes its publication basis"
        );
        if cancel_decision {
            assert_eq!(
                crate::producer::provider_contacts(),
                0,
                "the completed or held child resumes without another producer decision"
            );
        } else if superseded_child {
            assert!(
                crate::producer::provider_contacts() > 0,
                "the current source requires new child production"
            );
        } else if held_child {
            assert_eq!(
                crate::producer::provider_contacts(),
                0,
                "readiness recovery must reuse the actual published child"
            );
        } else {
            assert!(crate::producer::provider_contacts() > 0);
        }
        assert_eq!(
            output_lengths!(request, ["diamond-left"]),
            [if superseded_child { 8 } else { 6 }]
        );
    }
    let repeated = settle!(join, request);
    assert_eq!(repeated.producer_contacts_in_this_demand(), 0);
    assert!(take_decisions("diamond-join").is_empty());
}

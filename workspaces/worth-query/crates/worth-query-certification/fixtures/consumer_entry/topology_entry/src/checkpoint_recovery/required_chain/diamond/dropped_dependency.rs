//! A changed consumer must disclose its new decision before following obsolete edges.
use super::*;

#[test]
fn changed_own_output_can_drop_unresolved_consumed_dependencies() {
    journey(true, false);
}

#[test]
fn changed_own_decision_still_refreshes_the_dependency_it_reads() {
    journey(false, false);
}

#[test]
fn changed_decision_retries_after_cancellation_with_its_refreshed_child() {
    journey(false, true);
}

fn journey(drop_edges: bool, cancel_decision: bool) {
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
    drop((left, right));
    change_root!(request, application, "diamond-left", 5, 0x9176_3580_u64);
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
        assert_eq!(take_decisions("diamond-join"), [[6, 51]]);
        assert_eq!(output_lengths!(request, ["diamond-left"]), [6]);
        crate::producer::reset_provider_contacts();
    }
    let refreshed = settled_in_one_advance!(join, request, "the changed decision");
    if cancel_decision {
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
            [[6, 51]],
            "the new decision still needs the changed left output"
        );
        if cancel_decision {
            assert_eq!(
                crate::producer::provider_contacts(),
                0,
                "the refreshed child is reused without another producer decision"
            );
        } else {
            assert!(crate::producer::provider_contacts() > 0);
        }
        assert_eq!(output_lengths!(request, ["diamond-left"]), [6]);
    }
    let repeated = settle!(join, request);
    assert_eq!(repeated.producer_contacts_in_this_demand(), 0);
    assert!(take_decisions("diamond-join").is_empty());
}

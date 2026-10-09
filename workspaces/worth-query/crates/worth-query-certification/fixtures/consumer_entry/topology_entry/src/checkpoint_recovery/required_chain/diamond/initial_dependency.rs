//! An initial consumer must progress an exact pending native output it reads.
use super::*;

#[test]
fn an_initial_consumer_refreshes_the_pending_output_it_actually_reads() {
    journey(false, false, false);
}

#[test]
fn an_initial_consumer_does_not_certify_a_cancelled_decision() {
    journey(true, false, false);
}

#[test]
fn a_cancelled_initial_consumer_releases_and_retries_its_cached_dependency() {
    journey(true, true, false);
}

#[test]
fn an_initial_consumer_readmits_the_cached_dependency_it_actually_reads() {
    journey(false, true, false);
}

#[test]
#[cfg(feature = "test-output-delivery-faults")]
fn a_cached_read_keeps_custody_while_another_demand_delivers_its_pending_child() {
    journey(false, true, true);
}

fn journey(cancel_decision: bool, close_dependencies: bool, delay_child: bool) {
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
    settle!(left, request);
    settle!(right, request);
    // No accepted consumer output or prior consumed edge exists yet.
    let mut join = request
        .demand(ChainDemand("diamond-join".into()))
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&application)
        .unwrap();
    change_root!(request, application, "diamond-left", 5, 0x9176_3590_u64);
    crate::producer::reset_provider_contacts();
    if close_dependencies {
        drop((left, right));
    }
    #[cfg(feature = "test-output-delivery-faults")]
    if delay_child {
        application.delay_next_output_readiness_delivery_for_test();
        assert!(matches!(
            join.advance(&request).unwrap(),
            WorthQueryApplicationOutputDemandProgress::Pending
        ));
        assert!(take_decisions("diamond-join").is_empty());
        assert_eq!(output_lengths!(request, ["diamond-left"]), [6]);
        crate::producer::reset_provider_contacts();
        // A distinct authentic caller delivers the already performed child.
        // The initial consumer's temporary claim must survive its Pending.
        let mut delivered = request
            .demand(PlanarOutputDemand::new("diamond-left"))
            .start_in_program::<program::ChainProgram, program::ChainRoot>(&application)
            .unwrap();
        settle!(delivered, request);
        assert_eq!(crate::producer::provider_contacts(), 0);
        drop(delivered);
    }
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
            matches!(&stopped,
            Err(worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(denial))
                if denial.kind() == WorthQueryOutputDemandDenialKind::Cancelled),
            "{:?}",
            stopped.as_ref().err()
        );
        assert_eq!(take_decisions("diamond-join"), [[6, 51]]);
        assert_eq!(output_lengths!(request, ["diamond-left"]), [6]);
        crate::producer::reset_provider_contacts();
    }
    let initial = (0..16)
        .find_map(|_| match join.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Settled(done) => Some(done),
            WorthQueryApplicationOutputDemandProgress::Pending => None,
        })
        .unwrap_or_else(|| {
            panic!(
                "initial dependency did not settle: lengths={:?}, decisions={:?}, contacts={}",
                output_lengths!(request, ["diamond-left", "diamond-right"]),
                take_decisions("diamond-join"),
                crate::producer::provider_contacts()
            )
        });
    assert_eq!(
        initial.producer_contacts_in_this_demand(),
        if cancel_decision { 3 } else { 2 },
        "failed and completed producer attempts are counted honestly"
    );
    assert_eq!(take_decisions("diamond-join"), [[6, 51]]);
    assert_eq!(
        output_lengths!(request, ["diamond-left", "diamond-right"]),
        [6, 51]
    );
    if cancel_decision {
        assert_eq!(
            crate::producer::provider_contacts(),
            0,
            "the completed child is reused after cancellation"
        );
    } else if !delay_child {
        assert!(crate::producer::provider_contacts() > 0);
    }
    let repeated = settle!(join, request);
    assert_eq!(
        repeated.producer_contacts_in_this_demand() - initial.producer_contacts_in_this_demand(),
        0
    );
    assert!(take_decisions("diamond-join").is_empty());
}

#[test]
fn an_initial_consumer_finishes_the_requested_middle_before_its_own_decision() {
    let _guard = checkpoint_recovery_test_guard();
    take_decisions("anchor-c");
    let profile =
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard();
    let application = support::install_program_with_seed::<program::ChainProgram>(
        None,
        profile,
        4096,
        128 * 1024 * 1024,
        u64::try_from(profile.limits().source_currentness_work()).unwrap(),
        super::super::source_world::seed,
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut a = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .start_in_program::<program::ChainProgram, program::ChainRoot>(&application)
        .unwrap();
    let mut b = request
        .demand(ChainDemand("anchor-b".into()))
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&application)
        .unwrap();
    settle!(a, request);
    let initial_b = settle!(b, request);
    take_decisions("anchor-c");
    let mut c = request
        .demand(ChainDemand("anchor-c".into()))
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&application)
        .unwrap();
    change_root!(request, application, "anchor-a", 5, 0x9176_3591_u64);
    let done = settle!(c, request);
    assert_eq!(done.producer_contacts_in_this_demand(), 2);
    let decisions = take_all_decisions();
    assert_eq!(
        decisions
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        ["anchor-b", "anchor-c"]
    );
    assert_eq!(output_lengths!(request, ["anchor-a"]), [6]);
    assert_eq!(
        settle!(b, request).producer_contacts_in_this_demand()
            - initial_b.producer_contacts_in_this_demand(),
        0
    );
    assert_eq!(
        settle!(c, request).producer_contacts_in_this_demand()
            - done.producer_contacts_in_this_demand(),
        0
    );
    assert!(take_all_decisions().is_empty());
}

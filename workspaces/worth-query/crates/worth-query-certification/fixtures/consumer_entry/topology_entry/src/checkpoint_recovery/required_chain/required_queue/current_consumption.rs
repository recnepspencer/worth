//! Current roots decide before fresh queued claims enter their handlers.
use super::*;

#[test]
fn a_current_caller_is_accepted_before_fresh_queue_handler_contact() {
    let _guard = checkpoint_recovery_test_guard();
    let profile =
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard();
    let application = support::install_program_with_limits::<program::ChainProgram>(
        None,
        profile,
        support::limits(
            4096,
            support::invalidation(
                128 * 1024 * 1024,
                profile
                    .limits()
                    .source_currentness_work()
                    .map(|work| work as u64),
                4096,
            ),
        ),
        super::super::diamond::seed,
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    macro_rules! root {
        ($key:literal) => {
            request
                .demand(PlanarOutputDemand::new($key))
                .start_in_program::<program::ChainProgram, program::ChainRoot>(&application)
                .unwrap()
        };
    }
    let mut left = root!("diamond-left");
    let mut fresh = root!("diamond-right");
    let mut caller = root!("diamond-join");
    settle!(left, request);
    settle!(fresh, request);
    settle!(caller, request);
    settled_in_one_advance!(caller, request, "the Current baseline");
    writes_y!(request, application, "diamond-right", 54, 0x9176_4700_u64);
    let roots = ["diamond-join", "diamond-right"].map(|key| {
        request
            .query(PlanarRead {
                body_key: key.into(),
            })
            .execute()
            .unwrap()
            .observed_sources()[0]
            .root_entity_for_test()
    });
    let contacts =
        roots.map(|root| application.producer_contacts_at_root_on_this_thread_for_test(root));
    application.exhaust_request_at_performed_commit_for_test();
    let done = settled_in_one_advance!(caller, request, "the already accepted caller");
    assert_eq!(
        done.producer_contacts_in_this_demand(),
        1,
        "the Current caller retains its one initial contact even when queue work stops"
    );
    assert_eq!(
        application.producer_contacts_at_root_on_this_thread_for_test(roots[0]) - contacts[0],
        0,
        "acceptance needs no new caller handler contact"
    );
    assert_eq!(
        application.producer_contacts_at_root_on_this_thread_for_test(roots[1]) - contacts[1],
        1,
        "the fresh queued handler really ran after caller acceptance"
    );
}

#[test]
fn a_current_requested_target_returns_before_fresh_queue_handler_contact() {
    let _guard = checkpoint_recovery_test_guard();
    let application = chain_application();
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
    let mut unrelated = request
        .demand(PlanarOutputDemand::new("anchor-source-b"))
        .start_in_program::<program::ChainProgram, program::ChainRoot>(&application)
        .unwrap();
    settle!(a, request);
    settle!(b, request);
    settle!(unrelated, request);
    settled_in_one_advance!(unrelated, request, "the initial queue is consumed");
    let mut c = request
        .demand(ChainDemand("anchor-c".into()))
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&application)
        .unwrap();
    change_root_input!(request, application, 5, 0x9176_4701_u64);
    writes_y!(request, application, "anchor-source-b", 2, 0x9176_4702_u64);
    let root = request
        .query(PlanarRead {
            body_key: "anchor-source-b".into(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .root_entity_for_test();
    let contacts = application.producer_contacts_at_root_on_this_thread_for_test(root);
    take_all_decisions();
    let done = settled_in_one_advance!(
        c,
        request,
        "the initial consumer obtains its requested target"
    );
    assert_eq!(done.producer_contacts_in_this_demand(), 2);
    let decisions = take_all_decisions();
    assert_eq!(
        decisions
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        ["anchor-b", "anchor-c"],
        "the requested middle returns before the initial consumer decides"
    );
    assert_eq!(
        application.producer_contacts_at_root_on_this_thread_for_test(root) - contacts,
        0,
        "the requested return makes no handler contact for the fresh unrelated claim"
    );
    settled_in_one_advance!(unrelated, request, "the still-fresh unrelated caller");
    assert_eq!(
        application.producer_contacts_at_root_on_this_thread_for_test(root) - contacts,
        1,
        "the uncontacted claim really needed fresh handler work"
    );
}

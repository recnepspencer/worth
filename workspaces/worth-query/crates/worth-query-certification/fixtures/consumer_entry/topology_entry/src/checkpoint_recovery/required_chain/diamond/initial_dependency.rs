//! An initial consumer must progress an exact pending native output it reads.
use super::*;
mod sibling_program;

#[test]
fn an_initial_consumer_refreshes_the_pending_output_it_actually_reads() {
    journey(false, false, false, false, None);
}

#[test]
fn an_initial_consumer_does_not_certify_a_cancelled_decision() {
    journey(true, false, false, false, None);
}

#[test]
fn a_cancelled_initial_consumer_releases_and_retries_its_cached_dependency() {
    journey(true, true, false, false, None);
}

#[test]
fn an_initial_consumer_readmits_the_cached_dependency_it_actually_reads() {
    journey(false, true, false, false, None);
}

#[test]
fn a_cached_read_keeps_custody_while_another_demand_delivers_its_pending_child() {
    journey(false, true, true, false, None);
}

#[test]
fn convenience_settlement_follows_two_original_requested_outputs() {
    journey(false, true, false, true, None);
}

#[test]
fn explicit_one_step_returns_original_progress_before_later_same_request_completion() {
    journey(false, true, false, true, Some(std::num::NonZeroUsize::MIN));
}

fn journey(
    cancel_decision: bool,
    close_dependencies: bool,
    delay_child: bool,
    both_changed: bool,
    steps: Option<std::num::NonZeroUsize>,
) {
    let _guard = checkpoint_recovery_test_guard();
    let provider_observation = crate::producer::ProviderContacts::default();
    take_decisions("diamond-join");
    let profile =
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard();
    let application = support::observed::install_program_with_seed::<program::ChainProgram>(
        None,
        profile,
        4096,
        seed,
        &provider_observation,
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
    let controls = match steps {
        Some(steps) => WorthQueryOutputDemandControls::default().settlement_attempts(steps),
        None => WorthQueryOutputDemandControls::default(),
    };
    let mut join = request
        .demand(ChainDemand("diamond-join".into()))
        .controls(controls)
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&application)
        .unwrap();
    change_root!(request, application, "diamond-left", 5, 0x9176_3590_u64);
    if both_changed {
        change_root!(request, application, "diamond-right", 55, 0x9176_3592_u64);
    }
    provider_observation.reset();
    if close_dependencies {
        drop((left, right));
    }
    if delay_child {
        application.delay_next_output_readiness_delivery_for_test();
        assert!(matches!(
            join.settle(&request).unwrap(),
            WorthQueryApplicationOutputDemandProgress::Pending
        ));
        assert!(take_decisions("diamond-join").is_empty());
        assert_eq!(output_lengths!(request, ["diamond-left"]), [6]);
        provider_observation.reset();
        // A distinct authentic caller delivers the already performed child.
        // The initial consumer's temporary claim must survive its Pending.
        let mut delivered = request
            .demand(PlanarOutputDemand::new("diamond-left"))
            .start_in_program::<program::ChainProgram, program::ChainRoot>(&application)
            .unwrap();
        settle!(delivered, request);
        assert_eq!(provider_observation.count(), 0);
        drop(delivered);
    }
    if cancel_decision {
        let cancellation = authentication::WorthQueryCancellationSource::new();
        let cancelled_scope =
            authentication::WorthQueryRequestScope::new(None, cancellation.token());
        super::super::binding::cancel_during_next_decision("diamond-join", cancellation);
        let cancelled = application.request(&principal, &cancelled_scope);
        let stopped = join.settle(&cancelled);
        assert!(
            matches!(&stopped,
            Err(worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(denial))
                if denial.kind() == WorthQueryOutputDemandDenialKind::Cancelled),
            "{:?}",
            stopped.as_ref().err()
        );
        assert_eq!(take_decisions("diamond-join"), [[6, 51]]);
        assert_eq!(output_lengths!(request, ["diamond-left"]), [6]);
        provider_observation.reset();
    }
    let first = join.settle(&request).unwrap();
    let initial = match (steps, first) {
        (None, WorthQueryApplicationOutputDemandProgress::Settled(initial)) => initial,
        (Some(_), WorthQueryApplicationOutputDemandProgress::Progressed) => loop {
            match join.settle(&request).unwrap() {
                WorthQueryApplicationOutputDemandProgress::Settled(initial) => break initial,
                WorthQueryApplicationOutputDemandProgress::Progressed => {}
                WorthQueryApplicationOutputDemandProgress::Pending => {
                    panic!("no external wait remains")
                }
            }
        },
        _ => panic!("original consumer did not honor its explicit settlement step policy"),
    };
    assert_eq!(
        initial.producer_contacts_in_this_demand(),
        if cancel_decision || both_changed {
            3
        } else {
            2
        },
        "failed and completed producer attempts are counted honestly"
    );
    let right_value = if both_changed { 56 } else { 51 };
    assert_eq!(take_decisions("diamond-join"), [[6, right_value]]);
    assert_eq!(
        output_lengths!(request, ["diamond-left", "diamond-right"]),
        [6, right_value]
    );
    if cancel_decision {
        assert_eq!(
            provider_observation.count(),
            0,
            "the completed child is reused after cancellation"
        );
    } else if !delay_child {
        assert!(provider_observation.count() > 0);
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

// The earlier C read really waits for B's deferred native readiness; the later
// sibling B owns the exact settlement which makes that read runnable.
macro_rules! sibling_tree {
    ($program:ty, $root:ty, $connection:ty, $park_after_settlement:expr) => {{
        let _guard = checkpoint_recovery_test_guard();
        take_all_decisions();
        let mut application = support::install_program_with_seed::<$program>(
            None, Default::default(), 4096,
            super::super::source_world::seed,
        );
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        let mut a = request.demand(PlanarOutputDemand::new("anchor-a"))
            .start_in_program::<$program, $root>(&application).unwrap();
        settle!(a, request);
        let mut b = request.demand(ChainDemand("anchor-b".into()))
            .start_dependent_in_program::<$program, $connection>(&application).unwrap();
        settle!(b, request);
        assert_eq!(take_decisions("anchor-b"), [[2]]);
        drop((a,b));
        let source = request.query(PlanarRead { body_key: "anchor-a".into() })
            .execute().unwrap().observed_sources()[0].clone();
        let changed = request.advanced().mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".into(), replacement_y: length(5),
        }).expect_source(source).idempotency(&0x9176_3593_u64)
            .execute_performed::<$program, $root>(&application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation).unwrap();
        assert!(matches!(changed, WorthQueryApplicationPerformedMutationOutcome::Performed(_)));
        drop(changed);
        let mut a = request.demand(PlanarOutputDemand::new("anchor-a"))
            .start_in_program::<$program, $root>(&application).unwrap();
        settle!(a, request);
        assert_eq!(output_lengths!(request, ["anchor-a"]), [6]);
        drop(a);
        take_all_decisions();
        let mut original = request.advanced().start_program_outputs::<$program, $root>(
            &application, PlanarOutputDemand::new("anchor-a"), Default::default(),
        ).unwrap();
        assert!(matches!(original.advance(&application, &request).unwrap(),
            WorthQueryApplicationProgramOutputProgress::Progressed));
        assert!(original.settled_root_observation().is_some(),
            "the root returns before independent sibling discovery");
        assert!(take_all_decisions().is_empty());
        application.delay_next_output_readiness_delivery_for_test();
        let first = if $park_after_settlement {
            match original.advance(&application, &request).unwrap() {
                WorthQueryApplicationProgramOutputProgress::Progressed => {},
                _ => panic!("newly accepted later B must report one-time progress"),
            }
            // B is now complete and retained. Defer C's own native readiness:
            // revisiting B must not manufacture progress on this waiting pass.
            application.delay_next_output_readiness_delivery_for_test();
            assert!(matches!(original.advance(&application, &request).unwrap(),
                WorthQueryApplicationProgramOutputProgress::Progressed),
                "B's newly retained leaf finishes its effect-free record once");
            application.delay_next_output_readiness_delivery_for_test();
            match original.advance(&application, &request).unwrap() {
                WorthQueryApplicationProgramOutputProgress::Pending => {},
                WorthQueryApplicationProgramOutputProgress::Progressed => panic!("completed B replayed progress during C's actual wait"),
                WorthQueryApplicationProgramOutputProgress::Settled(_) => panic!("C's delayed native readiness must wait"),
            }
            original.settle(&application, &request).unwrap()
        } else {
            original.settle(&application, &request).unwrap()
        };
        let prematurely_pending = matches!(&first, WorthQueryApplicationProgramOutputProgress::Pending);
        let done = match first {
            WorthQueryApplicationProgramOutputProgress::Settled(done) => done,
            WorthQueryApplicationProgramOutputProgress::Pending => {
                assert_eq!(take_all_decisions(), [("anchor-b".to_owned(), vec![6])],
                    "later B performed while C still lacked current B");
                match original.settle(&application, &request).unwrap() {
                    WorthQueryApplicationProgramOutputProgress::Settled(done) => done,
                    _ => panic!("the same original tree must complete once B's exact settlement is retained"),
                }
            }
            WorthQueryApplicationProgramOutputProgress::Progressed => panic!("absent step policy must follow progress"),
        };
        assert_eq!(output_lengths!(request, ["anchor-a", "anchor-b", "anchor-c"]), [6,16,11]);
        let decisions = take_all_decisions();
        if prematurely_pending {
            assert_eq!(decisions, [("anchor-c".to_owned(), vec![16])]);
        } else {
            assert_eq!(decisions, [("anchor-b".to_owned(), vec![6]), ("anchor-c".to_owned(), vec![16])]);
        }
        assert_eq!(done.output_count(), 2);
        drop((done, original));
        application.close_conditional_runtime().unwrap();
        assert!(!prematurely_pending, "tree lost newly settled later sibling progress");
    }};
}
#[test]
fn later_edge_member_settlement_unblocks_earlier_consumer_in_same_settle() {
    sibling_tree!(
        sibling_program::EdgeProgram,
        sibling_program::EdgeRoot,
        sibling_program::ReverseConnection,
        false
    );
}
#[test]
fn later_branch_settlement_unblocks_earlier_consumer_in_same_settle() {
    sibling_tree!(
        sibling_program::BranchProgram,
        sibling_program::BranchRoot,
        sibling_program::LaterConnection,
        false
    );
}

#[test]
fn already_settled_edge_member_does_not_replay_progress_during_later_wait() {
    sibling_tree!(
        sibling_program::EdgeProgram,
        sibling_program::EdgeRoot,
        sibling_program::ReverseConnection,
        true
    );
}

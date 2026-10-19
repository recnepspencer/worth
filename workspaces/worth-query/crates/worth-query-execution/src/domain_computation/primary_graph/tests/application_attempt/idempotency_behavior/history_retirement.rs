use super::*;

#[test]
fn a_causal_replay_after_its_history_retires_answers_with_its_sealed_causality() {
    {
        use crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality;

        let world = installed_authorization_world(true);

        // The original commit, later commits, and replay are separate host calls.
        let request = live_scope();
        let principal = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                authenticated_principal(
                    &(host_custody.phase_for_application(&world.application)).test_host_request(),
                    &world,
                    &request,
                )
            })
            .expect("the fixture read owns its installed host call");
        let account = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                resolved_account(
                    &(host_custody.phase_for_application(&world.application)).test_host_request(),
                    &world,
                    "open",
                    &request,
                )
            })
            .expect("the fixture read owns its installed host call");
        let parent = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                world
                    .selected_product(
                        &(host_custody.phase_for_application(&world.application))
                            .test_host_request(),
                    )
                    .product()
                    .relational_basis()
                    .observation()
                    .commit_receipt()
                    .cloned()
                    .expect("fixture head")
            })
            .expect("the fixture read owns its installed host call");
        let pending = WorthQueryPendingAftermathCausality::undo_of(parent);
        let program = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                admitted_program(
                    &host_custody.phase_for_application(&world.application),
                    &world,
                    &principal,
                    &account,
                    &request,
                    "undone",
                )
            })
            .expect("the fixture read owns its installed host call");
        let WorthQueryApplicationCommitOutcome::Committed(original) = world
            .application
            .with_application_advancement(&request, |phase| {
                world
                    .application
                    .compare_and_commit_application_with_aftermath(
                        &phase,
                        program,
                        idempotency(41, 41),
                        pending.clone(),
                        ExecutionAllocationPolicy::SystemAllocation,
                    )
            })
            .unwrap()
        else {
            panic!("the causal commit lands");
        };
        let causality = original
            .aftermath_causality()
            .cloned()
            .expect("the fresh receipt carries its causality");
        let causal_commit = original
            .committed_product_publication()
            .composite_commit()
            .clone();
        drop(original);

        for (key, from, to) in [(43, "undone", "later-1"), (45, "later-1", "later-2")] {
            let principal = world
                .application
                .with_host_advancement(|phase| {
                    let host_custody = phase.test_host_request();
                    authenticated_principal(
                        &(host_custody.phase_for_application(&world.application))
                            .test_host_request(),
                        &world,
                        &request,
                    )
                })
                .expect("the fixture read owns its installed host call");
            let account = world
                .application
                .with_host_advancement(|phase| {
                    let host_custody = phase.test_host_request();
                    resolved_account(
                        &(host_custody.phase_for_application(&world.application))
                            .test_host_request(),
                        &world,
                        from,
                        &request,
                    )
                })
                .expect("the fixture read owns its installed host call");
            let program = world
                .application
                .with_host_advancement(|phase| {
                    let host_custody = phase.test_host_request();
                    admitted_program(
                        &host_custody.phase_for_application(&world.application),
                        &world,
                        &principal,
                        &account,
                        &request,
                        to,
                    )
                })
                .expect("the fixture read owns its installed host call");
            assert!(matches!(
                world.application.compare_and_commit_application(
                    program,
                    idempotency(key, key),
                    ExecutionAllocationPolicy::SystemAllocation
                ),
                WorthQueryApplicationCommitOutcome::Committed(_)
            ));
        }
        let history = world
            .application
            .with_host_advancement(|phase| {
                world.application.branches().history(
                    &phase,
                    world.application.current_world(),
                    std::num::NonZeroUsize::new(64),
                )
            })
            .expect("the original installed host opens a fresh phase")
            .expect("the live branch exposes its history");
        assert!(
            history
                .entries()
                .all(|entry| entry.selected_commit() != &causal_commit),
            "two later publications retired the causal commit's history"
        );
        drop(history);

        let principal = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                authenticated_principal(
                    &(host_custody.phase_for_application(&world.application)).test_host_request(),
                    &world,
                    &request,
                )
            })
            .expect("the fixture read owns its installed host call");
        let account = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                resolved_account(
                    &(host_custody.phase_for_application(&world.application)).test_host_request(),
                    &world,
                    "later-2",
                    &request,
                )
            })
            .expect("the fixture read owns its installed host call");
        let replay = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                admitted_program(
                    &host_custody.phase_for_application(&world.application),
                    &world,
                    &principal,
                    &account,
                    &request,
                    "undone",
                )
            })
            .expect("the fixture read owns its installed host call");
        let WorthQueryApplicationCommitOutcome::AlreadyCommitted(recovered) = world
            .application
            .with_application_advancement(&request, |phase| {
                world
                    .application
                    .compare_and_commit_application_with_aftermath(
                        &phase,
                        replay,
                        idempotency(41, 41),
                        pending,
                        ExecutionAllocationPolicy::SystemAllocation,
                    )
            })
            .unwrap()
        else {
            panic!("a causal replay after retirement answers its commit, never intent drift");
        };
        assert_eq!(recovered.aftermath_causality(), Some(&causality));
    }
}

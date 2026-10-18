//! Native commit-time basis staleness never turns an original row loan into an unpinned retry.
use super::*;
use std::{num::NonZeroUsize, time::Duration};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitDenialStage,
    WorthQueryApplicationUncommitted,
};

#[test]
fn source_only_pinned_commit_time_staleness_retains_terminal_original_loan() {
    let program = ApplicationProgramAuthoring::<ConsumerSchema, PinnedSourceProgram>::begin()
        .validated_program()
        .unwrap();
    let world = installation::install_selected_program(
        &crate::installed_schema(),
        8192,
        4096,
        worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster::new(
        ),
        &[],
        program,
    )
    .unwrap();
    let adapter = authentication::admit(world.application.installed_schema());
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    let input = PlanarSourceAdjustment {
        scope_key: "anchor-a".to_owned(),
        replacement_y: length(7),
    };
    reset_planar_source_adjustment_contacts(&input);
    let observed = request
        .query(PlanarRead {
            body_key: input.scope_key.clone(),
        })
        .execute()
        .unwrap();
    let original = request
        .mutate(input.clone())
        .expect_source(observed.observed_sources()[0].clone())
        .idempotency_owned(10_211)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .retain_operation::<PinnedSourceProgram, PinnedAction>(&world.application)
        .unwrap_or_else(|(_, cause)| panic!("actual original row constructs: {cause:?}"));
    drop(observed);
    drop(principal);
    drop(scope);
    let pause = world
        .application
        .runtime()
        .pause_after_application_attempt_registration_for_test(NonZeroUsize::new(1).unwrap());
    let terminal = std::thread::scope(|threads| {
        let attempt = threads.spawn(|| {
            let scope = authentication::request_scope();
            let principal = authentication::block_on(adapter.authenticate(
                authentication::LocalCredential::issued_for_model_owner(),
                &scope,
            ))
            .unwrap();
            reset_planar_source_adjustment_contacts(&input);
            let outcome = original.advance(&world.application.request(&principal, &scope));
            (outcome, planar_source_adjustment_contacts(&input))
        });
        assert!(pause.wait_until_reached(Duration::from_secs(10)));
        let scope = authentication::request_scope();
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        let request = world.application.request(&principal, &scope);
        let observed = request
            .query(PlanarRead {
                body_key: input.scope_key.clone(),
            })
            .execute()
            .unwrap();
        let competitor = request
            .mutate(PlanarSourceAdjustment {
                scope_key: input.scope_key.clone(),
                replacement_y: length(3),
            })
            .expect_source(observed.observed_sources()[0].clone())
            .idempotency_owned(10_212)
            .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
            .retain_operation::<PinnedSourceProgram, PinnedAction>(&world.application)
            .unwrap_or_else(|(_, cause)| panic!("real competitor constructs: {cause:?}"));
        match competitor.advance(&request) {
            Advance::Completed(done) => {
                done.close(&request)
                    .unwrap_or_else(|(_, cause)| panic!("competitor close: {cause:?}"));
            }
            _ => panic!("competing original row publishes"),
        }
        let before = world
            .application
            .with_host_advancement(|phase| request.retain_read(&phase).unwrap())
            .unwrap();
        pause.release();
        let (outcome, original_contacts) = attempt.join().unwrap();
        assert_eq!(original_contacts, (1, 1));
        let terminal = match outcome {
            Advance::Terminal(terminal) => terminal,
            _ => panic!("pinned commit-time stale source cannot reexecute"),
        };
        assert!(
            matches!(terminal.native_outcome(), Some(worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::Commit(
            WorthQueryApplicationUncommitted::Denied(cause))) if cause.kind() == WorthQueryApplicationCommitDenialKind::ProductBasisStale
                && cause.stage() == WorthQueryApplicationCommitDenialStage::InvariantExecution)
        );
        let after = world
            .application
            .with_host_advancement(|phase| request.retain_read(&phase).unwrap())
            .unwrap();
        assert_eq!(before.selected_commit(), after.selected_commit());
        assert_eq!(
            request
                .query(PlanarRead {
                    body_key: input.scope_key.clone()
                })
                .execute()
                .unwrap()
                .rows()[0]
                .y,
            length(3)
        );
        terminal
    });
    let fresh_scope = authentication::request_scope();
    let fresh_principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &fresh_scope,
    ))
    .unwrap();
    terminal
        .close(&world.application.request(&fresh_principal, &fresh_scope))
        .unwrap_or_else(|(_, cause)| panic!("terminal original row loan closes: {cause:?}"));
    // This source was fresh when its decision ran, then genuinely lost commit currentness.
    assert_eq!(
        planar_source_adjustment_contacts(&input),
        (0, 0),
        "the main-thread recorder observed no original handler execution"
    );
    assert!(world
        .application
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .is_empty());
}

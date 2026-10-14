//! Terminal actual World no-effect custody closes without declaring outputs complete.
use super::*;
use crate::application_program::ConsumerFixedAction;
use std::time::Duration;
use worth_query_host::facade::product::WorthQueryApplicationNoEffectCause;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome, WorthQueryApplicationReadObservationDenial,
        WorthQueryOperationAdvance as Advance, WorthQueryOperationCloseDenial,
    },
    primary_graph::{
        WorthQueryApplicationUncommitted as U, WorthQueryPrincipalResolutionMode,
        WorthQueryProductBranchAdmissionDenial, WorthQueryProductBranchAdmissionRefusalKind,
    },
};
use worth_query_topology_entry::{
    planar_source_adjustment_contacts, reset_planar_source_adjustment_contacts, BodyKey, PositionY,
};
#[derive(Clone, Copy)]
enum Interference {
    NativeWrite,
    ObservationCapacity,
}
#[test]
fn fixed_product_stale_terminal_closes_without_output_completion() {
    terminal_court(Interference::NativeWrite);
}
#[test]
fn fixed_no_effect_capacity_terminal_closes_without_output_completion() {
    terminal_court(Interference::ObservationCapacity);
}
fn terminal_court(interference: Interference) {
    let world = installation::install(&crate::installed_schema());
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
        replacement_y: length(2),
    };
    reset_planar_source_adjustment_contacts(&input);
    let before = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let source = request
        .query(PlanarRead {
            body_key: input.scope_key.clone(),
        })
        .execute()
        .unwrap();
    let initial_y = source.rows()[0].y;
    let original = request
        .mutate(input.clone())
        .expect_source(source.observed_sources()[0].clone())
        .idempotency_owned(10_641)
        .retain_operation::<crate::ConsumerProgram, ConsumerFixedAction>(&world.application)
        .unwrap_or_else(|(_, cause)| panic!("descriptor: {cause:?}"));
    drop(source);
    let record = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(world.application.current_world())
                .select(&phase)
                .unwrap()
                .resolve_entity(
                    &phase,
                    BodyKey::reference::<ConsumerSchema>(),
                    input.scope_key.clone(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Certification,
                )
                .unwrap()
                .relational_record_identity_parts()
        })
        .unwrap();
    let pause = world
        .application
        .pause_after_application_candidate_preparation_for_test(NonZeroUsize::MIN);
    let terminal = std::thread::scope(|threads| {
        let (release, released) = std::sync::mpsc::sync_channel::<()>(0);
        let application = &world.application;
        let adapter = &adapter;
        let input = &input;
        let pause = &pause;
        let before = &before;
        let interfere = threads.spawn(move || {
            assert!(pause.wait_until_reached(Duration::from_secs(10)));
            let scope = authentication::request_scope();
            let principal = authentication::block_on(adapter.authenticate(authentication::LocalCredential::issued_for_model_owner(), &scope)).unwrap();
            let request = application.request(&principal, &scope);
            let mut reads = Vec::new();
            let winner = match interference {
                Interference::NativeWrite => {
                    application.publish_native_field_write_for_test(application.current_world(), record, PositionY::reference::<ConsumerSchema>(), length(4), &scope).unwrap();
                    request.retain_current_read().unwrap().selected_commit().clone()
                }
                Interference::ObservationCapacity => {
                    let mut exhausted = false;
                    for _ in 0..=crate::application_invariant_acceptance::resources::ACTIVE_WORLD_OBSERVATIONS {
                        match request.retain_current_read() {
                            Ok(read) => reads.push(read),
                            Err(WorthQueryApplicationReadObservationDenial::ProductSelection(WorthQueryProductBranchAdmissionDenial::Refused(WorthQueryProductBranchAdmissionRefusalKind::ObservationCapacityExhausted))) => {exhausted=true; break;}
                            Err(other) => panic!("pressure stopped at another owner: {other:?}"),
                        }
                    }
                    assert!(exhausted);
                    before.clone()
                }
            };
            pause.release();
            // Real observation custody stays on its issuing thread until the native attempt returns.
            let _ = released.recv();
            drop(reads);
            winner
        });
        let outcome = original.advance(&request);
        assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
        release.send(()).unwrap_or_else(|_| match &outcome {
            Advance::Terminal { operation, denial } => panic!(
                "actual source stopped before World seam: {denial:?}, native {:?}",
                operation.native_outcome()
            ),
            Advance::Refused { denial, .. } => {
                panic!("actual admission stopped before World seam: {denial:?}")
            }
            _ => panic!("World interference did not rendezvous"),
        });
        let winner = interfere.join().unwrap();
        let terminal = match outcome {
            Advance::Terminal { operation, .. } => operation,
            _ => panic!("native no-effect cannot settle outputs"),
        };
        match interference {
            Interference::NativeWrite => assert!(
                matches!(terminal.native_outcome(), Some(WorthQueryApplicationMutationOutcome::Commit(U::ProductStale(stale)))
                if stale.expected_product().selected_commit() == before && stale.observed_product().is_none())
            ),
            Interference::ObservationCapacity => assert!(
                matches!(terminal.native_outcome(), Some(WorthQueryApplicationMutationOutcome::Commit(U::NoEffect(cause)))
                if cause.cause() == WorthQueryApplicationNoEffectCause::CapacityExhausted)
            ),
        }
        assert_eq!(
            request.retain_current_read().unwrap().selected_commit(),
            &winner
        );
        assert_eq!(
            request
                .query(PlanarRead {
                    body_key: input.scope_key.clone()
                })
                .execute()
                .unwrap()
                .rows()[0]
                .y,
            match interference {
                Interference::NativeWrite => length(4),
                Interference::ObservationCapacity => initial_y,
            }
        );
        terminal
    });
    assert_eq!(world.application.active_application_attempts_for_test(), 0);
    assert_eq!(
        world
            .application
            .world_active_publication_attempts_for_test(),
        0
    );
    assert!(world
        .application
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .is_empty());
    let stop = worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource::new();
    stop.cancel();
    let stopped =
        worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope::new(
            None,
            stop.token(),
        );
    let (terminal, cause) = terminal
        .close(&world.application.request(&principal, &stopped))
        .err()
        .expect("stopped close retains actual terminal");
    assert!(matches!(
        cause,
        WorthQueryOperationCloseDenial::Execution(_)
    ));
    assert!(terminal.native_outcome().is_some());
    drop(request);
    drop(principal);
    drop(scope);
    let fresh_scope = authentication::request_scope();
    let fresh_principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &fresh_scope,
    ))
    .unwrap();
    terminal
        .close(&world.application.request(&fresh_principal, &fresh_scope))
        .unwrap_or_else(|(_, cause)| panic!("fresh no-effect close: {cause:?}"));
    assert_eq!(
        planar_source_adjustment_contacts(&input),
        (1, 1),
        "the fixed owner remains on the executing main thread"
    );
}

//! Pinned row/result-set facts are checked before the projected handler.
use super::*;
use crate::application_program::pinned_source::{PinnedAction, PinnedSourceProgram};
use worth_query_decl::facade::application_program::ApplicationProgramAuthoring;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestMutationDenial, WorthQueryOperationCloseDenial,
    WorthQuerySourceOnlyAdvance as Advance,
};
use worth_query_host::facade::primary_graph::{
    MutationHandlerExecutionDenial, WorthQueryApplicationAttemptDenialKind,
};
use worth_query_topology_entry::{
    planar_source_adjustment_contacts, reset_planar_source_adjustment_contacts,
};

#[test]
fn source_only_pinned_row_moved_is_terminal() {
    journey(false, 1);
}
#[test]
fn source_only_pinned_result_set_unconsumed_value_move_completes() {
    journey(true, 1);
}
#[test]
fn source_only_pinned_row_unchanged_completes() {
    journey(false, 0);
}
#[test]
fn source_only_pinned_result_set_unchanged_completes() {
    journey(true, 0);
}

#[test]
fn source_only_pinned_row_retired_child_is_terminal() {
    journey(false, 2);
}

fn journey(result_set: bool, movement: u8) {
    let program = ApplicationProgramAuthoring::<ConsumerSchema, PinnedSourceProgram>::begin()
        .validated_program()
        .unwrap();
    let world = installation::install_selected_program(
        &crate::installed_schema(),
        worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster::new(
        ),
        &[],
        program,
    )
    .unwrap();
    let adapter = authentication::admit(world.application.installed_schema());
    let input = PlanarSourceAdjustment {
        scope_key: "anchor-a".to_owned(),
        replacement_y: length(7),
    };
    reset_planar_source_adjustment_contacts(&input);
    let operation = {
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
        let description = request.mutate(input.clone());
        let description = if result_set {
            description.expect_result_set(observed.result_set_observation().unwrap().clone())
        } else {
            description.expect_source(observed.observed_sources()[0].clone())
        };
        description
            .idempotency_owned(10_201)
            .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
            .retain_operation::<PinnedSourceProgram, PinnedAction>(&world.application)
            .unwrap_or_else(|(_, cause)| panic!("actual no-output action: {cause:?}"))
    };
    // Only the operation now owns the initiating observation and original domain input.
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    if movement == 1 {
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
            .idempotency_owned(10_202)
            .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
            .retain_operation::<PinnedSourceProgram, PinnedAction>(&world.application)
            .unwrap_or_else(|(_, cause)| panic!("competing original owner: {cause:?}"));
        match competitor.advance(&request) {
            Advance::Completed(done) => {
                done.close(&request)
                    .unwrap_or_else(|(_, cause)| panic!("competitor close: {cause:?}"));
            }
            _ => panic!("actual competing source commits"),
        }
    }
    if movement == 2 {
        use worth_query_consumer_values::{PlanarVertex, PlanarVertexReplacement};
        use worth_query_topology_entry::VertexReplacement;
        let observed = request
            .query(PlanarRead {
                body_key: input.scope_key.clone(),
            })
            .execute()
            .unwrap();
        let outcome = request
            .advanced()
            .mutate(VertexReplacement {
                scope_key: input.scope_key.clone(),
                replacement: PlanarVertexReplacement {
                    retired_key: "anchor-b".to_owned(),
                    next_key: "anchor-c".to_owned(),
                    replacement: PlanarVertex {
                        body_key: "replacement-b".to_owned(),
                        x: length(11),
                        y: length(1),
                    },
                },
            })
            .expect_source(observed.observed_sources()[0].clone())
            .idempotency(&10_203_u64)
            .execute_in_program(
                &world.application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        assert!(matches!(outcome, worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::Committed { .. }));
    }
    let before = world
        .application
        .with_host_advancement(|phase| request.retain_read(&phase).unwrap())
        .unwrap();
    match operation.advance(&request) {
        Advance::Terminal(terminal) if movement == 2 || (movement == 1 && !result_set) => {
            assert!(matches!(terminal.native_admission_denial(),
                Some(WorthQueryApplicationRequestMutationDenial::Handler(MutationHandlerExecutionDenial::Attempt(cause)))
                if cause.kind() == if movement == 2 { WorthQueryApplicationAttemptDenialKind::SourceRetired } else { WorthQueryApplicationAttemptDenialKind::SourceChanged }));
            assert!(terminal.native_outcome().is_none());
            assert_eq!(planar_source_adjustment_contacts(&input), (0, 0));
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
                length(if movement == 2 { 1 } else { 3 })
            );
            let other = world
                .application
                .runtime()
                .with_host_advancement(|phase| {
                    world
                        .application
                        .runtime()
                        .branches()
                        .fork(world.application.current_world())
                        .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                        .create(&phase)
                        .unwrap()
                })
                .unwrap();
            let (terminal, denial) = terminal
                .close(&request.on_branch(other))
                .err()
                .expect("foreign close returns actual custody");
            assert!(matches!(
                denial,
                WorthQueryOperationCloseDenial::FreshRequestMismatch
            ));
            assert!(terminal.native_admission_denial().is_some());
            let fresh_scope = authentication::request_scope();
            let fresh_principal = authentication::block_on(adapter.authenticate(
                authentication::LocalCredential::issued_for_model_owner(),
                &fresh_scope,
            ))
            .unwrap();
            terminal
                .close(&world.application.request(&fresh_principal, &fresh_scope))
                .unwrap_or_else(|(_, cause)| panic!("fresh native loan release: {cause:?}"));
            assert_eq!(planar_source_adjustment_contacts(&input), (0, 0));
        }
        Advance::Completed(done) if movement == 0 || result_set => {
            assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
            assert_eq!(
                request
                    .query(PlanarRead {
                        body_key: input.scope_key.clone()
                    })
                    .execute()
                    .unwrap()
                    .rows()[0]
                    .y,
                length(7)
            );
            done.close(&request)
                .unwrap_or_else(|(_, cause)| panic!("valid close: {cause:?}"));
        }
        Advance::Refused { denial, .. } => panic!("unexpected retryable refusal: {denial:?}"),
        _ => panic!("native posture differs from source truth"),
    }
    assert!(world
        .application
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .is_empty());
}

mod commit_stale;

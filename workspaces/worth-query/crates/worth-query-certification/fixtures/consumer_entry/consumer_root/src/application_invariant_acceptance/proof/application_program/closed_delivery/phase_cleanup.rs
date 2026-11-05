//! Original disposal survives scope loss and uses the actual runtime's phase.
use super::*;
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
        WorthQueryManagedComputationResourceDenial,
    },
    application_entry::WorthQueryProgramOutputCloseDenial,
    product::{
        WorthQueryApplicationProductBranchCleanupDenial,
        WorthQueryApplicationProductBranchCloseDenial, WorthQueryProductBranchOwnerCleanupDenial,
    },
};

#[test]
fn original_delivery_cleanup_uses_owning_phase_after_scope_loss_and_keeps_close_only_retries() {
    let _observation = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let mut world = installation::install(&crate::installed_schema());
    let mut foreign = installation::install(&crate::installed_schema());
    let branch = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .branches()
                .fork(world.application.current_world())
                .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    let tree = {
        let scope = authentication::request_scope();
        let adapter = authentication::admit(world.application.installed_schema());
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        let tree = settle_tree(
            &world,
            &world
                .application
                .request(&principal, &scope)
                .on_branch(branch),
        );
        assert_eq!(1 + tree.settlement().output_count(), 5);
        tree
    };
    // No principal or application request survives this handoff.
    let contacts = world
        .application
        .producer_contacts_on_this_thread_for_test();
    let refused = foreign
        .application
        .with_host_advancement(|phase| {
            tree.close_for_delivery_in_advancement(&phase)
                .err()
                .expect("an identically typed foreign runtime cannot close this owner")
        })
        .unwrap();
    assert!(matches!(
        refused.denial(),
        WorthQueryProgramOutputCloseDenial::Execution(WorthQueryAdvancementDenial::Resource(
            WorthQueryManagedComputationResourceDenial::ForeignAdvancementPhase
        ))
    ));
    assert_eq!(refused.branch(), branch);
    let cancellation = WorthQueryCancellationSource::new();
    let stopped = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let mut retained = None;
    let _stopped_result = world
        .application
        .with_application_advancement(&stopped, |phase| {
            cancellation.cancel();
            // Keep the refused owner independently of the enclosing run's result.
            retained = Some(
                refused
                    .retry_close_in_advancement(&phase)
                    .err()
                    .expect("an already open cancelled phase cannot discharge the owner"),
            );
        });
    let refused = retained.expect("the exact close-only owner escapes the stopped phase");
    assert!(matches!(
        refused.denial(),
        WorthQueryProgramOutputCloseDenial::Execution(WorthQueryAdvancementDenial::Interrupted(
            WorthQueryManagedComputationInterruption::Cancelled
        ))
    ));
    let closed = world
        .application
        .with_host_advancement(|phase| {
            refused
                .retry_close_in_advancement(&phase)
                .unwrap_or_else(|stop| panic!("same original close retry: {:?}", stop.denial()))
        })
        .unwrap();
    assert_eq!(closed.branch(), branch);
    assert_eq!(1 + closed.settlement().output_count(), 5);
    assert_eq!(
        world
            .application
            .producer_contacts_on_this_thread_for_test(),
        contacts,
        "cleanup never contacts producers"
    );
    let pending = world
        .application
        .with_host_advancement(|phase| world.application.on_branch(branch).close(&phase))
        .unwrap();
    let pending = match pending {
        Err(WorthQueryApplicationProductBranchCloseDenial::OwnerCleanupPending(pending)) => pending,
        other => panic!("the closed delivery retains original readonly pins: {other:?}"),
    };
    assert!(matches!(
        pending.denial(),
        WorthQueryApplicationProductBranchCleanupDenial::Product(
            WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryStillRetained
        )
    ));
    drop(closed);
    assert!(pending
        .into_cleanup()
        .retry()
        .expect("final readonly pin released")
        .is_complete());
    foreign.application.close_conditional_runtime().unwrap();
    world.application.close_conditional_runtime().unwrap();
}

#[test]
fn unstarted_original_cleanup_refuses_cancellation_before_any_native_tree_exists() {
    let _observation = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let mut world = installation::install(&crate::installed_schema());
    let operation = {
        let scope = authentication::request_scope();
        let adapter = authentication::admit(world.application.installed_schema());
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        world
            .application
            .request(&principal, &scope)
            .demand(PlanarOutputDemand::new("anchor-a"))
            .retain_program_outputs::<ConsumerProgram, ConsumerProgramRoot>(&world.application)
            .unwrap_or_else(|stop| panic!("original construction: {:?}", stop.denial()))
    };
    let contacts = world
        .application
        .producer_contacts_on_this_thread_for_test();
    let cancellation = WorthQueryCancellationSource::new();
    let stopped = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let mut retained = None;
    let _stopped_result = world
        .application
        .with_application_advancement(&stopped, |phase| {
            cancellation.cancel();
            retained = Some(
                operation
                    .close_in_advancement(&phase)
                    .err()
                    .expect("empty native custody still requires a live owning phase"),
            );
        });
    let refused = retained.unwrap();
    assert!(matches!(
        refused.denial(),
        WorthQueryProgramOutputCloseDenial::Execution(WorthQueryAdvancementDenial::Interrupted(
            WorthQueryManagedComputationInterruption::Cancelled
        ))
    ));
    let closed = world
        .application
        .with_host_advancement(|phase| {
            refused
                .retry_close_in_advancement(&phase)
                .unwrap_or_else(|stop| panic!("original disposal retry: {:?}", stop.denial()))
        })
        .unwrap();
    assert!(closed.terminal_denial().is_none());
    assert!(closed.construction_denial().is_none());
    assert_eq!(
        world
            .application
            .producer_contacts_on_this_thread_for_test(),
        contacts
    );
    drop(closed);
    world.application.close_conditional_runtime().unwrap();
}

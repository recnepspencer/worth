pub use std::num::NonZeroUsize;

pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
pub use worth_query_execution::facade::product::WorthQueryProductBranchCreationRecoveryCause;
pub use worth_query_execution::facade::product::WorthQueryProductBranchCloseDenial;
pub use worth_query_execution::facade::product::WorthQueryProductBranchOwnerCleanupDenial;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationProductBranchCloseDenial;

#[test]
fn real_sibling_denial_is_inspectable_releasable_and_leaves_no_recovery_orphan() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
    let (recovery, occupied_branch) = crate::domain_computation::primary_graph::tests::product_branch_creation_recovery::fork_failure::create(&world, request_execution);
    let inspection = recovery
        .inspect()
        .expect("the retained owner effects remain inspectable");
    assert_eq!(
        inspection.cause(),
        WorthQueryProductBranchCreationRecoveryCause::SiblingOwnerDenied
    );
    assert_eq!(inspection.owner_effect_count(), 1);
    assert!(inspection.live_obligation_count() > 0);
    drop(inspection);
    drop(recovery);

    let creation_page = world
        .application
        .branches()
        .recovery_page(None, NonZeroUsize::new(1).unwrap())
        .expect("creation recovery remains discoverable after caller custody is dropped");
    let [creation_row] = creation_page.rows() else {
        panic!("the creation facade must rediscover the exact World recovery record")
    };
    let creation_recovery = world
        .application
        .branches()
        .readmit_recovery(creation_row.handle())
        .expect("the creation facade must readmit its own World record");
    assert_eq!(
        creation_recovery.inspect().unwrap().cause(),
        WorthQueryProductBranchCreationRecoveryCause::SiblingOwnerDenied
    );
    drop(creation_recovery);

    let generic_page = world
        .application
        .product_publication_recovery_page(None, NonZeroUsize::new(1).unwrap())
        .expect("the application recovery catalog remains available");
    let [generic_row] = generic_page.rows() else {
        panic!("the generic catalog must retain the dropped creation record")
    };
    let recovery = world
        .application
        .readmit_product_publication_recovery(generic_row.handle())
        .expect("the application facade must readmit the creation record without raw work access");

    let graph = (*world.application.runtime_for_test()).primary_graph().unwrap();
    let integration = graph.integration_handle_for_test();
    let active_transaction = integration.with_runtime(request_execution, |runtime| {
        let identity = runtime
            .branch_identity(&worth_relational::facade::history::BranchId(
                "query-product-1-relational".to_owned(),
            ))
            .expect("the performed Relational fork remains installed");
        let (_, basis) = runtime.observe_branch(&identity).unwrap();
        runtime
            .begin_branch_transaction(
                &basis,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .expect("a real operation retains the fork during cleanup")
    });
    let failure = world
        .application
        .release_product_publication_recovery(recovery, 0)
        .expect_err("the active Relational operation must defer owner cleanup");
    let cleanup = failure
        .into_owner_cleanup()
        .expect("the World record was released into managed Query cleanup");
    let failure = cleanup.retry().expect_err("the operation remains active");
    assert_eq!(
        failure.denial(),
        WorthQueryProductBranchOwnerCleanupDenial::RelationalOperationsStillActive
    );
    drop(failure);
    assert_eq!(world.application.branches().pending_cleanup().len(), 1);
    drop(active_transaction);
    let mut pending = world.application.branches().pending_cleanup();
    assert_eq!(pending.len(), 1);
    let cleanup = pending.remove(0);
    let released = cleanup
        .retry()
        .expect("the dropped handle must be rediscovered with exact owner authority");
    assert_eq!(released.retired_component_count(), 1);
    assert!(released.is_complete());
    assert!(world.application.branches().pending_cleanup().is_empty());
    let page = world
        .application
        .product_publication_recovery_page(None, NonZeroUsize::new(1).unwrap())
        .expect("bounded recovery discovery remains available");
    assert!(
        page.rows().is_empty(),
        "consuming Query recovery must remove the exact World recovery record"
    );

    let close = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(occupied_branch)
                .close(&phase)
                .expect("the collision fixture product branch remains explicitly closable")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(close.retired_component_count(), 1);
    assert!(close.is_complete());
}

#[test]
fn cleanup_capacity_denial_precedes_world_retirement() {
    let world = installed_authorization_world(true);
    let branch = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .branches()
                .fork(world.application.current_world())
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .expect("the forked product must publish")
        })
        .expect("the fixture read owns its installed host call");
    let mut held = Vec::new();
    while let Ok(reservation) = (*world
        .application
        .product_runtime_for_test())
        .reserve_owner_cleanup_for_creation_for_test()
    {
        held.push(reservation);
    }

    world
        .application
        .with_host_advancement(|phase| {
            assert!(matches!(
                world.application.on_branch(branch).close(&phase,),
                Err(WorthQueryApplicationProductBranchCloseDenial::Product(
                    WorthQueryProductBranchCloseDenial::CleanupCapacityExhausted
                ))
            ))
        })
        .expect("the fixture read owns its installed host call");
    let selected = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(branch)
                .select(&phase)
                .expect("capacity denial must leave the World occurrence active")
        })
        .expect("the fixture read owns its installed host call");
    drop(selected);
    drop(held);
    world
        .application
        .with_host_advancement(|phase| {
            assert!(world
                .application
                .on_branch(branch)
                .close(&phase,)
                .expect("released cleanup capacity permits the original close")
                .is_complete())
        })
        .expect("the fixture read owns its installed host call");
}

#[test]
fn unwind_after_retirement_install_leaves_discoverable_cleanup() {
    let world = installed_authorization_world(true);
    let branch = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .branches()
                .fork(world.application.current_world())
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .expect("the forked product must publish")
        })
        .expect("the fixture read owns its installed host call");
    (*world
        .application
        .product_runtime_for_test())
        .replace_owner_cleanup_after_install_hook_for_test(Some(Box::new(|| {
            panic!("injected unwind after retirement cleanup installation")
        })));
    let unwind = world
        .application
        .with_host_advancement(|phase| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _ = world.application.on_branch(branch).close(&phase);
            }))
        })
        .expect("the fixture read owns its installed host call");
    assert!(unwind.is_err());
    (*world
        .application
        .product_runtime_for_test())
        .replace_owner_cleanup_after_install_hook_for_test(None);
    world
        .application
        .with_host_advancement(|phase| {
            assert!(world.application.on_branch(branch).select(&phase,).is_err())
        })
        .expect("the fixture read owns its installed host call");
    assert!(
        world
            .application
            .product_runtime()
            .product_branches()
            .pending_cleanup()
            .is_empty(),
        "the outer lifecycle must not claim application retirement custody"
    );

    let mut pending = world.application.branches().pending_cleanup();
    assert_eq!(pending.len(), 1);
    let receipt = pending
        .remove(0)
        .retry()
        .expect("the preallocated installed entry survives the unwind");
    assert_eq!(receipt.retired_component_count(), 1);
    assert!(world.application.branches().pending_cleanup().is_empty());
}

#[test]
fn retry_unwind_restores_progress_without_repeating_owner_retirement() {
    let world = installed_authorization_world(true);
    let branch = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .branches()
                .fork(world.application.current_world())
                .components(|components| components.fork_relational().fork_signal())
                .create(&phase)
                .expect("the two-component product must publish")
        })
        .expect("the fixture read owns its installed host call");
    (*world
        .application
        .product_runtime_for_test())
        .replace_owner_cleanup_after_component_progress_hook(Some(Box::new(|| {
            panic!("injected unwind after one owner retirement")
        })));
    let failure = world
        .application
        .with_host_advancement(|phase| world.application.on_branch(branch).close(&phase))
        .expect("the fixture read owns its installed host call")
        .expect_err("the original Execution request classifies the cleanup unwind");
    let WorthQueryApplicationProductBranchCloseDenial::OwnerCleanupPending(failure) = failure
    else {
        panic!("Native progress must remain in an inspectable cleanup obligation");
    };
    assert!(matches!(
        failure.denial(),
        worth_query_execution::facade::product::WorthQueryApplicationProductBranchCleanupDenial::Product(
            WorthQueryProductBranchOwnerCleanupDenial::Execution(
                worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial::Panicked
            )
        )
    ));
    assert!(matches!(
        failure.into_cleanup().pending_work().as_slice(),
        [worth_query_execution::facade::product::WorthQueryProductBranchOwnerCleanupWork::SignalBranch { .. }]
    ));
    (*world
        .application
        .product_runtime_for_test())
        .replace_owner_cleanup_after_component_progress_hook(None);

    let mut pending = world.application.branches().pending_cleanup();
    assert_eq!(pending.len(), 1);
    let receipt = pending
        .remove(0)
        .retry()
        .expect("retry must continue after the component already retired");
    assert_eq!(receipt.retired_component_count(), 2);
    assert!(world.application.branches().pending_cleanup().is_empty());
}

#[path = "creation_release/mod.rs"]
pub mod creation_release;
#[path = "fork_failure/mod.rs"]
pub mod fork_failure;

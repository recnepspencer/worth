use worth_query_host::facade::product::{
    WorthQueryApplicationProductBranchCloseDenial, WorthQueryProductBranchAdmissionDenial,
    WorthQueryProductBranchCloseDenial, WorthQueryProductBranchOwnerCleanupDenial,
};

use super::world::CourtroomWorld;

#[test]
fn public_transaction_reuses_its_admitted_selection_at_snapshot_capacity() {
    let world = CourtroomWorld::publish_with_active_snapshot_limit("ready", 4);
    let branch = world.application.current_world();
    let observer = world.application.application_query_basis_observer();
    let before = observer.observe();

    let committed = world
        .change_input_on_branch(branch, "single-admitted-selection")
        .require_committed()
        .expect("the carried selection must commit without opening another snapshot");

    assert_eq!(committed.product_branch(), branch);
    assert_eq!(
        observer.observe().acquisitions() - before.acquisitions(),
        4,
        "the admitted mutation path must not add a fifth basis acquisition by reselecting at commit"
    );
}

#[test]
fn foreign_and_retired_tokens_fail_before_query_basis_work() {
    let world = CourtroomWorld::publish("ready");
    let foreign = CourtroomWorld::publish("ready");
    let observer = world.application.application_query_basis_observer();

    let before_foreign = observer.observe();
    let foreign_denial = world
        .application
        .on_branch(foreign.application.current_world())
        .select()
        .err()
        .expect("a branch token from another World owner must be refused");
    assert_eq!(
        foreign_denial,
        WorthQueryProductBranchAdmissionDenial::ForeignOwner
    );
    assert_eq!(
        observer.observe().acquisitions(),
        before_foreign.acquisitions(),
        "foreign-token denial must precede Query basis acquisition"
    );

    let source = world.application.current_world();
    let retired = world
        .application
        .branches()
        .fork(source)
        .components(|components| {
            components
                .reuse_exact_relational_basis()
                .reuse_exact_signal_basis()
        })
        .create()
        .expect("the owned reuse branch must publish");
    let close = world
        .application
        .on_branch(retired)
        .close()
        .expect("an unretained public product can close");
    assert!(close.is_complete());

    let before_retired = observer.observe();
    let retired_denial = world
        .application
        .on_branch(retired)
        .select()
        .err()
        .expect("a retired occurrence token must be refused");
    assert_eq!(
        retired_denial,
        WorthQueryProductBranchAdmissionDenial::RetiredBranch
    );
    assert_eq!(
        observer.observe().acquisitions(),
        before_retired.acquisitions(),
        "retired-token denial must precede Query basis acquisition"
    );
    assert!(matches!(
        world.application.on_branch(retired).close().unwrap_err(),
        WorthQueryApplicationProductBranchCloseDenial::Product(
            WorthQueryProductBranchCloseDenial::Selection(
                WorthQueryProductBranchAdmissionDenial::RetiredBranch
            )
        )
    ));
}

#[test]
fn retained_read_defers_component_cleanup_and_preserves_retry_authority() {
    let world = CourtroomWorld::publish("ready");
    let branch = world
        .application
        .branches()
        .fork(world.application.current_world())
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("the Relational fork must publish");
    let retained = world
        .application
        .on_branch(branch)
        .select()
        .expect("the new product must be selectable");

    let WorthQueryApplicationProductBranchCloseDenial::Product(
        WorthQueryProductBranchCloseDenial::OwnerCleanupPending(failure),
    ) = world
        .application
        .on_branch(branch)
        .close()
        .expect_err("the selected read must retain the branch-private World history")
    else {
        panic!("the close must preserve retryable owner cleanup")
    };
    assert_eq!(
        failure.denial(),
        WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryStillRetained
    );

    drop(failure);
    assert_eq!(world.application.branches().pending_cleanup().len(), 1);
    drop(retained);
    let mut pending = world.application.branches().pending_cleanup();
    assert_eq!(pending.len(), 1);
    let cleanup = pending.remove(0);
    let cleanup = cleanup
        .retry()
        .expect("cleanup must resume with the original owner authority");
    assert!(cleanup.is_complete());
    assert_eq!(cleanup.retired_component_count(), 1);
    assert!(world.application.branches().pending_cleanup().is_empty());
}

#[test]
fn reuse_only_descendant_reclaims_its_history_before_parent_component_cleanup() {
    let world = CourtroomWorld::publish("ready");
    let parent = world
        .application
        .branches()
        .fork(world.application.current_world())
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("the parent Relational fork must publish");
    let child = world
        .application
        .branches()
        .fork(parent)
        .components(|components| {
            components
                .reuse_exact_relational_basis()
                .reuse_exact_signal_basis()
        })
        .create()
        .expect("the reuse-only child must publish");
    let committed = world
        .change_input_on_branch(child, "reuse-descendant-change")
        .require_committed()
        .expect("the child must install a branch-local World descendant");
    let WorthQueryApplicationProductBranchCloseDenial::Product(
        WorthQueryProductBranchCloseDenial::OwnerCleanupPending(failure),
    ) = world
        .application
        .on_branch(child)
        .close()
        .expect_err("the caller-held publication receipt must retain its World history")
    else {
        panic!("the close must preserve retryable history cleanup")
    };
    assert_eq!(
        failure.denial(),
        WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryStillRetained
    );
    drop(committed);
    let child_close = failure
        .into_cleanup()
        .retry()
        .expect("internal evidence was retired, so releasing the caller receipt completes cleanup");
    assert_eq!(child_close.retired_component_count(), 0);
    let parent_close = world
        .application
        .on_branch(parent)
        .close()
        .expect("the parent must no longer be retained by the closed child");
    assert_eq!(parent_close.retired_component_count(), 1);
    assert!(world.application.branches().pending_cleanup().is_empty());
}

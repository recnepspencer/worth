use std::sync::Arc;

use worth_query_host::facade::product::{
    WorthQueryApplicationProductBranchCleanupDenial, WorthQueryApplicationProductBranchCloseDenial,
    WorthQueryProductBranchAdmissionDenial, WorthQueryProductBranchCloseDenial,
    WorthQueryProductBranchOwnerCleanupDenial,
};
use worth_query_host::facade::{primary_graph, runtime};

use super::adapters::ReplacementPredicate;
use super::world::{self, CourtroomWorld};

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
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(foreign.application.current_world())
                .select(&phase)
                .err()
                .expect("a branch token from another World owner must be refused")
        })
        .expect("installed host call opens");
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
        .with_host_advancement(|phase| {
            world
                .application
                .branches()
                .fork(source)
                .components(|components| {
                    components
                        .reuse_exact_relational_basis()
                        .reuse_exact_signal_basis()
                })
                .create(&phase)
                .expect("the owned reuse branch must publish")
        })
        .expect("installed host call opens");
    let close = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(retired)
                .close(&phase)
                .expect("an unretained public product can close")
        })
        .expect("installed host call opens");
    assert!(close.is_complete());

    let before_retired = observer.observe();
    let retired_denial = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(retired)
                .select(&phase)
                .err()
                .expect("a retired occurrence token must be refused")
        })
        .expect("installed host call opens");
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
        world
            .application
            .with_host_advancement(|phase| world
                .application
                .on_branch(retired)
                .close(&phase)
                .unwrap_err())
            .expect("installed host call opens"),
        WorthQueryApplicationProductBranchCloseDenial::Product(
            WorthQueryProductBranchCloseDenial::Selection(
                WorthQueryProductBranchAdmissionDenial::RetiredBranch
            )
        )
    ));
}

#[path = "selected_product_lifecycle/retained_read_defers_component_cleanup_and_preserves_retry_autho.rs"]
mod retained_read_defers_component_cleanup_and_preserves_retry_autho;

#[test]
fn reuse_only_descendant_reclaims_its_history_before_parent_component_cleanup() {
    let world = CourtroomWorld::publish("ready");
    let parent = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .branches()
                .fork(world.application.current_world())
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .expect("the parent Relational fork must publish")
        })
        .expect("installed host call opens");
    let child = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .branches()
                .fork(parent)
                .components(|components| {
                    components
                        .reuse_exact_relational_basis()
                        .reuse_exact_signal_basis()
                })
                .create(&phase)
                .expect("the reuse-only child must publish")
        })
        .expect("installed host call opens");
    let committed = world
        .change_input_on_branch(child, "reuse-descendant-change")
        .require_committed()
        .expect("the child must install a branch-local World descendant");
    let WorthQueryApplicationProductBranchCloseDenial::OwnerCleanupPending(failure) = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(child)
                .close(&phase)
                .expect_err("the caller-held publication receipt must retain its World history")
        })
        .expect("installed host call opens")
    else {
        panic!("the close must preserve retryable history cleanup")
    };
    assert!(matches!(
        failure.denial(),
        WorthQueryApplicationProductBranchCleanupDenial::Product(
            WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryStillRetained
        )
    ));
    drop(committed);
    let child_close = failure
        .into_cleanup()
        .retry()
        .expect("internal evidence was retired, so releasing the caller receipt completes cleanup");
    assert_eq!(child_close.retired_component_count(), 0);
    let parent_close = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(parent)
                .close(&phase)
                .expect("the parent must no longer be retained by the closed child")
        })
        .expect("installed host call opens");
    assert_eq!(parent_close.retired_component_count(), 1);
    assert!(world.application.branches().pending_cleanup().is_empty());
}

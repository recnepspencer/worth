use crate::support::public_bridge_runtime::{
    public_graph_support_profile, public_product_world_resources_with_branch_limit,
    PublicBridgeRuntimeHarness,
};
use worth_query::facade::product::{
    WorthQueryProductBranchCloseDenial, WorthQueryProductBranchCreateError,
    WorthQueryProductBranchOwnerCleanupDenial,
};

#[test]
fn outer_workspace_close_releases_exact_branch_and_reuses_world_capacity() {
    let harness = PublicBridgeRuntimeHarness::new();
    let runtime = harness
        .bridge_backed_runtime_builder()
        .support_profile(public_graph_support_profile())
        .build_with_product_world_resources(public_product_world_resources_with_branch_limit(2));
    let workspace = runtime
        .workspace("public.product-branch-lifecycle")
        .expect("the public runtime opens its workspace");
    let root = workspace.current_world();
    let branch = workspace
        .branches()
        .fork(root)
        .components(|components| components.reuse_exact_relational_basis().fork_signal())
        .create()
        .expect("the only non-root branch capacity publishes");

    let saturated = workspace
        .branches()
        .fork(root)
        .components(|components| components.reuse_exact_relational_basis().fork_signal())
        .create();
    assert!(matches!(
        saturated,
        Err(WorthQueryProductBranchCreateError::Creation(_))
    ));

    let retained = workspace
        .observe_operating_world(branch)
        .expect("the created branch admits a real retained read");
    let WorthQueryProductBranchCloseDenial::OwnerCleanupPending(failure) = workspace
        .branches()
        .close(branch)
        .expect_err("the retained read must defer retired World history cleanup")
    else {
        panic!("close must preserve bounded cleanup custody after World retirement")
    };
    assert_eq!(
        failure.denial(),
        WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryStillRetained
    );
    drop(failure);
    assert_eq!(workspace.branches().pending_cleanup().len(), 1);
    drop(retained);

    let mut pending = workspace.branches().pending_cleanup();
    let closed = pending
        .pop()
        .expect("outer retirement cleanup remains discoverable")
        .retry()
        .expect("releasing the read permits exact history and owner cleanup");
    assert!(closed.is_complete());
    assert_eq!(closed.retired_component_count(), 1);
    assert!(workspace.branches().pending_cleanup().is_empty());
    assert!(workspace.observe_operating_world(branch).is_err());

    let replacement = workspace
        .branches()
        .fork(root)
        .components(|components| components.reuse_exact_relational_basis().fork_signal())
        .create()
        .expect("explicit close returns the bounded branch capacity");
    assert_ne!(replacement, branch, "closed occurrences are never reused");
    let replacement_close = workspace
        .branches()
        .close(replacement)
        .expect("the replacement closes through the same public lifecycle");
    assert_eq!(replacement_close.product_branch(), replacement);
    assert_eq!(replacement_close.retired_component_count(), 1);
    assert!(replacement_close.is_complete());
}

use worth_query_host::facade::product::WorthQueryProductBranch;

use super::super::world::CourtroomWorld;

pub(crate) fn creates_and_selects_all_component_postures() {
    let world = CourtroomWorld::publish("blocked");
    let source = world.application.current_world();

    let reuse_both = create(&world, source, |components| {
        components
            .reuse_exact_relational_basis()
            .reuse_exact_signal_basis()
    });
    let fork_relational = create(&world, source, |components| {
        components.fork_relational().reuse_exact_signal_basis()
    });
    let fork_signal = create(&world, source, |components| {
        components.reuse_exact_relational_basis().fork_signal()
    });
    let fork_both = create(&world, source, |components| {
        components.fork_relational().fork_signal()
    });

    for branch in [reuse_both, fork_relational, fork_signal, fork_both] {
        let selected = world.application.on_branch(branch).select().unwrap();
        assert_eq!(selected.product().product_branch(), branch.id());
    }

    let committed = world
        .change_input_on_branch(reuse_both, "public-facade-change")
        .require_committed()
        .expect("the public transaction must commit");
    assert_eq!(committed.product_branch(), reuse_both.id());
    drop(committed);

    for (branch, expected_retired_components) in [
        (reuse_both, 0),
        (fork_relational, 1),
        (fork_signal, 1),
        (fork_both, 2),
    ] {
        let closed = world
            .application
            .on_branch(branch)
            .close()
            .expect("each public creation posture must cleanly close");
        assert_eq!(
            closed.retired_component_count(),
            expected_retired_components
        );
        assert!(closed.is_complete());
    }
}

fn create(
    world: &CourtroomWorld,
    source: WorthQueryProductBranch,
    choose: impl FnOnce(
        worth_query_host::facade::declaration::branch::WorthQueryProductBranchComponents,
    ) -> worth_query_host::facade::declaration::branch::WorthQueryProductBranchComponents,
) -> WorthQueryProductBranch {
    world
        .application
        .branches()
        .fork(source)
        .components(choose)
        .create()
        .unwrap()
}

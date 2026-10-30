// Retained ancestry is explicit; exact ancestry and pages keep refusing gaps.
pub(crate) use {
    crate::court::ordinary_context::exact_history::signal_fork_child,
    worth_runtime_world::facade::*,
};

#[test]
fn retained_selection_proves_only_endpoint_ancestry_after_real_retirement() {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let court = crate::court::ordinary_context::CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let sibling = signal_fork_child(&court, &root, "retained-sibling");
    let first = court.publish_cargo(&root, "5");
    let middle = court.observe(&root);
    let second = court.publish_cargo(&middle, "6");
    let head = court.observe(&middle);
    let removed = middle.selected_commit().clone();
    drop((first, second, middle));
    let retired = court
        .world
        .lifecycle_port()
        .retire_unprotected_history(head.selected_commit())
        .unwrap();
    assert!(
        retired.contains(&removed),
        "the gap is real, not a hypothetical lineage"
    );
    let port = court.world.observation_port();
    assert!(matches!(
        port.select_product_branch_ancestor(&execution, &head, root.selected_commit(), None),
        Err(RuntimeWorldServiceDenial::Denied(
            ProductBranchAncestorSelectionDenial::History(
                CompositeAncestorSelectionDenial::RetiredHistoryGap
            )
        ))
    ));
    let selected = port
        .select_retained_product_branch_ancestor(
            &execution,
            &head,
            root.selected_commit(),
            std::num::NonZeroUsize::new(2),
        )
        .expect("retained endpoints and the owner's splice prove ancestry");
    assert_eq!(selected.source().identity(), root.selected_commit());
    assert_eq!(selected.source().basis(), root.basis());
    assert_eq!(selected.target(), &head);
    assert_eq!(selected.parent_links_followed(), 1);
    assert!(matches!(
        port.select_retained_product_branch_ancestor(
            &execution,
            &head,
            root.selected_commit(),
            Some(std::num::NonZeroUsize::MIN)
        ),
        Err(RuntimeWorldServiceDenial::Denied(
            ProductBranchAncestorSelectionDenial::History(
                CompositeAncestorSelectionDenial::SelectionLimitReached
            )
        ))
    ));
    assert!(matches!(
        port.select_retained_product_branch_ancestor(
            &execution,
            &head,
            sibling.selected_commit(),
            None
        ),
        Err(RuntimeWorldServiceDenial::Denied(
            ProductBranchAncestorSelectionDenial::History(
                CompositeAncestorSelectionDenial::NotAncestor
            )
        ))
    ));
    let cursor = port.open_exact_product_history(execution, &head).unwrap();
    assert!(matches!(
        port.prepare_exact_product_history_page(execution, &cursor),
        Err(RuntimeWorldServiceDenial::Denied(
            ProductExactHistoryDenial::History(CompositeAncestorSelectionDenial::RetiredHistoryGap)
        ))
    ));
    drop((selected, cursor, root, sibling, head));
    court.finish();
}

use super::*;

fn signal_fork_child(
    court: &CompositeSupplyChainCourt,
    source: &ProductBranchObservation,
    name: &str,
) -> ProductBranchObservation {
    let RuntimeWorldBranchCreationOutcome::Performed(child) = court
        .world
        .branch_port()
        .create_product_branch(
            source.clone(),
            ProductBranchCreationIntent::from_source(
                name,
                ProductBranchCreationPlans::new(
                    RelationalBranchCreationPlan::ReuseExact,
                    SignalBranchCreationPlan::ForkExact {
                        target: worth_signal::facade::branch::validate_signal_branch_name(name)
                            .unwrap(),
                    },
                ),
            )
            .unwrap(),
            &RuntimeWorldCancellationSource::new().token(),
        )
        .unwrap()
    else {
        panic!("genuine branch creation must perform");
    };
    child
}

#[test]
fn exact_product_pages_keep_the_pinned_fork_ancestry_and_stage_owner_positions() {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let court = CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let sibling = signal_fork_child(&court, &root, "history-sibling");
    let first = court.publish_cargo(&root, "5");
    let first_head = court.observe(&root);
    let child = signal_fork_child(&court, &first_head, "history-child");
    let port = court.world.observation_port();
    let mut cursor = port.open_exact_product_history(execution, &child).unwrap();
    let later = court.publish_cargo(&first_head, "6");
    assert_ne!(
        later.new_product_head().selected_commit(),
        first_head.selected_commit()
    );
    assert_eq!(
        cursor.target(),
        &child,
        "new publications cannot move the pinned source"
    );
    let foreign = CompositeSupplyChainCourt::compile();
    let foreign_root = foreign.bootstrap();
    assert!(matches!(
        foreign
            .world
            .observation_port()
            .prepare_exact_product_history_page(execution, &cursor),
        Err(RuntimeWorldServiceDenial::Denied(
            ProductExactHistoryDenial::Branch(RuntimeWorldBranchAdmissionDenial::ForeignOwner)
        ))
    ));
    let mut stale = Some(
        port.prepare_exact_product_history_page(execution, &cursor)
            .unwrap(),
    );
    let mut observed = Vec::new();
    let mut native_publications = Vec::new();
    loop {
        let prepared = port
            .prepare_exact_product_history_page(execution, &cursor)
            .unwrap();
        assert_eq!(prepared.source_visits(), 1);
        let commit = prepared.commit().unwrap();
        observed.push(commit.identity().clone());
        if let Some(native) = commit.relational_publication_identity() {
            native_publications.push(native.clone());
        }
        let (page, successor) = cursor.accept_page(prepared).unwrap();
        assert_eq!(page.commit().unwrap().identity(), observed.last().unwrap());
        assert_eq!(page.source_visits(), 1);
        cursor = successor;
        if observed.len() == 1 {
            assert!(matches!(
                cursor.accept_page(stale.take().unwrap()),
                Err(ProductExactHistoryDenial::InvalidContinuation)
            ));
            let cancellation = worth_execution::CancellationSource::new();
            let controlled_host = host.clone().with_cancellation(cancellation.token());
            let prepared = port
                .prepare_exact_product_history_page(
                    worth_execution::ExecutionRequest::serial(&controlled_host),
                    &cursor,
                )
                .unwrap();
            cancellation.cancel();
            assert!(matches!(
                cursor.accept_page(prepared),
                Err(ProductExactHistoryDenial::Request(_))
            ));
            let declined = port
                .prepare_exact_product_history_page(execution, &cursor)
                .unwrap();
            drop(declined);
            let prepared = port
                .prepare_exact_product_history_page(execution, &cursor)
                .unwrap();
            let other = port.open_exact_product_history(execution, &child).unwrap();
            assert!(matches!(
                other.accept_page(prepared),
                Err(ProductExactHistoryDenial::InvalidContinuation)
            ));
            let prepared = port
                .prepare_exact_product_history_page(execution, &cursor)
                .unwrap();
            let unpublished = cursor.accept_page(prepared).unwrap();
            drop(unpublished);
        }
        if page.is_complete() {
            break;
        }
    }
    assert_eq!(
        observed,
        [
            child.selected_commit().clone(),
            first_head.selected_commit().clone(),
            root.selected_commit().clone()
        ]
    );
    assert!(!observed.contains(sibling.selected_commit()));
    assert!(!observed.contains(later.new_product_head().selected_commit()));
    assert_eq!(
        native_publications.len(),
        1,
        "reuse and bootstrap carry existing components, not another native publication"
    );
    let complete = port
        .prepare_exact_product_history_page(execution, &cursor)
        .unwrap();
    assert_eq!(
        (
            complete.source_visits(),
            complete.commit().is_none(),
            complete.is_complete()
        ),
        (0, true, true)
    );
    drop(complete);
    drop((
        cursor,
        root,
        first_head,
        first,
        child,
        sibling,
        later,
        foreign_root,
    ));
    foreign.finish();
    court.finish();
}

#[test]
fn exact_product_history_protects_the_next_parent_and_refuses_retired_gaps() {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let court = CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let first = court.publish_cargo(&root, "5");
    let first_head = court.observe(&root);
    let second = court.publish_cargo(&first_head, "6");
    let second_head = court.observe(&first_head);
    let third = court.publish_cargo(&second_head, "7");
    let removed = first_head.selected_commit().clone();
    let next = second_head.selected_commit().clone();
    let head = court.observe(&second_head);
    let port = court.world.observation_port();
    let cursor = port.open_exact_product_history(execution, &head).unwrap();
    let prepared = port
        .prepare_exact_product_history_page(execution, &cursor)
        .unwrap();
    drop((first, second, third, first_head, second_head));
    let retired = court
        .world
        .lifecycle_port()
        .retire_unprotected_history(head.selected_commit())
        .unwrap();
    assert!(retired.contains(&removed));
    assert!(
        !retired.contains(&next),
        "the prepared successor holds actual catalog protection"
    );
    assert!(matches!(
        port.select_product_branch_ancestor(&execution, &head, root.selected_commit(), None),
        Err(RuntimeWorldServiceDenial::Denied(
            ProductBranchAncestorSelectionDenial::History(
                CompositeAncestorSelectionDenial::RetiredHistoryGap
            )
        ))
    ));
    let (page, inherited) = cursor.accept_page(prepared).unwrap();
    assert_eq!(page.commit().unwrap().identity(), head.selected_commit());
    assert!(matches!(
        port.prepare_exact_product_history_page(execution, &inherited),
        Err(RuntimeWorldServiceDenial::Denied(
            ProductExactHistoryDenial::History(CompositeAncestorSelectionDenial::RetiredHistoryGap)
        ))
    ));
    drop((page, inherited));
    let retired = court
        .world
        .lifecycle_port()
        .retire_unprotected_history(head.selected_commit())
        .unwrap();
    assert!(
        retired.contains(&next),
        "dropping the successor releases its actual parent pin"
    );
    assert!(matches!(
        port.prepare_exact_product_history_page(execution, &cursor),
        Err(RuntimeWorldServiceDenial::Denied(
            ProductExactHistoryDenial::History(CompositeAncestorSelectionDenial::RetiredHistoryGap)
        ))
    ));
    drop((cursor, root, head));
    court.finish();
}

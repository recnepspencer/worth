//! Genuine old component roots create independent writable destinations.
use super::*;
mod refusals;
use std::collections::BTreeMap;
use worth_relational::facade::mvcc::RelationalPublicationOutcome;

fn expected(grain: &str) -> BTreeMap<String, String> {
    [
        ("harbor", "open"),
        ("voyage", "12"),
        ("manifest", "harbor"),
        ("grain", grain),
        ("steel", "6"),
    ]
    .into_iter()
    .map(|(name, value)| (name.to_owned(), value.to_owned()))
    .collect()
}

#[test]
fn moved_native_source_forks_exact_old_truth_without_replacing_current_or_sibling() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let court = CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let token = RuntimeWorldCancellationSource::new().token();
    let branch = court.world.branch_port();
    let RuntimeWorldBranchCreationOutcome::Performed(untouched) = branch
        .create_product_branch(
            execution,
            root.clone(),
            ProductBranchCreationIntent::from_source("untouched", plans(true, true, "untouched"))
                .unwrap(),
            &token,
        )
        .unwrap()
    else {
        panic!("independent sibling")
    };
    let old = court
        .records
        .read(execution, root.basis().relational_basis());
    assert_eq!(old.records, expected("4"));
    let candidate =
        court
            .records
            .candidate(execution, root.basis().relational_basis(), "grain", "9");
    let RelationalPublicationOutcome::Performed(performed) = court
        .records
        .runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("real Native move")
    };
    let committed = court
        .records
        .runtime
        .settle_performed_publication(performed)
        .unwrap();
    let current = court
        .records
        .runtime
        .observe_branch(&court.records.runtime.main_branch_identity())
        .unwrap()
        .1;
    assert_ne!(
        current.reference(),
        root.basis().relational_basis().reference()
    );
    assert_eq!(
        court.records.read(execution, &current).records,
        expected("9")
    );
    assert!(matches!(
        branch.create_product_branch(
            execution,
            root.clone(),
            ProductBranchCreationIntent::from_source(
                "current-only",
                plans(true, true, "current-only")
            )
            .unwrap(),
            &token
        ),
        Err(RuntimeWorldServiceDenial::Denied(
            RuntimeWorldBranchAdmissionDenial::ForkSourceChanged
        ))
    ));
    assert!(court
        .records
        .runtime
        .branch_identity(&BranchId("current-only".into()))
        .is_err());
    let retained = branch.admit_retained_branch_source(&root).unwrap();
    let RuntimeWorldBranchCreationOutcome::Performed(child) = branch
        .create_product_branch_from_retained_source(
            execution,
            retained,
            ProductBranchCreationIntent::from_source("old-child", plans(true, true, "old-child"))
                .unwrap(),
            &token,
        )
        .unwrap()
    else {
        panic!("retained source has two real owner forks")
    };
    assert_ne!(
        child.basis().relational_basis().identity(),
        root.basis().relational_basis().identity()
    );
    assert_eq!(
        child
            .basis()
            .relational_basis()
            .descriptor()
            .root_identity(),
        root.basis().relational_basis().descriptor().root_identity()
    );
    assert_eq!(
        child.basis().relational_basis().observation().commit_id(),
        root.basis().relational_basis().observation().commit_id()
    );
    assert_ne!(
        child.basis().signal_basis().branch_id(),
        root.basis().signal_basis().branch_id()
    );
    assert_eq!(
        court
            .records
            .read(execution, child.basis().relational_basis()),
        old
    );
    let changed = court.publish_cargo(&child, "7");
    assert_eq!(
        court
            .records
            .read(
                execution,
                changed.new_product_head().basis().relational_basis()
            )
            .records,
        expected("7")
    );
    assert_eq!(
        court
            .records
            .read(execution, child.basis().relational_basis()),
        old
    );
    assert_eq!(
        court
            .records
            .read(execution, untouched.basis().relational_basis()),
        old
    );
    assert_eq!(
        court.records.read(execution, &current).records,
        expected("9")
    );
    assert_eq!(court.observe(&root), root);
    court
        .records
        .runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    let changed_child = court.observe(&child);
    let retired_child = branch.retire_product_branch(&changed_child).unwrap();
    let retired_untouched = branch.retire_product_branch(&untouched).unwrap();
    assert_eq!(retired_child.owner_retirement_work().len(), 2);
    assert_eq!(retired_untouched.owner_retirement_work().len(), 2);
    drop((
        retired_child,
        retired_untouched,
        changed_child,
        changed,
        child,
        untouched,
        root,
        current,
        committed,
    ));
    court.finish();
}

#[test]
fn old_world_source_keeps_original_cleanup_when_signal_moves_after_admission() {
    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let court = CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let moved = court.publish_cargo(&root, "5");
    let current = court.observe(&root);
    assert_eq!(
        current.selected_commit(),
        moved.new_product_head().selected_commit()
    );
    let branch = court.world.branch_port();
    let retained = branch.admit_retained_branch_source(&root).unwrap();
    let token = RuntimeWorldCancellationSource::new().token();
    let RuntimeWorldBranchCreationOutcome::Performed(old_child) = branch
        .create_product_branch_from_retained_source(
            &execution,
            retained,
            ProductBranchCreationIntent::from_source(
                "old-world-child",
                plans(true, true, "old-world-child"),
            )
            .unwrap(),
            &token,
        )
        .unwrap()
    else {
        panic!("an actually old World source reaches retained registry insertion")
    };
    assert_eq!(
        court
            .records
            .read(&execution, old_child.basis().relational_basis())
            .records,
        expected("4")
    );
    assert_eq!(
        court
            .records
            .read(&execution, current.basis().relational_basis())
            .records,
        expected("5")
    );
    assert_ne!(
        old_child.basis().relational_basis().identity(),
        root.basis().relational_basis().identity()
    );
    assert_ne!(
        old_child.basis().signal_basis().branch_id(),
        root.basis().signal_basis().branch_id()
    );
    assert_eq!(court.observe(&root), current);
    let retained = branch.admit_retained_branch_source(&root).unwrap();
    let prepared = court
        .world
        .publication_port()
        .prepare_with_signal(
            current.clone(),
            CompositePublicationIntent::with_signal(None),
            &token,
            None,
        )
        .unwrap();
    let RuntimeWorldPublicationOutcome::Performed(signal_moved) =
        court.world.publication_port().execute_with_signal(
            execution,
            prepared,
            &mut court.context(&execution, &current),
            &token,
            |_| Ok(()),
        )
    else {
        panic!("real Signal source displacement")
    };
    let signal_moved = signal_moved.consume();
    assert!(matches!(
        branch.admit_retained_branch_source(&root),
        Err(RuntimeWorldServiceDenial::Denied(
            RuntimeWorldBranchAdmissionDenial::ForkSourceChanged
        ))
    ));
    let RuntimeWorldBranchCreationOutcome::ProductUnpublished(effects) = branch
        .create_product_branch_from_retained_source(
            &execution,
            retained,
            ProductBranchCreationIntent::from_source(
                "signal-loser",
                plans(true, true, "signal-loser"),
            )
            .unwrap(),
            &token,
        )
        .unwrap()
    else {
        panic!("original Signal receiver refuses after the real Native fork")
    };
    assert_eq!(effects.owner_effect_count(), 1);
    let partial = court
        .records
        .runtime
        .branch_identity(&BranchId("signal-loser".into()))
        .unwrap();
    let basis = court.records.runtime.admit_branch_basis(&partial).unwrap();
    assert_eq!(
        court.records.read(&execution, &basis).records,
        expected("4")
    );
    assert_eq!(
        court
            .records
            .read(
                &execution,
                signal_moved.new_product_head().basis().relational_basis()
            )
            .records,
        expected("5")
    );
    drop(basis);
    let handle = effects.recovery_handle();
    drop(effects);
    let (_original, cleanup) = court
        .world
        .recovery_port()
        .release_effects(
            &execution,
            &handle,
            0,
            court.observe_recovery_original(&execution, &handle),
        )
        .unwrap()
        .into_parts();
    assert_eq!(cleanup.owner_retirement_work().len(), 1);
    let OwnerRetirementWork::RelationalBranchRetirement { identity, .. } =
        &cleanup.owner_retirement_work()[0]
    else {
        panic!("only performed Native custody")
    };
    assert_eq!(identity, &partial);
    assert!(matches!(
        court
            .records
            .runtime
            .owner_component_services()
            .lifecycle_port()
            .delete_branch(&execution, identity)
            .unwrap(),
        worth_relational::facade::branch::RelationalBranchDeletionOutcome::Deleted(_)
    ));
    assert!(court
        .records
        .runtime
        .branch_identity(&BranchId("signal-loser".into()))
        .is_err());
    let selected = court.observe(&root);
    assert_eq!(
        selected.selected_commit(),
        signal_moved.new_product_head().selected_commit()
    );
    assert_eq!(selected.basis(), signal_moved.new_product_head().basis());
    drop(selected);
    let retired_child = branch.retire_product_branch(&old_child).unwrap();
    assert_eq!(retired_child.owner_retirement_work().len(), 2);
    drop((
        retired_child,
        cleanup,
        signal_moved,
        moved,
        current,
        old_child,
        root,
    ));
    court.finish();
}

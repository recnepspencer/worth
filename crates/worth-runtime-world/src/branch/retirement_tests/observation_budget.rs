use super::fork_creation::setup_with_relational_source;
use super::*;

#[test]
fn fresh_observations_are_bounded_and_clones_share_one_charge() {
    let (_fixture, owner, root) = setup(2);
    let mut observations = (1..8)
        .map(|_| {
            owner
                .observe_product_branch(root.branch_identity())
                .unwrap()
        })
        .collect::<Vec<_>>();
    let cloned = observations.last().unwrap().clone();
    let before = owner.state.retention.cost_snapshot();
    assert_eq!(
        owner
            .observe_product_branch(root.branch_identity())
            .unwrap_err(),
        RuntimeWorldBranchAdmissionDenial::CapacityExhausted
    );
    assert_eq!(owner.state.retention.cost_snapshot(), before);
    drop(observations.pop());
    assert_eq!(
        owner
            .observe_product_branch(root.branch_identity())
            .unwrap_err(),
        RuntimeWorldBranchAdmissionDenial::CapacityExhausted
    );
    drop(cloned);
    let fresh = owner
        .observe_product_branch(root.branch_identity())
        .unwrap();
    assert_eq!(fresh, root);
    assert_eq!(
        owner
            .observe_product_branch(root.branch_identity())
            .unwrap_err(),
        RuntimeWorldBranchAdmissionDenial::CapacityExhausted
    );
}

#[test]
fn creation_reserves_observation_capacity_before_any_component_effect() {
    for fork in [false, true] {
        let (_fixture, owner, root) = setup_with_relational_source(3);
        let mut observations = (1..8)
            .map(|_| {
                owner
                    .observe_product_branch(root.branch_identity())
                    .unwrap()
            })
            .collect::<Vec<_>>();
        let intent = if fork {
            relational_fork_intent("child", "component-child")
        } else {
            reuse_intent("child")
        };
        let cancellation = RuntimeWorldCancellationSource::new();
        let before = owner.state.retention.cost_snapshot();
        let result = owner.create_product_branch(RuntimeWorldBranchCreationRequest::new(
            root.clone(),
            intent.clone(),
            &cancellation.token(),
        ));
        assert!(matches!(
            result,
            Err(RuntimeWorldBranchAdmissionDenial::CapacityExhausted)
        ));
        assert_eq!(owner.state.retention.cost_snapshot(), before);
        assert_eq!(owner.state.branches.branch_count(), 1);
        assert_eq!(owner.state.branches.reserved_branch_count(), 0);
        assert_eq!(owner.state.operation.active(), 0);
        assert_eq!(owner.recovery_record_count(), 0);
        drop(observations.pop());
        // The same component destination must still be unused after denial.
        let result = owner
            .create_product_branch(RuntimeWorldBranchCreationRequest::new(
                root.clone(),
                intent,
                &cancellation.token(),
            ))
            .unwrap();
        let RuntimeWorldBranchCreationOutcome::Performed(child) = result else {
            panic!("the denied fork must not have consumed the destination")
        };
        assert_eq!(
            owner
                .observe_product_branch(child.branch_identity())
                .unwrap_err(),
            RuntimeWorldBranchAdmissionDenial::CapacityExhausted
        );
        drop(child);
        owner
            .observe_product_branch(root.branch_identity())
            .unwrap();
    }
}

#[test]
fn close_reports_observations_that_remain_live_until_their_last_clone_drops() {
    let (_fixture, owner, root) = setup(2);
    let cloned = root.clone();
    let fresh = owner
        .observe_product_branch(root.branch_identity())
        .unwrap();
    let child = create_reused_branch(&owner, &root, reuse_intent("child"));
    let report = owner.close().unwrap();
    assert_eq!(report.outstanding_observations(), 3);
    assert_eq!(report.released_observation_pins(), 0);
    assert_eq!(report.released_product_head_pins(), 1);
    assert_eq!(owner.state.retention.active_observation_count(), 3);
    drop(root);
    assert_eq!(owner.state.retention.active_observation_count(), 3);
    drop((cloned, fresh, child));
    assert_eq!(owner.state.retention.active_observation_count(), 0);
}

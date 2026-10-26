use super::*;

#[test]
fn branch_capacity_evidence_counts_root_heads_and_reserved_forks_without_effect() {
    let (_fixture, owner, root) = setup(4);
    let own_heads = [
        create_reused_branch(&owner, &root, reuse_intent("own-1")),
        create_reused_branch(&owner, &root, reuse_intent("own-2")),
    ];
    let outside = owner
        .state
        .branches
        .reserve_branch(
            owner.owner_identity(),
            crate::branch::ProductBranchName::try_new("outside-in-flight").unwrap(),
        )
        .unwrap();
    let cancellation = RuntimeWorldCancellationSource::new();
    let before = owner.state.retention.cost_snapshot();
    let refuse = |name: &str| {
        RuntimeWorldBranchService::create_product_branch(
            &owner,
            RuntimeWorldBranchCreationRequest::new(
                root.clone(),
                reuse_intent(name),
                &cancellation.token(),
            ),
        )
        .unwrap_err()
    };
    let first = refuse("refused");
    assert_eq!(
        first,
        RuntimeWorldBranchAdmissionDenial::BranchCapacityExhausted {
            limit: 4,
            occupancy: 4
        }
    );
    assert_eq!(first, refuse("refused"));
    assert_eq!(owner.state.branches.branch_count(), 1 + own_heads.len());
    assert_eq!(owner.state.branches.reserved_branch_count(), 1);
    assert_eq!(owner.state.retention.cost_snapshot(), before);
    drop(outside);
    let spare = create_reused_branch(&owner, &root, reuse_intent("refused"));
    // Same total, now all slots are accounted for by this holder and the root.
    assert_eq!(refuse("another"), first);
    assert_eq!(owner.state.branches.branch_count(), 1 + own_heads.len() + 1);
    assert_eq!(owner.state.branches.reserved_branch_count(), 0);
    let retirement = owner.retire_product_branch(&spare).unwrap();
    assert!(retirement.owner_retirement_work().is_empty());
    // A retained observation of a retired branch holds no World membership.
    let replacement = create_reused_branch(&owner, &root, reuse_intent("replacement"));
    assert_ne!(
        replacement.lifecycle_incarnation(),
        spare.lifecycle_incarnation()
    );
}

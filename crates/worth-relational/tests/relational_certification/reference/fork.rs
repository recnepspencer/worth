use super::reference_attempt_evidence::{
    assert_denial_left_no_reference_residue, capture_reference_evidence,
};
use super::world::supply_chain::{
    assert_oracle_matches, canonical_empty_supply_chain_runtime, certified_supply_chain_world,
    SupplyChainScale,
};
use worth_relational::facade::branch::{RelationalBranchIdentityDenial, RelationalForkDenial};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::transactions::WorkerIntentBatch;
#[test]
fn supply_chain_fork_shares_one_source_artifact_and_starts_a_new_reference_line() {
    let (world, expected) = certified_supply_chain_world(SupplyChainScale::court());
    assert_oracle_matches(&world, &expected);
    let source_catalog_count = world.runtime.history().immutable_commit_count();
    let before = capture_reference_evidence(
        &world.runtime,
        &BranchId("main".to_owned()),
        &BranchId("storm".to_owned()),
        world.commit.commit_id,
    );

    let (descriptor, source_basis) = world
        .runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("installed main branch has an exact fork source");
    assert_eq!(descriptor.source_branch(), &BranchId("main".to_owned()));
    let outcome = world
        .runtime
        .fork_branch(BranchId("storm".to_owned()), source_basis)
        .expect("fork source is consumed by the owner fork transition");
    assert_oracle_matches(&world, &expected);

    assert_eq!(
        outcome.target_identity().branch_id(),
        &BranchId("storm".to_owned())
    );
    assert_eq!(outcome.target_truth_version().as_u64(), 0);
    assert_eq!(
        outcome.source_observation().target(),
        outcome.target_observation().target()
    );
    let source_target = outcome
        .source_observation()
        .target()
        .as_basis()
        .expect("installed source has a committed target");
    let target_target = outcome
        .target_observation()
        .target()
        .as_basis()
        .expect("fork target retains the committed target");
    assert_eq!(source_target.roots(), target_target.roots());
    assert!(source_target
        .roots()
        .truth_root()
        .iter()
        .any(|byte| *byte != 0));
    assert!(source_target
        .roots()
        .schema_root()
        .iter()
        .any(|byte| *byte != 0));
    assert_eq!(descriptor.observation(), outcome.source_observation());
    assert_eq!(outcome.fork_provenance(), outcome.source_observation());
    assert_ne!(
        outcome.source_observation().branch_id(),
        outcome.target_observation().branch_id()
    );
    assert_eq!(outcome.target_observation().generation().get(), 0);
    assert_eq!(outcome.shared_commit_id(), Some(world.commit.commit_id));
    let catalog_identity = world
        .runtime
        .history()
        .immutable_commit_identity(world.commit.commit_id)
        .expect("source commit is present in the immutable catalog");
    assert_eq!(catalog_identity.commit_id(), world.commit.commit_id);
    assert_eq!(catalog_identity.version_id(), world.commit.version_id);
    assert_eq!(
        catalog_identity.authoring_branch(),
        &BranchId("main".to_owned())
    );
    let catalog_receipt = world
        .runtime
        .history()
        .immutable_commit_receipt(world.commit.commit_id)
        .expect("source artifact retains its immutable ordered parentage");
    assert_eq!(catalog_receipt.parents, world.commit.parents);
    let (_, maintenance_basis) = world
        .runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("the source remains an exact live basis for a sibling fork");
    let maintenance = world
        .runtime
        .fork_branch(BranchId("maintenance".to_owned()), maintenance_basis)
        .expect("a second sibling fork reuses the same immutable source artifact");
    assert_eq!(maintenance.shared_commit_id(), Some(world.commit.commit_id));
    assert_eq!(
        maintenance.target_observation().target(),
        outcome.target_observation().target()
    );
    assert_ne!(
        maintenance.target_identity().branch_id(),
        outcome.target_identity().branch_id()
    );
    assert_eq!(
        world.runtime.history().immutable_commit_count(),
        source_catalog_count
    );
    // The immutable catalog identity is the Phase-4 observation boundary.
    // Replay is a later cert/maintenance lane and must not be imported by
    // this Supply Chain currentness court.
    assert_eq!(catalog_identity.commit_id(), world.commit.commit_id);
    assert_eq!(
        world.runtime.history().immutable_commit_count(),
        source_catalog_count
    );
    let after = capture_reference_evidence(
        &world.runtime,
        &BranchId("main".to_owned()),
        &BranchId("storm".to_owned()),
        world.commit.commit_id,
    );
    assert_eq!(after.catalog_count, before.catalog_count);
    assert_eq!(after.artifact_identity, before.artifact_identity);
    assert_eq!(after.source_identity, before.source_identity);
    assert!(matches!(
        after.target_identity,
        Ok(ref identity) if identity.branch_id() == &BranchId("storm".to_owned())
    ));
}

#[test]
fn source_observation_is_foreign_after_runtime_clone() {
    let (world, expected) = certified_supply_chain_world(SupplyChainScale::court());
    assert_oracle_matches(&world, &expected);
    let (_, source_basis) = world
        .runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("installed main branch has an exact fork source");
    let clone = world.runtime.fork().expect("settled runtime forks");
    let source_identity = world.runtime.main_branch_identity();
    let clone_identity = clone.main_branch_identity();
    assert_eq!(source_identity.branch_id(), clone_identity.branch_id());
    assert_ne!(
        source_identity.runtime_instance_id(),
        clone_identity.runtime_instance_id()
    );

    let clone_before = capture_reference_evidence(
        &clone,
        &BranchId("main".to_owned()),
        &BranchId("clone-storm".to_owned()),
        world.commit.commit_id,
    );

    assert!(matches!(
        clone.fork_branch(BranchId("clone-storm".to_owned()), source_basis),
        Err(RelationalForkDenial::ForeignRuntime)
    ));
    let clone_after = capture_reference_evidence(
        &clone,
        &BranchId("main".to_owned()),
        &BranchId("clone-storm".to_owned()),
        world.commit.commit_id,
    );
    assert_denial_left_no_reference_residue(&clone_before, &clone_after);

    let (_, clone_basis) = clone
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("the rebound clone keeps its own live fork source");
    let outcome = clone
        .fork_branch(BranchId("clone-storm".to_owned()), clone_basis)
        .expect("the clone can fork from its rebound source");
    assert_eq!(outcome.fork_provenance(), outcome.source_observation());
    assert_oracle_matches(&world, &expected);
}

#[test]
fn malformed_fork_target_denies_before_reference_installation() {
    let (world, expected) = certified_supply_chain_world(SupplyChainScale::court());
    assert_oracle_matches(&world, &expected);
    let (_, basis) = world
        .runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("main branch has a fork source");
    let before = capture_reference_evidence(
        &world.runtime,
        &BranchId("main".to_owned()),
        &BranchId(String::new()),
        world.commit.commit_id,
    );

    assert!(matches!(
        world.runtime.fork_branch(BranchId(String::new()), basis),
        Err(RelationalForkDenial::InvalidTarget(error))
            if error.class == worth_relational::facade::history::BranchCreateErrorClass::InvalidTarget
    ));
    assert!(matches!(
        world.runtime.branch_identity(&BranchId(String::new())),
        Err(RelationalBranchIdentityDenial::UnknownBranch(_))
    ));
    let after = capture_reference_evidence(
        &world.runtime,
        &BranchId("main".to_owned()),
        &BranchId(String::new()),
        world.commit.commit_id,
    );
    assert_denial_left_no_reference_residue(&before, &after);
    assert_oracle_matches(&world, &expected);
}

#[test]
fn empty_and_duplicate_fork_denials_preserve_catalog_and_existing_reference() {
    let empty_runtime = canonical_empty_supply_chain_runtime(SupplyChainScale::court());
    let empty_before = capture_reference_evidence(
        &empty_runtime,
        &BranchId("main".to_owned()),
        &BranchId("empty-target".to_owned()),
        worth_relational::facade::history::CommitId(0),
    );
    assert!(matches!(
        empty_runtime.observe_fork_source(&BranchId("main".to_owned())),
        Err(RelationalForkDenial::EmptySource)
    ));
    let empty_after = capture_reference_evidence(
        &empty_runtime,
        &BranchId("main".to_owned()),
        &BranchId("empty-target".to_owned()),
        worth_relational::facade::history::CommitId(0),
    );
    assert_denial_left_no_reference_residue(&empty_before, &empty_after);
    assert_eq!(
        empty_after
            .source_state
            .as_ref()
            .expect("canonical empty Supply Chain main cell")
            .observation()
            .target(),
        &worth_foundational::FoundationalBranchTarget::Empty
    );

    let (world, expected) = certified_supply_chain_world(SupplyChainScale::court());
    assert_oracle_matches(&world, &expected);
    let (_, source_basis) = world
        .runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("installed main branch has an exact fork source");
    let catalog_before = world.runtime.history().immutable_commit_count();
    world
        .runtime
        .fork_branch(BranchId("storm".to_owned()), source_basis)
        .expect("first target fork succeeds");
    let before = capture_reference_evidence(
        &world.runtime,
        &BranchId("main".to_owned()),
        &BranchId("storm".to_owned()),
        world.commit.commit_id,
    );

    let (_, duplicate_basis) = world
        .runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("source remains live after fork");
    let materializations_before = world
        .runtime
        .phase4_reference_cost_counters()
        .artifact_clones;
    assert!(matches!(
        world
            .runtime
            .fork_branch(BranchId("storm".to_owned()), duplicate_basis),
        Err(RelationalForkDenial::DuplicateTarget)
    ));
    assert_eq!(
        world.runtime.history().immutable_commit_count(),
        catalog_before,
        "duplicate target denial must not append a second artifact"
    );
    assert_eq!(
        world
            .runtime
            .phase4_reference_cost_counters()
            .artifact_clones,
        materializations_before,
        "duplicate target denial must not reconstruct an artifact"
    );
    assert!(world
        .runtime
        .branch_identity(&BranchId("storm".to_owned()))
        .is_ok());
    let after = capture_reference_evidence(
        &world.runtime,
        &BranchId("main".to_owned()),
        &BranchId("storm".to_owned()),
        world.commit.commit_id,
    );
    assert_denial_left_no_reference_residue(&before, &after);
    assert_oracle_matches(&world, &expected);
}

#[test]
fn stale_fork_source_denial_does_not_install_a_target_reference() {
    let (world, expected) = certified_supply_chain_world(SupplyChainScale::court());
    assert_oracle_matches(&world, &expected);
    let (_, stale_basis) = world
        .runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("installed main branch has an exact fork source");
    let identity = world.runtime.main_branch_identity();
    let context = world
        .runtime
        .admit_branch_basis(&identity)
        .expect("configured main branch must remain owner-admissible");
    let mut transaction = world
        .runtime
        .begin_branch_transaction(
            &context,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("owner-admitted transaction context");
    transaction
        .push_batch(
            WorkerIntentBatch::new("advance-main-before-fork"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction
        .commit(
            &world.runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("main truth advances");
    let catalog_after_advance = world.runtime.history().immutable_commit_count();
    let before = capture_reference_evidence(
        &world.runtime,
        &BranchId("main".to_owned()),
        &BranchId("stale".to_owned()),
        world.commit.commit_id,
    );

    assert!(matches!(
        world
            .runtime
            .fork_branch(BranchId("stale".to_owned()), stale_basis),
        Err(RelationalForkDenial::StaleSource)
    ));
    assert_eq!(
        world.runtime.history().immutable_commit_count(),
        catalog_after_advance,
        "stale source denial must not append an artifact"
    );
    assert!(matches!(
        world
            .runtime
            .branch_identity(&BranchId("stale".to_owned())),
        Err(RelationalBranchIdentityDenial::UnknownBranch(branch))
            if branch == BranchId("stale".to_owned())
    ));
    let after = capture_reference_evidence(
        &world.runtime,
        &BranchId("main".to_owned()),
        &BranchId("stale".to_owned()),
        world.commit.commit_id,
    );
    assert_denial_left_no_reference_residue(&before, &after);
    assert_oracle_matches(&world, &expected);
}

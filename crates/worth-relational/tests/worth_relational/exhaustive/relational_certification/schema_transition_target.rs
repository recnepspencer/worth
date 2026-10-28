#[allow(unused_imports)]
pub(crate) use crate::fast::relational_certification::schema_transition_target::*;

#[test]
fn two_v1_siblings_transition_independently_and_retained_v1_truth_stays_exact() {
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, _) = certified_supply_chain_world(execution, SupplyChainScale::court());
    let main_identity = world.runtime.main_branch_identity();
    let (_, main_basis) = world.runtime.observe_branch(&main_identity).unwrap();
    let retained_v1 = world
        .runtime
        .snapshots()
        .snapshot_for_observation(&main_basis.observation())
        .unwrap();

    let mut transitioned_roots = Vec::new();
    let mut transitioned_commits = Vec::new();
    for name in ["hazard-v2", "hazard-v2-secondary"] {
        let (_, source) = world
            .runtime
            .observe_fork_source(&BranchId("main".to_owned()))
            .unwrap();
        let branch = BranchId(name.to_owned());
        world
            .runtime
            .fork_branch(execution, branch.clone(), source)
            .unwrap();
        let before = world.runtime.branch_reference_state(&branch).unwrap();
        let batch = lower_supply_chain_production_delta(
            execution,
            &world.runtime,
            &world.program,
            &world.handles,
            &branch,
            &BTreeSet::new(),
            DeltaId::AdoptHazardClassificationV2,
        )
        .unwrap();
        let committed = commit_supply_chain_delta(
            execution,
            &world.runtime,
            &world.program,
            branch.clone(),
            DeltaId::AdoptHazardClassificationV2,
            batch,
        );
        let after = world.runtime.branch_reference_state(&branch).unwrap();
        assert_ne!(
            after, before,
            "each schema transition moves only its branch"
        );
        let observed = observe_supply_chain_snapshot(
            execution,
            &world.program,
            &world.handles.for_snapshot(committed.snapshot.clone()),
            &world.runtime,
            &committed.snapshot,
        )
        .unwrap();
        assert_eq!(observed.schema, SchemaVersion::V2);
        compare(
            &expected_supply_chain_branch(
                &world.program,
                BranchLabel::HazardV2,
                Some(DeltaId::AdoptHazardClassificationV2),
            ),
            &observed,
        )
        .expect("each V2 sibling matches the independent semantic oracle");
        let identity = world.runtime.branch_identity(&branch).unwrap();
        let (_, basis) = world.runtime.observe_branch(&identity).unwrap();
        transitioned_roots.push(basis.observation().selected_root_identity());

        let envelope = world
            .runtime
            .replay()
            .canonical_commit_envelope(execution, committed.commit.commit_id)
            .unwrap()
            .unwrap()
            .clone();
        let replay_record = world.runtime.publication().latest_replay().unwrap().clone();
        assert_eq!(replay_record.commit_id, committed.commit.commit_id);
        assert_eq!(replay_record.schema_authority, envelope.schema_authority);
        transitioned_commits.push((branch, committed.commit.commit_id, after));
        world
            .runtime
            .snapshots()
            .release_snapshot(&committed.snapshot)
            .unwrap();
    }
    assert_ne!(
        transitioned_roots[0], transitioned_roots[1],
        "independent sibling publications install distinct roots"
    );
    for (index, (branch, _, expected_reference)) in transitioned_commits.iter().enumerate() {
        let reference = world.runtime.branch_reference_state(branch).unwrap();
        assert_eq!(&reference, expected_reference);
        let identity = world.runtime.branch_identity(branch).unwrap();
        let (_, basis) = world.runtime.observe_branch(&identity).unwrap();
        assert_eq!(
            basis.observation().selected_root_identity(),
            transitioned_roots[index],
            "a later sibling publication cannot move this branch's selected root"
        );
        let snapshot = world
            .runtime
            .snapshots()
            .snapshot_for_observation(&basis.observation())
            .unwrap();
        let observed = observe_supply_chain_snapshot(
            execution,
            &world.program,
            &world.handles.for_snapshot(snapshot.clone()),
            &world.runtime,
            &snapshot,
        )
        .unwrap();
        compare(
            &expected_supply_chain_branch(
                &world.program,
                BranchLabel::HazardV2,
                Some(DeltaId::AdoptHazardClassificationV2),
            ),
            &observed,
        )
        .expect("both siblings remain exact after both publications");
        world
            .runtime
            .snapshots()
            .release_snapshot(&snapshot)
            .unwrap();
    }
    let (replay_branch, replay_commit, _) = transitioned_commits[0].clone();
    let replay = world
        .runtime
        .replay_authority()
        .replay_commit(
            RelationalReplayRequest {
                commit_id: replay_commit,
                branch_id: replay_branch,
                execution_mode: ReplayExecutionMode::SerialDeterministic,
                verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
            },
            execution,
        )
        .unwrap();
    assert_eq!(replay.failure, None, "V2 sibling replay failed: {replay:?}");
    assert!(replay.mismatches.is_empty());

    let historical = observe_supply_chain_snapshot(
        execution,
        &world.program,
        &world.handles.for_snapshot(retained_v1.clone()),
        &world.runtime,
        &retained_v1,
    )
    .unwrap();
    assert_eq!(historical.schema, SchemaVersion::V1);
    compare(
        &expected_supply_chain_branch(&world.program, BranchLabel::Operating, None),
        &historical,
    )
    .expect("the retained V1 root preserves complete semantic truth");
    world
        .runtime
        .snapshots()
        .release_snapshot(&retained_v1)
        .unwrap();
}

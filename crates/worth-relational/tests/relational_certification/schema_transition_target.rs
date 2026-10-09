use std::collections::BTreeSet;

use super::invariant_oracle_expectations::expected_supply_chain_branch;
use super::world::supply_chain::{
    assert_oracle_matches, certified_supply_chain_world, commit_supply_chain_delta, compare,
    entity_kind_id, hazard_v2_transition, lower_supply_chain_production_delta,
    observe_supply_chain_snapshot, schema_registry_with_altered_port_contract, BranchLabel,
    DeltaId, EntityKind, SchemaVersion, SupplyChainScale,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::replay::{
    RelationalReplayRequest, ReplayExecutionMode, ReplayVerificationMode,
};
use worth_relational::facade::runtime::RelationalSchemaTransitionAdmissionDenialKind;
use worth_relational::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry, SchemaId,
    SchemaVersionId,
};

#[test]
fn branch_schema_target_admission_is_typed_and_never_moves_a_branch() {
    let (world, baseline) = certified_supply_chain_world(SupplyChainScale::court());
    assert_oracle_matches(&world, &baseline);
    let main = BranchId("main".to_owned());
    let before = world.runtime.branch_reference_state(&main).unwrap();
    let identity = world.runtime.main_branch_identity();
    let basis = world.runtime.admit_branch_basis(&identity).unwrap();

    let mut wrong_source = hazard_v2_transition();
    wrong_source.source_schema_version_id = SchemaVersionId(7);
    let denial = world
        .runtime
        .begin_branch_schema_transition(
            &basis,
            wrong_source,
            None,
            world
                .program
                .schema_registry_for_version(SchemaVersion::V2)
                .unwrap(),
        )
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        RelationalSchemaTransitionAdmissionDenialKind::SourceBasisMismatch
    );
    assert_eq!(world.runtime.branch_reference_state(&main).unwrap(), before);

    let altered_target = schema_registry_with_altered_port_contract(SchemaVersionId(2)).unwrap();
    let denial = world
        .runtime
        .begin_branch_schema_transition(&basis, hazard_v2_transition(), None, altered_target)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        RelationalSchemaTransitionAdmissionDenialKind::TargetContractMismatch
    );
    assert_eq!(world.runtime.branch_reference_state(&main).unwrap(), before);

    let partial_target = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: entity_kind_id(EntityKind::Port),
            kind_name: "supply_chain.entity.Port".to_owned(),
            schema_id: SchemaId("supply_chain".to_owned()),
            schema_version_id: SchemaVersionId(2),
            aspect_contract_declarations: KindAspectContractDeclarations::new(Vec::new()),
        })
        .unwrap();
    let denial = world
        .runtime
        .begin_branch_schema_transition(&basis, hazard_v2_transition(), None, partial_target)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        RelationalSchemaTransitionAdmissionDenialKind::TargetContractMismatch
    );
    assert_eq!(world.runtime.branch_reference_state(&main).unwrap(), before);

    let transition = hazard_v2_transition();
    let denial = world
        .runtime
        .begin_branch_schema_transition(
            &basis,
            transition.clone(),
            None,
            world
                .program
                .schema_registry_for_version(SchemaVersion::V1)
                .unwrap(),
        )
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        RelationalSchemaTransitionAdmissionDenialKind::TargetBasisMismatch
    );
    assert_eq!(world.runtime.branch_reference_state(&main).unwrap(), before);

    let transaction = world
        .runtime
        .begin_branch_schema_transition(
            &basis,
            transition,
            None,
            world
                .program
                .schema_registry_for_version(SchemaVersion::V2)
                .unwrap(),
        )
        .unwrap();
    drop(transaction);
    assert_eq!(world.runtime.branch_reference_state(&main).unwrap(), before);
}

#[test]
fn two_v1_siblings_transition_independently_and_retained_v1_truth_stays_exact() {
    let (world, _) = certified_supply_chain_world(SupplyChainScale::court());
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
        world.runtime.fork_branch(branch.clone(), source).unwrap();
        let before = world.runtime.branch_reference_state(&branch).unwrap();
        let batch = lower_supply_chain_production_delta(
            &world.runtime,
            &world.program,
            &world.handles,
            &branch,
            &BTreeSet::new(),
            DeltaId::AdoptHazardClassificationV2,
        )
        .unwrap();
        let committed = commit_supply_chain_delta(
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
            .canonical_commit_envelope(committed.commit.commit_id)
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
        .replay_commit(RelationalReplayRequest {
            commit_id: replay_commit,
            branch_id: replay_branch,
            execution_mode: ReplayExecutionMode::SerialDeterministic,
            verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
        });
    assert_eq!(replay.failure, None, "V2 sibling replay failed: {replay:?}");
    assert!(replay.mismatches.is_empty());

    let historical = observe_supply_chain_snapshot(
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

#[test]
fn stale_schema_transition_basis_cannot_install_target_authority() {
    let (world, _) = certified_supply_chain_world(SupplyChainScale::court());
    let identity = world.runtime.main_branch_identity();
    let basis = world.runtime.admit_branch_basis(&identity).unwrap();
    let ordinary = world
        .runtime
        .begin_branch_transaction(
            &basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    let committed = ordinary
        .commit(
            &world.runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    world
        .runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();

    let denial = world
        .runtime
        .begin_branch_schema_transition(
            &basis,
            hazard_v2_transition(),
            None,
            world
                .program
                .schema_registry_for_version(SchemaVersion::V2)
                .unwrap(),
        )
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        RelationalSchemaTransitionAdmissionDenialKind::TransactionAdmission
    );
    let fresh_basis = world.runtime.admit_branch_basis(&identity).unwrap();
    let snapshot = world
        .runtime
        .snapshots()
        .snapshot_for_observation(&fresh_basis.observation())
        .unwrap();
    let observed = observe_supply_chain_snapshot(
        &world.program,
        &world.handles.for_snapshot(snapshot.clone()),
        &world.runtime,
        &snapshot,
    )
    .unwrap();
    assert_eq!(observed.schema, SchemaVersion::V1);
    world
        .runtime
        .snapshots()
        .release_snapshot(&snapshot)
        .unwrap();
}

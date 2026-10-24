mod fixtures;
mod serde_support;

use crate::facade::merge::{
    AspectMergePolicyKind, MergeExecutionAuthorityContract, RelationalMergeStrategyWitness,
};
use crate::merge::data::{RelationalMergeProofPacket, RelationalMergeTopologyStrategyWitnessRow};
use crate::tests::support::{checkpoint_and_recover_with, create_branch_from_main, create_entity};

use fixtures::{
    deletion_row, durable_runtime_with_schema_declared_entity_policy, merge_request, policy_row,
    published_merge_authority, runtime_with_schema_declared_entity_policy, strategy_witness,
    topology_row, update_entity_status_on_branch,
};
use serde_support::{strategy_witness_payload, StrategyWitnessPayloadMutator};

#[test]
fn retained_merge_strategy_witness_preserves_exact_strategy_truth() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_schema_declared_entity_policy(AspectMergePolicyKind::PreferRicher);
    let shared = create_entity(execution, &runtime, "shared");
    create_branch_from_main(&runtime, "feature");
    update_entity_status_on_branch(execution, &runtime, shared, "inactive", "main");
    update_entity_status_on_branch(execution, &runtime, shared, "active", "feature");

    let prepared = runtime
        .merge()
        .prepare_merge_execution(execution, merge_request())
        .expect("prepared merge");
    let planning_witness = runtime
        .merge()
        .retain_merge_strategy_witness_from_planning_artifact(prepared.artifact());
    let prepared_witness = runtime
        .merge()
        .retain_merge_strategy_witness_from_prepared_execution(&prepared);

    assert_eq!(planning_witness, prepared_witness);
    assert_eq!(
        planning_witness.request_digest(),
        prepared.request().request_digest()
    );
    assert_eq!(
        planning_witness.branch_basis_digest(),
        prepared.execution_ready_plan().basis.basis_digest()
    );
    assert_eq!(
        planning_witness.execution_authority_contract(),
        &prepared.artifact().execution_authority_contract
    );
    assert!(!planning_witness.aspect_policy_rows().is_empty());
    assert!(planning_witness.topology_rows().is_empty());
    assert!(planning_witness.deletion_rows().is_empty());
    assert!(planning_witness.aspect_policy_rows().iter().any(|row| {
        row.applied_policies()
            .iter()
            .any(|policy| policy.policy == AspectMergePolicyKind::PreferRicher)
    }));
}

#[test]
fn retained_merge_strategy_witness_rejects_forged_or_rehashed_truth() {
    let witness = strategy_witness(
        vec![policy_row(AspectMergePolicyKind::PreferRicher)],
        vec![topology_row(
            crate::facade::merge::TopologyExecutionClass::RelationEndpointRewiredLocal,
            crate::facade::merge::MergeExecutionReadiness::Blocked,
        )],
        vec![deletion_row(
            crate::facade::merge::DeletionExecutionClass::DeletedVsModified,
            crate::facade::merge::MergeExecutionReadiness::Blocked,
        )],
        honest_contract(),
    );
    let encoded = rmp_serde::to_vec_named(&witness).expect("encode strategy witness");
    let decoded: RelationalMergeStrategyWitness =
        rmp_serde::from_slice(&encoded).expect("decode strategy witness");
    assert_eq!(decoded, witness);

    let forged_topology: Result<RelationalMergeStrategyWitness, _> = rmp_serde::from_slice(
        &rmp_serde::to_vec_named(&strategy_witness_payload(
            &witness,
            StrategyWitnessPayloadMutator {
                topology_class: Some(
                    crate::facade::merge::TopologyExecutionClass::TopologyRegionConflict,
                ),
                witness_digest: None,
                execution_authority_contract: None,
                topology_readiness: None,
                deletion_class: None,
                policy_kind: None,
            },
        ))
        .expect("encode forged topology payload"),
    );
    assert!(forged_topology.is_err());

    let forged_digest: Result<RelationalMergeStrategyWitness, _> = rmp_serde::from_slice(
        &rmp_serde::to_vec_named(&strategy_witness_payload(
            &witness,
            StrategyWitnessPayloadMutator {
                witness_digest: Some("forged-strategy-witness-digest"),
                execution_authority_contract: None,
                topology_readiness: None,
                topology_class: None,
                deletion_class: None,
                policy_kind: None,
            },
        ))
        .expect("encode forged digest payload"),
    );
    assert!(forged_digest.is_err());

    let forged_topology_readiness: Result<RelationalMergeStrategyWitness, _> =
        rmp_serde::from_slice(
            &rmp_serde::to_vec_named(&strategy_witness_payload(
                &witness,
                StrategyWitnessPayloadMutator {
                    witness_digest: None,
                    execution_authority_contract: None,
                    topology_readiness: Some(
                        crate::facade::merge::MergeExecutionReadiness::Admitted,
                    ),
                    topology_class: Some(
                        crate::facade::merge::TopologyExecutionClass::RelationEndpointRewiredLocal,
                    ),
                    deletion_class: None,
                    policy_kind: None,
                },
            ))
            .expect("encode forged topology readiness payload"),
        );
    assert!(forged_topology_readiness.is_err());
}

#[test]
fn retained_merge_strategy_witness_differentiates_policy_topology_and_deletion_subidentity() {
    let contract = honest_contract();
    let baseline = strategy_witness(
        vec![policy_row(AspectMergePolicyKind::PreferRicher)],
        vec![topology_row(
            crate::facade::merge::TopologyExecutionClass::RelationEndpointRewiredLocal,
            crate::facade::merge::MergeExecutionReadiness::Blocked,
        )],
        vec![deletion_row(
            crate::facade::merge::DeletionExecutionClass::DeletedVsModified,
            crate::facade::merge::MergeExecutionReadiness::Blocked,
        )],
        contract.clone(),
    );
    let policy_drift = strategy_witness(
        vec![policy_row(AspectMergePolicyKind::FailOnConflict)],
        vec![topology_row(
            crate::facade::merge::TopologyExecutionClass::RelationEndpointRewiredLocal,
            crate::facade::merge::MergeExecutionReadiness::Blocked,
        )],
        vec![deletion_row(
            crate::facade::merge::DeletionExecutionClass::DeletedVsModified,
            crate::facade::merge::MergeExecutionReadiness::Blocked,
        )],
        contract.clone(),
    );
    let topology_drift = strategy_witness(
        vec![policy_row(AspectMergePolicyKind::PreferRicher)],
        vec![topology_row(
            crate::facade::merge::TopologyExecutionClass::TopologyRegionConflict,
            crate::facade::merge::MergeExecutionReadiness::Blocked,
        )],
        vec![deletion_row(
            crate::facade::merge::DeletionExecutionClass::DeletedVsModified,
            crate::facade::merge::MergeExecutionReadiness::Blocked,
        )],
        contract.clone(),
    );
    let deletion_drift = strategy_witness(
        vec![policy_row(AspectMergePolicyKind::PreferRicher)],
        vec![topology_row(
            crate::facade::merge::TopologyExecutionClass::RelationEndpointRewiredLocal,
            crate::facade::merge::MergeExecutionReadiness::Blocked,
        )],
        vec![deletion_row(
            crate::facade::merge::DeletionExecutionClass::DeletedVsRewired,
            crate::facade::merge::MergeExecutionReadiness::Blocked,
        )],
        contract,
    );

    assert_ne!(
        baseline.aspect_policy_rows()[0].row_digest(),
        policy_drift.aspect_policy_rows()[0].row_digest()
    );
    assert_ne!(
        baseline.topology_rows()[0].row_digest(),
        topology_drift.topology_rows()[0].row_digest()
    );
    assert_ne!(
        baseline.deletion_rows()[0].row_digest(),
        deletion_drift.deletion_rows()[0].row_digest()
    );
    assert_ne!(baseline.witness_digest(), policy_drift.witness_digest());
    assert_ne!(baseline.witness_digest(), topology_drift.witness_digest());
    assert_ne!(baseline.witness_digest(), deletion_drift.witness_digest());
}

#[test]
fn retained_merge_strategy_witness_survives_prepared_execution_publication_and_recovery() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime =
        durable_runtime_with_schema_declared_entity_policy(AspectMergePolicyKind::PreferRicher);
    let shared = create_entity(execution, &runtime, "shared");
    create_branch_from_main(&runtime, "feature");
    update_entity_status_on_branch(execution, &runtime, shared, "inactive", "main");
    update_entity_status_on_branch(execution, &runtime, shared, "active", "feature");

    let prepared = runtime
        .merge()
        .prepare_merge_execution(execution, merge_request())
        .expect("prepared merge");
    let live_witness = runtime
        .merge()
        .retain_merge_strategy_witness_from_prepared_execution(&prepared);
    let outcome = runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed merge");
    let live_authority = published_merge_authority(&runtime, outcome.commit.commit.commit_id);
    let (_recovery, recovered) = checkpoint_and_recover_with(execution, &runtime, || {
        durable_runtime_with_schema_declared_entity_policy(AspectMergePolicyKind::PreferRicher)
    });
    let recovered_authority =
        published_merge_authority(&recovered, outcome.commit.commit.commit_id);

    assert_eq!(live_witness, outcome.execution_summary.strategy_witness);
    assert_eq!(
        live_witness,
        live_authority.execution_summary.strategy_witness
    );
    assert_eq!(
        live_witness,
        recovered_authority.execution_summary.strategy_witness
    );
    assert_eq!(
        live_witness.witness_digest(),
        outcome
            .execution_summary
            .proof_packet
            .strategy_witness_digest()
    );
    assert!(outcome
        .execution_summary
        .retains_consistent_proof_packet_authority());
}

#[test]
fn retained_merge_strategy_witness_live_authority_denies_internal_forged_strategy_truth() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_schema_declared_entity_policy(AspectMergePolicyKind::PreferRicher);
    let shared = create_entity(execution, &runtime, "shared");
    create_branch_from_main(&runtime, "feature");
    update_entity_status_on_branch(execution, &runtime, shared, "inactive", "main");
    update_entity_status_on_branch(execution, &runtime, shared, "active", "feature");

    let prepared = runtime
        .merge()
        .prepare_merge_execution(execution, merge_request())
        .expect("prepared merge");
    let outcome = runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed merge");
    let honest_summary = outcome.execution_summary.clone();
    let honest_witness = honest_summary.strategy_witness.clone();
    let forged_witness = RelationalMergeStrategyWitness::retained(
        honest_witness.request_digest().to_string(),
        honest_witness.branch_basis_digest().to_string(),
        honest_witness.execution_authority_contract().clone(),
        std::sync::Arc::from(honest_witness.aspect_policy_rows().to_vec()),
        std::sync::Arc::from(vec![RelationalMergeTopologyStrategyWitnessRow::retained(
            honest_witness.aspect_policy_rows()[0].record().clone(),
            honest_witness.aspect_policy_rows()[0]
                .target_record()
                .cloned(),
            crate::facade::merge::TopologyExecutionClass::TopologyRegionConflict,
            crate::facade::merge::MergeExecutionReadiness::Admitted,
            None,
        )]),
        std::sync::Arc::from(honest_witness.deletion_rows().to_vec()),
    );
    let honest_packet = honest_summary.proof_packet.clone();
    let forged_packet = RelationalMergeProofPacket::retained_execution_admitted(
        honest_packet.request().clone(),
        honest_packet.branch_basis().clone(),
        std::sync::Arc::from(honest_packet.admitted_merge_surface().to_vec()),
        honest_packet.correspondence_witness_digest().to_string(),
        honest_packet
            .schema_reconciliation_witness_digest()
            .to_string(),
        forged_witness.witness_digest().to_string(),
        honest_packet
            .foundational_request_lowering_digest()
            .to_string(),
        honest_packet.planning_digest().to_string(),
        honest_packet.execution_digest().to_string(),
    );
    let forged_summary = crate::transactions::data::MergeExecutionSummary {
        strategy_witness: forged_witness,
        proof_packet: forged_packet,
        ..honest_summary
    };

    assert!(!forged_summary.retains_consistent_proof_packet_authority());
}

fn honest_contract() -> MergeExecutionAuthorityContract {
    MergeExecutionAuthorityContract {
        decision_surface: crate::facade::merge::MergeExecutionDecisionSurface::LoweredRecordDecisionOnly,
        identity_authority: crate::facade::merge::MergeExecutionConsumptionRule::ConsumeCanonicalLoweredArtifactOnly,
        conflict_authority: crate::facade::merge::MergeExecutionConsumptionRule::ConsumeCanonicalLoweredArtifactOnly,
        policy_authority: crate::facade::merge::MergeExecutionConsumptionRule::ConsumeCanonicalLoweredArtifactOnly,
        value_authorization: crate::facade::merge::MergeExecutionAuthorizationRule::MustNotWidenBeyondAuthorizedAspectValueSurface,
    }
}

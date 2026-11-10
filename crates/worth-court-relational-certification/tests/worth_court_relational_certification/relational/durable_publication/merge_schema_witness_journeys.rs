use super::branch_history_publications::*;
use super::merge_proof_journey::{merge_request, published_merge_authority};
use super::merge_witness_authoring::*;
use super::merge_witness_schema::policy_builder;
use super::merge_witness_schema::runtime_with_relation_identity_registry;
use worth_relational::facade::merge::*;
use worth_relational::facade::schema::{
    SchemaReconciliationClassification, SchemaReconciliationPolicy,
};
use worth_testkit_relational::durable_medium::TestDurableMedium;
#[test]
fn retained_schema_reconciliation_witness_preserves_runtime_additive_truth_across_authority_lanes()
{
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let builder = policy_builder(AspectMergePolicyKind::PreferRicher, true);
    let runtime = builder.clone().build_durable(medium.attachment()).unwrap();
    create_named_entity_on_branch(
        execution,
        &runtime,
        "main-shared",
        "shared-name",
        None,
        "main",
    );
    fork(execution, &runtime, "feature");
    create_named_entity_on_branch(
        execution,
        &runtime,
        "feature-shared",
        "shared-name",
        Some("active"),
        "feature",
    );

    let prepared = runtime
        .merge()
        .prepare_merge_execution(
            execution,
            runtime
                .bind_merge_execution_request(merge_request())
                .unwrap(),
        )
        .expect("prepared merge");
    let planning_witness = prepared.artifact().schema_reconciliation_witness.clone();
    let additive_row = planning_witness
        .rows()
        .iter()
        .find(|row| row.classification() == SchemaReconciliationClassification::Additive)
        .expect("runtime additive schema witness row");
    let outcome = runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed merge");
    let live_authority =
        published_merge_authority(execution, &runtime, outcome.commit.commit.commit_id);
    super::branch_history_recovery::checkpoint(execution, &runtime, &medium);
    let (_, recovered) = super::branch_history_recovery::recover(
        execution,
        &medium,
        builder.build_durable(medium.attachment()).unwrap(),
    );
    let recovered_authority =
        published_merge_authority(execution, &recovered, outcome.commit.commit.commit_id);

    assert_eq!(
        additive_row.policy(),
        Some(SchemaReconciliationPolicy::PreserveInformation)
    );
    assert_eq!(additive_row.denial(), None);
    assert_eq!(
        additive_row.posture(),
        RelationalSchemaReconciliationWitnessPosture::Reconciled
    );
    assert!(additive_row.correspondence_linkage().is_some());
    assert!(!planning_witness.rows().is_empty());
    assert_eq!(
        planning_witness,
        outcome.execution_summary.schema_reconciliation_witness
    );
    assert_eq!(
        planning_witness,
        live_authority
            .execution_summary
            .schema_reconciliation_witness
    );
    assert_eq!(
        planning_witness,
        recovered_authority
            .execution_summary
            .schema_reconciliation_witness
    );
    assert_eq!(
        planning_witness.witness_digest(),
        outcome
            .execution_summary
            .proof_packet
            .schema_reconciliation_witness_digest()
    );
}
#[test]
fn retained_schema_reconciliation_witness_keeps_runtime_structural_denial_typed() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = runtime_with_relation_identity_registry(&medium);
    let source = create_empty_entity(execution, &runtime, "source");
    let target_a = create_empty_entity(execution, &runtime, "target-a");
    let target_b = create_empty_entity(execution, &runtime, "target-b");
    let target_c = create_empty_entity(execution, &runtime, "target-c");
    let relation = create_relation(execution, &runtime, source, target_a, "edge-a");
    fork(execution, &runtime, "feature");
    delete_relation_on_branch(
        execution,
        &runtime,
        relation,
        worth_relational::facade::history::BranchId("feature".to_string()),
    );
    create_relation_on_branch(
        execution,
        &runtime,
        BranchId("feature".into()),
        RelationSpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(2),
            client_key: "edge-a".into(),
            source: EntityReference::Existing(source),
            target: EntityReference::Existing(target_b),
            fields: field_patch("label", "label", "edge-a"),
        },
    );
    create_relation_on_branch(
        execution,
        &runtime,
        BranchId("feature".into()),
        RelationSpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(2),
            client_key: "edge-c".into(),
            source: EntityReference::Existing(source),
            target: EntityReference::Existing(target_c),
            fields: field_patch("label", "label", "edge-c"),
        },
    );

    let artifact = runtime
        .merge()
        .inspect_planning_scope(
            execution,
            runtime
                .bind_merge_planning_request(merge_request().into())
                .unwrap(),
        )
        .expect("planning artifact");
    let row = artifact
        .schema_reconciliation_witness
        .rows()
        .iter()
        .find(|row| {
            row.classification() == SchemaReconciliationClassification::StructuralContinuityDenied
        })
        .expect("runtime structural schema witness row");

    assert_eq!(
        row.denial(),
        Some(RelationalSchemaReconciliationWitnessDenial::StructuralIncompatible)
    );
    assert_eq!(row.policy(), None);
    assert_eq!(
        row.posture(),
        RelationalSchemaReconciliationWitnessPosture::Denied
    );
}

use worth_relational::facade::{
    history::BranchId,
    identity::{KindId, PartitionId},
    transactions::{EntityReference, RelationSpec},
};
use worth_testkit_relational::durable_publication::field_patch;

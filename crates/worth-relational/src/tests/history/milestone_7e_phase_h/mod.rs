mod fixtures;
mod serde_support;

use crate::facade::merge::{
    AspectMergePolicyKind, RelationalSchemaReconciliationWitness,
    RelationalSchemaReconciliationWitnessDenial, RelationalSchemaReconciliationWitnessPosture,
};
use crate::facade::schema::{SchemaReconciliationClassification, SchemaReconciliationPolicy};
use crate::tests::support::create_branch_from_main;

use fixtures::{
    additive_row, create_named_entity_on_branch, denied_narrowing_row, merge_planning_request,
    merge_request, runtime_with_schema_declared_entity_policy, structural_incompatible_row,
    synthetic_witness, type_incompatible_row,
};
use serde_support::{row_payloads, witness_payload};

#[test]
fn retained_schema_reconciliation_witness_preserves_exact_category_and_linkage_truth() {
    let witness = synthetic_witness(vec![
        additive_row(),
        denied_narrowing_row(),
        type_incompatible_row(),
        structural_incompatible_row(),
    ]);
    let rows = witness.rows();

    assert_eq!(
        rows[0].classification(),
        SchemaReconciliationClassification::Additive
    );
    assert_eq!(
        rows[0].policy(),
        Some(SchemaReconciliationPolicy::PreserveInformation)
    );
    assert_eq!(rows[0].denial(), None);
    assert!(rows[0].correspondence_linkage().is_some());

    assert_eq!(
        rows[1].classification(),
        SchemaReconciliationClassification::Narrowing
    );
    assert_eq!(rows[1].policy(), None);
    assert_eq!(
        rows[1].denial(),
        Some(RelationalSchemaReconciliationWitnessDenial::UnvalidatedSchemaCorrespondence)
    );
    assert_eq!(
        rows[1].posture(),
        RelationalSchemaReconciliationWitnessPosture::Denied
    );

    assert_eq!(
        rows[2].classification(),
        SchemaReconciliationClassification::TypeContinuityDenied
    );
    assert_eq!(
        rows[2].denial(),
        Some(RelationalSchemaReconciliationWitnessDenial::PolicyRejected)
    );

    assert_eq!(
        rows[3].classification(),
        SchemaReconciliationClassification::StructuralContinuityDenied
    );
    assert_eq!(
        rows[3].denial(),
        Some(RelationalSchemaReconciliationWitnessDenial::StructuralIncompatible)
    );
}

#[test]
fn retained_schema_reconciliation_witness_rejects_forged_or_rehashed_truth() {
    let witness = synthetic_witness(vec![additive_row()]);
    let encoded = rmp_serde::to_vec_named(&witness).expect("encode witness");
    let decoded: RelationalSchemaReconciliationWitness =
        rmp_serde::from_slice(&encoded).expect("decode witness");
    assert_eq!(decoded, witness);

    let forged_truth = witness_payload(
        &witness,
        row_payloads(
            witness.rows(),
            Some(SchemaReconciliationClassification::Narrowing),
            None,
            None,
        ),
        None,
    );
    let forged_truth_result: Result<RelationalSchemaReconciliationWitness, _> =
        rmp_serde::from_slice(
            &rmp_serde::to_vec_named(&forged_truth).expect("encode forged classification payload"),
        );
    assert!(forged_truth_result.is_err());

    let forged_posture = witness_payload(
        &witness,
        row_payloads(
            witness.rows(),
            None,
            Some(RelationalSchemaReconciliationWitnessPosture::Denied),
            Some(RelationalSchemaReconciliationWitnessDenial::ManualResolutionRequired),
        ),
        None,
    );
    let forged_posture_result: Result<RelationalSchemaReconciliationWitness, _> =
        rmp_serde::from_slice(
            &rmp_serde::to_vec_named(&forged_posture).expect("encode forged posture payload"),
        );
    assert!(forged_posture_result.is_err());

    let forged_witness_digest = witness_payload(
        &witness,
        row_payloads(witness.rows(), None, None, None),
        Some("forged-witness-digest"),
    );
    let forged_witness_result: Result<RelationalSchemaReconciliationWitness, _> =
        rmp_serde::from_slice(
            &rmp_serde::to_vec_named(&forged_witness_digest)
                .expect("encode forged witness digest payload"),
        );
    assert!(forged_witness_result.is_err());
}

#[test]
fn retained_schema_reconciliation_witness_preserves_runtime_narrowing_truth() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_schema_declared_entity_policy(AspectMergePolicyKind::LastWriterWins);
    create_named_entity_on_branch(
        execution,
        &runtime,
        "main-shared",
        "shared-name",
        Some("active"),
        "main",
    );
    create_branch_from_main(&runtime, "feature");
    create_named_entity_on_branch(
        execution,
        &runtime,
        "feature-shared",
        "shared-name",
        None,
        "feature",
    );

    let prepared = runtime
        .merge()
        .prepare_merge_execution(execution, merge_request())
        .expect("prepared merge");
    let row = prepared
        .artifact()
        .schema_reconciliation_witness
        .rows()
        .iter()
        .find(|row| row.classification() == SchemaReconciliationClassification::Narrowing)
        .expect("runtime narrowing schema witness row");

    assert_eq!(
        row.policy(),
        Some(SchemaReconciliationPolicy::PreserveTargetContract)
    );
    assert_eq!(row.denial(), None);
    assert_eq!(
        row.posture(),
        RelationalSchemaReconciliationWitnessPosture::Reconciled
    );
    assert!(row.correspondence_linkage().is_some());
}

#[test]
fn retained_schema_reconciliation_witness_keeps_runtime_type_denial_typed() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_schema_declared_entity_policy(AspectMergePolicyKind::FailOnConflict);
    create_named_entity_on_branch(
        execution,
        &runtime,
        "main-shared",
        "shared-name",
        Some("inactive"),
        "main",
    );
    create_branch_from_main(&runtime, "feature");
    create_named_entity_on_branch(
        execution,
        &runtime,
        "feature-shared",
        "shared-name",
        Some("active"),
        "feature",
    );

    let artifact = runtime
        .merge()
        .inspect_planning_scope(execution, merge_planning_request())
        .expect("planning artifact");
    let row = artifact
        .schema_reconciliation_witness
        .rows()
        .iter()
        .find(|row| {
            row.classification() == SchemaReconciliationClassification::TypeContinuityDenied
        })
        .expect("runtime type-incompatible schema witness row");

    assert_eq!(
        row.denial(),
        Some(RelationalSchemaReconciliationWitnessDenial::PolicyRejected)
    );
    assert_eq!(row.policy(), None);
    assert_eq!(
        row.posture(),
        RelationalSchemaReconciliationWitnessPosture::Denied
    );
    assert!(row.correspondence_linkage().is_some());
}

mod serde_support;

use crate::facade::history::BranchId;
use crate::facade::identity::KindId;
use crate::facade::merge::{
    MergeExecutionRequest, MergeIntent, RelationalMergeCorrespondenceWitness,
    RelationalMergeCorrespondenceWitnessPosture,
};
use crate::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use crate::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationIntegrityDeclarations,
    RelationKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use crate::tests::support::{
    aspect_key, create_branch_from_main, create_entity, create_entity_outcome_on_branch,
    entity_field_aspect, field_key, runtime_with_test_schema, update_entity,
    update_entity_on_branch, CascadeDeletePolicy, CrossContextPolicy,
};
use std::sync::Arc;

use serde_support::{forged_row_with_basis, recomputed_witness, row_payloads, witness_payload};

#[test]
fn retained_merge_correspondence_witness_rejects_forged_or_incomplete_truth() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = merge_ready_runtime(execution);
    let prepared = runtime
        .merge()
        .prepare_merge_execution(execution, merge_request())
        .expect("prepared merge");
    let witness = runtime
        .merge()
        .retain_merge_correspondence_witness_from_prepared_execution(&prepared);

    let encoded = rmp_serde::to_vec_named(&witness).expect("encode witness");
    let decoded: RelationalMergeCorrespondenceWitness =
        rmp_serde::from_slice(&encoded).expect("decode witness");
    assert_eq!(decoded, witness);

    let forged_digest = rmp_serde::to_vec_named(&witness_payload(
        &witness,
        row_payloads(
            witness.rows(),
            Some("forged-row-candidate-digest"),
            None,
            None,
        ),
        Some(witness.witness_digest()),
    ))
    .expect("encode forged row digest payload");
    let forged_digest_result: Result<RelationalMergeCorrespondenceWitness, _> =
        rmp_serde::from_slice(&forged_digest);
    assert!(forged_digest_result.is_err());

    let forged_witness_digest = rmp_serde::to_vec_named(&witness_payload(
        &witness,
        row_payloads(witness.rows(), None, None, None),
        Some("forged-witness-digest"),
    ))
    .expect("encode forged witness payload");
    let forged_witness_result: Result<RelationalMergeCorrespondenceWitness, _> =
        rmp_serde::from_slice(&forged_witness_digest);
    assert!(forged_witness_result.is_err());

    let forged_rows = row_payloads(
        witness.rows(),
        None,
        Some(RelationalMergeCorrespondenceWitnessPosture::DeniedAmbiguous),
        None,
    );
    let forged_row_result: Result<RelationalMergeCorrespondenceWitness, _> = rmp_serde::from_slice(
        &rmp_serde::to_vec_named(&witness_payload(
            &witness,
            forged_rows.clone(),
            Some(witness.witness_digest()),
        ))
        .expect("encode forged row payload"),
    );
    assert!(forged_row_result.is_err());

    let forged_row_with_recomputed_witness = witness_payload(&witness, forged_rows, None);
    let forged_row_with_recomputed_witness_result: Result<RelationalMergeCorrespondenceWitness, _> =
        rmp_serde::from_slice(
            &rmp_serde::to_vec_named(&forged_row_with_recomputed_witness)
                .expect("encode recomputed forged row payload"),
        );
    assert!(forged_row_with_recomputed_witness_result.is_err());
}

#[test]
fn retained_merge_correspondence_witness_preserves_authority_basis_distinctions() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = merge_ready_runtime(execution);
    let prepared = runtime
        .merge()
        .prepare_merge_execution(execution, merge_request())
        .expect("prepared merge");
    let witness = runtime
        .merge()
        .retain_merge_correspondence_witness_from_prepared_execution(&prepared);
    let admitted_row = witness
        .admitted_rows()
        .next()
        .expect("admitted correspondence row")
        .clone();

    let identical_rows = vec![admitted_row.clone()];
    let identical_witness = recomputed_witness(&witness, identical_rows.clone());
    assert_eq!(
        identical_witness,
        recomputed_witness(&witness, identical_rows)
    );

    let lineage_shifted_row = forged_row_with_basis(
        &admitted_row,
        crate::facade::merge::IdentityBasisKind::LineageIdentity,
    );
    let lineage_shifted_witness = recomputed_witness(&witness, vec![lineage_shifted_row]);
    assert_ne!(lineage_shifted_witness, identical_witness);
    assert_ne!(
        lineage_shifted_witness.witness_digest(),
        identical_witness.witness_digest()
    );
}

#[test]
fn retained_merge_correspondence_witness_preserves_runtime_declared_key_ambiguity_as_denial() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_declared_key_identity_registry();
    create_entity(execution, &runtime, "shared-name");
    create_entity(execution, &runtime, "shared-name");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "shared-name",
        BranchId("feature".to_string()),
    );

    let artifact = runtime
        .merge()
        .inspect_planning_scope(
            execution,
            crate::facade::merge::MergePlanningRequest::new(
                BranchId("main".to_string()),
                BranchId("feature".to_string()),
                MergeIntent::ReconcileIntoTarget,
            ),
        )
        .expect("planning artifact");
    let witness = runtime
        .merge()
        .retain_merge_correspondence_witness_from_planning_artifact(&artifact);
    let denied_row = witness
        .rows()
        .iter()
        .find(|row| row.posture() == RelationalMergeCorrespondenceWitnessPosture::DeniedAmbiguous)
        .expect("declared-key ambiguous correspondence row");

    assert!(matches!(
        denied_row.reason(),
        crate::facade::merge::IdentityResolutionReason::DeclaredBasisAmbiguousVisibleTargetMatch
    ));
    assert!(matches!(
        denied_row.authority_basis(),
        crate::facade::merge::IdentityBasisKind::DeclaredKeySet(_)
    ));
    assert!(denied_row.target_record().is_none());
}

fn merge_ready_runtime(execution: &worth_execution::ExecutionRequest<'_, '_>) -> RelationalRuntime {
    let runtime = runtime_with_test_schema();
    let shared = create_entity(execution, &runtime, "shared");
    create_branch_from_main(&runtime, "feature");
    update_entity(execution, &runtime, shared, "shared-value");
    update_entity_on_branch(
        execution,
        &runtime,
        shared,
        "shared-value",
        BranchId("feature".to_string()),
    );
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );
    runtime
}

fn runtime_with_declared_key_identity_registry() -> RelationalRuntime {
    let name_key = aspect_key("name");
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                entity_field_aspect(name_key.clone(), field_key("name")),
            ])
            .with_identity_declarations(vec![
                crate::facade::merge::IdentityBasisDeclaration {
                    scope: crate::facade::merge::IdentityBasisScope::AspectKey(name_key.clone()),
                    basis: crate::facade::merge::IdentityBasisKind::DeclaredKeySet(
                        Arc::<[worth_foundational::facade::AspectKey]>::from([name_key]).into(),
                    ),
                },
            ]),
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "test.relation".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: RelationIntegrityDeclarations::default(),
            })
        })
        .expect("declared-key identity registry");
    RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
        .expect("valid native runtime configuration")
}

fn merge_request() -> MergeExecutionRequest {
    MergeExecutionRequest::new(
        BranchId("main".to_string()),
        BranchId("feature".to_string()),
        MergeIntent::ReconcileIntoTarget,
    )
}

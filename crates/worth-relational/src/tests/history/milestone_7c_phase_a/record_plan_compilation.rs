use std::sync::Arc;

use crate::facade::history::BranchId;
use crate::facade::identity::KindId;
use crate::facade::merge::{
    AspectMergePolicyDeclaration, AspectMergePolicyKind, MergeExecutionRequest, MergeIntent,
};
use crate::facade::runtime::RelationalRuntimeApi;
use crate::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationIntegrityDeclarations,
    RelationKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use crate::merge::data::{
    BoundExecutableMergeRecordPlan, ExecutableAspectPlan, MaterializedAspectValueEvidence,
    MergeValueMaterialization,
};
use crate::tests::support::{
    create_branch_from_main, create_entity, create_entity_outcome_on_branch, entity_field_aspect,
    field_key, persisted_runtime_with_test_schema, read_entity_field, update_entity,
    update_entity_on_branch, CascadeDeletePolicy, CrossContextPolicy,
};
use worth_foundational::facade::AspectKey;

#[test]
fn prepare_merge_execution_compiles_source_addition_record_plan() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    create_entity(&runtime, "root");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(&runtime, "feature-only", BranchId("feature".to_string()));

    let prepared = runtime
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            },
        )
        .expect("prepared merge");

    assert_eq!(prepared.bound_executable_plan().record_plans.len(), 1);
    match &prepared.bound_executable_plan().record_plans[0] {
        BoundExecutableMergeRecordPlan::AdoptSource(plan) => match &plan.source_visible_snapshot {
            crate::merge::data::VisibleMergeRecordSnapshot::Entity(entity) => {
                assert_eq!(
                    read_entity_field(entity, field_key("name")),
                    Some("feature-only".into())
                );
            }
            other => panic!("expected entity source snapshot, got {other:?}"),
        },
        other => panic!("expected adopt-source record plan, got {other:?}"),
    }
}

#[test]
fn prepare_merge_execution_compiles_exact_shared_record_plan() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                entity_field_aspect(
                    crate::tests::support::aspect_key("name"),
                    crate::tests::support::field_key("name"),
                ),
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
        .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build();
    let shared = create_entity(&runtime, "shared");
    create_branch_from_main(&runtime, "feature");
    update_entity(&runtime, shared, "same");
    update_entity_on_branch(&runtime, shared, "same", BranchId("feature".to_string()));

    let prepared = runtime
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            },
        )
        .expect("prepared merge");

    assert_eq!(prepared.bound_executable_plan().record_plans.len(), 1);
    match &prepared.bound_executable_plan().record_plans[0] {
        BoundExecutableMergeRecordPlan::PreserveShared(plan) => {
            assert!(!plan.equality_witness.witness_digest.is_empty());
            assert_eq!(
                plan.provenance.classification,
                crate::merge::data::MergeConflictClass::ExactSharedTruth
            );
            assert!(!plan.aspect_plan.is_empty());
            match &plan.aspect_plan[0] {
                ExecutableAspectPlan::PreserveSharedValue { shared_value, .. } => {
                    assert_eq!(
                        shared_value.policy,
                        MergeValueMaterialization::EqualityWitnessDigest
                    );
                    match &shared_value.evidence {
                        MaterializedAspectValueEvidence::EqualityWitnessDigest(digest) => {
                            assert!(!digest.is_empty());
                        }
                        other => panic!("expected equality witness evidence, got {other:?}"),
                    }
                }
                other => panic!("expected preserve-shared aspect plan, got {other:?}"),
            }
        }
        other => panic!("expected preserve-shared record plan, got {other:?}"),
    }
}

#[test]
fn prepare_merge_execution_compiles_reconcile_record_plan() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let name_key = AspectKey::new("name").unwrap();
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                entity_field_aspect(
                    crate::tests::support::aspect_key("name"),
                    crate::tests::support::field_key("name"),
                ),
            ])
            .with_identity_declarations(vec![crate::facade::merge::IdentityBasisDeclaration {
                scope: crate::facade::merge::IdentityBasisScope::AspectKey(name_key.clone()),
                basis: crate::facade::merge::IdentityBasisKind::DeclaredKeySet(Arc::from([
                    name_key.clone(),
                ])),
            }])
            .with_merge_policy_declarations(vec![AspectMergePolicyDeclaration {
                aspect_key: name_key.clone(),
                policy: AspectMergePolicyKind::PreferRicher,
            }]),
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
        .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build();
    create_entity(&runtime, "root");
    create_branch_from_main(&runtime, "feature");
    create_entity(&runtime, "shared-name");
    create_entity_outcome_on_branch(&runtime, "shared-name", BranchId("feature".to_string()));

    let prepared = runtime
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            },
        )
        .expect("prepared merge");

    assert_eq!(prepared.bound_executable_plan().record_plans.len(), 1);
    match &prepared.bound_executable_plan().record_plans[0] {
        BoundExecutableMergeRecordPlan::Reconcile(plan) => {
            assert_eq!(plan.identity_basis.source_record, plan.source_record);
            assert_eq!(plan.identity_basis.target_record, plan.target_record);
            assert!(!plan.aspect_plan.is_empty());
            match &plan.aspect_plan[0] {
                ExecutableAspectPlan::ReconcileValue {
                    source_value,
                    target_value,
                    base_value,
                    ..
                } => {
                    let source_value = source_value.as_ref().expect("source value");
                    let target_value = target_value.as_ref().expect("target value");
                    let base_value = base_value.as_ref().expect("base value");
                    assert_eq!(
                        source_value.policy,
                        MergeValueMaterialization::SnapshotPinnedRead
                    );
                    assert_eq!(
                        target_value.policy,
                        MergeValueMaterialization::SnapshotPinnedRead
                    );
                    assert_eq!(
                        base_value.policy,
                        MergeValueMaterialization::SnapshotPinnedRead
                    );
                    assert_pinned_visible_aspect_evidence(source_value);
                    assert_pinned_visible_aspect_evidence(base_value);
                }
                other => panic!("expected reconcile aspect plan, got {other:?}"),
            }
        }
        other => panic!("expected reconcile record plan, got {other:?}"),
    }
}

fn assert_pinned_visible_aspect_evidence(value: &crate::merge::data::MaterializedAspectValue) {
    match &value.evidence {
        MaterializedAspectValueEvidence::PinnedVisibleAspect { locator, .. } => {
            assert!(
                matches!(
                    locator,
                    worth_foundational::facade::AspectValueLocator::WholeAspect(aspect)
                        if aspect.authority()
                            == worth_foundational::facade::LocatorAuthority::Authoritative
                ),
                "expected authoritative whole-aspect locator, got {locator:?}"
            );
        }
        other => panic!("expected pinned visible aspect evidence, got {other:?}"),
    }
}

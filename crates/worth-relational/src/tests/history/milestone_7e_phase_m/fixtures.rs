use crate::facade::history::{BranchId, CommitId};
use crate::facade::identity::KindId;
use crate::facade::inspection::RelationalMergeSupportInspectionWitness;
use crate::facade::merge::{
    AspectMergePolicyDeclaration, AspectMergePolicyKind, IdentityBasisDeclaration,
    IdentityBasisKind, IdentityBasisScope, MergeExecutionOutcome, MergeExecutionRequest,
    MergeIntent, RelationalMergeProofPacketCanonicalBasis,
};
use crate::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use crate::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationIntegrityDeclarations,
    RelationKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use crate::merge::data::PreparedMergeExecution;
use crate::tests::support::{
    aspect_key, create_branch_from_main, create_entity, entity_field_aspect, field_key,
    string_aspect_field_patch, unique_test_store_path, CascadeDeletePolicy, CrossContextPolicy,
    DurabilityMode, DurableStoreLayout,
};
use crate::transactions::data::{MergeExecutionSummary, PublishedMergeExecutionAuthority};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CollaborationTruthSnapshot {
    pub authority: PublishedMergeExecutionAuthority,
    pub support: RelationalMergeSupportInspectionWitness,
    pub canonical_basis: RelationalMergeProofPacketCanonicalBasis,
}

pub(super) fn runtime_with_collaboration_merge_history() -> RelationalRuntime {
    let name_key = aspect_key("name");
    let status_key = aspect_key("status");
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                entity_field_aspect(name_key.clone(), field_key("name")),
                entity_field_aspect(status_key.clone(), field_key("status")),
            ])
            .with_identity_declarations(vec![IdentityBasisDeclaration {
                scope: IdentityBasisScope::AspectKey(name_key.clone()),
                basis: IdentityBasisKind::DeclaredKeySet(std::sync::Arc::from([name_key])),
            }])
            .with_merge_policy_declarations(vec![AspectMergePolicyDeclaration {
                aspect_key: status_key,
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
        .expect("phase 13 schema registry");
    RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .durability_mode(DurabilityMode::PersistedSegmentedLocalFs)
        .durable_store_layout(DurableStoreLayout {
            root_path: unique_test_store_path("worth-relational-7e-phase-m-certification"),
            segment_commit_capacity: 2,
        })
        .build()
}

pub(super) fn execute_feature_merge(runtime: &RelationalRuntime) -> MergeExecutionOutcome {
    let prepared = prepared_feature_merge(runtime);
    runtime
        .execute_prepared_merge(prepared)
        .expect("executed feature merge")
}

pub(super) fn prepared_feature_merge(runtime: &RelationalRuntime) -> PreparedMergeExecution {
    install_merge_scenario(runtime);
    runtime
        .merge()
        .prepare_merge_execution(MergeExecutionRequest::new(
            BranchId("main".to_string()),
            BranchId("feature".to_string()),
            MergeIntent::ReconcileIntoTarget,
        ))
        .expect("prepared feature merge")
}

pub(super) fn alternate_branch_basis(
    runtime: &RelationalRuntime,
) -> crate::history::data::RelationalMergeBranchBasis {
    runtime
        .history()
        .resolve_merge_branch_basis(&BranchId("main".to_string()), &BranchId("alt".to_string()))
        .expect("alternate branch basis")
}

pub(super) fn published_merge_authority(
    runtime: &RelationalRuntime,
    commit_id: CommitId,
) -> PublishedMergeExecutionAuthority {
    runtime
        .replay()
        .canonical_commit_envelope(commit_id)
        .and_then(|envelope| envelope.merge_execution_authority.clone())
        .expect("published merge authority")
}

pub(super) fn snapshot_from_summary(
    runtime: &RelationalRuntime,
    summary: &MergeExecutionSummary,
) -> CollaborationTruthSnapshot {
    let authority = PublishedMergeExecutionAuthority {
        execution_summary: summary.clone(),
        structural_summary: crate::transactions::data::MergeExecutionStructuralSummary {
            executed_record_count: summary.executed_record_count,
            adopted_source_record_count: summary.adopted_source_record_count,
            preserved_shared_record_count: summary.preserved_shared_record_count,
            reconciled_record_count: summary.reconciled_record_count,
            converged_deleted_on_both_sides_count: summary.converged_deleted_on_both_sides_count,
            deleted_on_both_sides_lineage_unchanged_count: summary
                .deleted_on_both_sides_lineage_unchanged_count,
            emitted_mutation_intent_count: summary.emitted_mutation_intent_count,
            emitted_entity_create_count: 0,
            emitted_relation_create_count: 0,
            emitted_entity_update_count: 0,
        },
    };
    snapshot_from_authority(runtime, authority)
}

pub(super) fn snapshot_from_authority(
    runtime: &RelationalRuntime,
    authority: PublishedMergeExecutionAuthority,
) -> CollaborationTruthSnapshot {
    let support = runtime
        .inspect_what_happened()
        .prepare_published_merge_support_inspection_witness(&authority)
        .expect("support inspection witness");
    let canonical_basis = lower_packet(runtime, authority.execution_summary.proof_packet());
    CollaborationTruthSnapshot {
        authority,
        support,
        canonical_basis,
    }
}

fn lower_packet(
    runtime: &RelationalRuntime,
    packet: &crate::facade::merge::RelationalMergeProofPacket,
) -> RelationalMergeProofPacketCanonicalBasis {
    let outcome = runtime
        .merge()
        .lower_merge_proof_packet_to_foundational_canonical_basis(packet);
    let worth_proof::TransitionOutcome::Success(basis) = outcome else {
        panic!("phase 13 canonical lowering must succeed for retained merge proof packet");
    };
    basis
}

fn install_merge_scenario(runtime: &RelationalRuntime) {
    if runtime
        .history()
        .branch_head(&BranchId("feature".to_string()))
        .is_some()
    {
        return;
    }
    let shared = create_entity(runtime, "shared");
    create_branch_from_main(runtime, "feature");
    create_branch_from_main(runtime, "alt");
    update_entity_status_on_branch(runtime, shared, "inactive", "main");
    update_entity_status_on_branch(runtime, shared, "active", "feature");
    update_entity_status_on_branch(runtime, shared, "pending", "alt");
}

fn update_entity_status_on_branch(
    runtime: &RelationalRuntime,
    entity_id: crate::facade::identity::EntityId,
    status: &str,
    branch: &str,
) {
    let mut txn = {
        let transaction_validation_input =
            crate::tests::support::test_owner_transaction_validation_input_for_branch(
                runtime,
                BranchId(branch.to_string()),
            );
        runtime
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    txn.push_batch(
        crate::facade::transactions::WorkerIntentBatch::new(format!("status-{branch}")).push(
            crate::facade::transactions::MutationIntent::Entity(
                crate::facade::transactions::EntityMutationIntent::UpdateFields(
                    crate::facade::transactions::UpdateEntityFieldsIntent {
                        entity_id,
                        fields: string_aspect_field_patch([(
                            aspect_key("status"),
                            field_key("status"),
                            status,
                        )]),
                    },
                ),
            ),
        ),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    txn.commit(
        runtime,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("update entity status on branch");
}

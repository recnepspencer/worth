use crate::facade::history::BranchId;
use crate::facade::identity::KindId;
use crate::facade::merge::{
    AspectMergePolicyDeclaration, AspectMergePolicyKind, AspectPolicyResolutionRecord,
    MergeExecutionAuthorityContract, MergeExecutionReadiness, MergePolicyDecisionBoundary,
    MergePolicyOwnershipSurface, MergePolicyProofBoundary, RelationalMergeAspectPolicyWitnessRow,
    RelationalMergeDeletionStrategyWitnessRow, RelationalMergeStrategyWitness,
    RelationalMergeTopologyStrategyWitnessRow, ResolvedAspectMergePolicy,
};
use crate::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use crate::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationIntegrityDeclarations,
    RelationKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use crate::tests::support::{
    aspect_key, entity_field_aspect, field_key, string_aspect_field_patch, unique_test_store_path,
    CascadeDeletePolicy, CrossContextPolicy, DurabilityMode, DurableStoreLayout,
};
use crate::transactions::data::PublishedMergeExecutionAuthority;
use std::sync::Arc;

pub(super) fn runtime_with_schema_declared_entity_policy(
    policy: AspectMergePolicyKind,
) -> RelationalRuntime {
    runtime_with_schema_declared_entity_policy_builder(policy, None)
}

pub(super) fn persisted_runtime_with_schema_declared_entity_policy(
    policy: AspectMergePolicyKind,
) -> RelationalRuntime {
    runtime_with_schema_declared_entity_policy_builder(
        policy,
        Some(unique_test_store_path(
            "worth-relational-7e-phase-i-strategy",
        )),
    )
}

pub(super) fn merge_request() -> crate::facade::merge::MergeExecutionRequest {
    crate::facade::merge::MergeExecutionRequest::new(
        BranchId("main".to_string()),
        BranchId("feature".to_string()),
        crate::facade::merge::MergeIntent::ReconcileIntoTarget,
    )
}

pub(super) fn update_entity_status_on_branch(
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
    .expect("update status");
}

pub(super) fn published_merge_authority(
    runtime: &RelationalRuntime,
    commit_id: crate::facade::history::CommitId,
) -> PublishedMergeExecutionAuthority {
    runtime
        .replay()
        .canonical_commit_envelope(commit_id)
        .and_then(|envelope| envelope.merge_execution_authority.clone())
        .expect("published merge authority")
}

pub(super) fn policy_row(policy: AspectMergePolicyKind) -> RelationalMergeAspectPolicyWitnessRow {
    RelationalMergeAspectPolicyWitnessRow::retained(
        crate::transactions::data::RecordRef::Entity(crate::identity::data::EntityId::new(
            crate::identity::data::PartitionId::main(),
            1,
            1,
        )),
        Some(crate::transactions::data::RecordRef::Entity(
            crate::identity::data::EntityId::new(crate::identity::data::PartitionId::main(), 1, 1),
        )),
        crate::facade::merge::MergeConflictClass::SchemaDeclaredCorrespondence,
        Arc::from([AspectPolicyResolutionRecord {
            aspect_key: aspect_key("status"),
            comparison: crate::facade::merge::AspectComparisonState::Divergent,
            applied_policy: Some(policy.clone()),
            decision_boundary: MergePolicyDecisionBoundary::AutoResolved,
            resolved_value_strategy: None,
        }]),
        Arc::from([ResolvedAspectMergePolicy {
            aspect_key: aspect_key("status"),
            policy,
        }]),
        MergePolicyProofBoundary {
            ownership_surface: MergePolicyOwnershipSurface::RuntimeOnly,
            decision_boundary: MergePolicyDecisionBoundary::AutoResolved,
        },
    )
}

pub(super) fn topology_row(
    topology_class: crate::facade::merge::TopologyExecutionClass,
    readiness: MergeExecutionReadiness,
) -> RelationalMergeTopologyStrategyWitnessRow {
    let blocked_reason = match readiness {
        MergeExecutionReadiness::Admitted => None,
        MergeExecutionReadiness::Blocked => Some(match topology_class {
            crate::facade::merge::TopologyExecutionClass::RelationEndpointStable => {
                crate::facade::merge::LoweredMergeBlockedReason::ManualConflictResolutionRequired
            }
            crate::facade::merge::TopologyExecutionClass::RelationEndpointRewiredLocal => {
                crate::facade::merge::LoweredMergeBlockedReason::RelationEndpointRewiredLocal
            }
            crate::facade::merge::TopologyExecutionClass::RelationEndpointRewiredEscalated => {
                crate::facade::merge::LoweredMergeBlockedReason::RelationEndpointRewiredEscalated
            }
            crate::facade::merge::TopologyExecutionClass::TopologyRegionConflict => {
                crate::facade::merge::LoweredMergeBlockedReason::TopologyRegionConflict
            }
        }),
        MergeExecutionReadiness::Rejected => None,
    };
    RelationalMergeTopologyStrategyWitnessRow::retained(
        crate::transactions::data::RecordRef::Relation(crate::identity::data::RelationId::new(
            crate::identity::data::PartitionId::main(),
            2,
            1,
        )),
        Some(crate::transactions::data::RecordRef::Relation(
            crate::identity::data::RelationId::new(
                crate::identity::data::PartitionId::main(),
                2,
                1,
            ),
        )),
        topology_class,
        readiness,
        blocked_reason,
    )
}

pub(super) fn deletion_row(
    deletion_class: crate::facade::merge::DeletionExecutionClass,
    readiness: MergeExecutionReadiness,
) -> RelationalMergeDeletionStrategyWitnessRow {
    let blocked_reason = match readiness {
        MergeExecutionReadiness::Admitted => None,
        MergeExecutionReadiness::Blocked => Some(match deletion_class {
            crate::facade::merge::DeletionExecutionClass::SourceDeletedTargetLive => {
                crate::facade::merge::LoweredMergeBlockedReason::SourceDeletedTargetLive
            }
            crate::facade::merge::DeletionExecutionClass::SourceLiveTargetDeleted => {
                crate::facade::merge::LoweredMergeBlockedReason::SourceLiveTargetDeleted
            }
            crate::facade::merge::DeletionExecutionClass::DeletedOnBothSides => {
                crate::facade::merge::LoweredMergeBlockedReason::DeletedOnBothSides
            }
            crate::facade::merge::DeletionExecutionClass::DeletedVsModified => {
                crate::facade::merge::LoweredMergeBlockedReason::DeletedVsModified
            }
            crate::facade::merge::DeletionExecutionClass::DeletedVsRewired => {
                crate::facade::merge::LoweredMergeBlockedReason::DeletedVsRewired
            }
        }),
        MergeExecutionReadiness::Rejected => None,
    };
    RelationalMergeDeletionStrategyWitnessRow::retained(
        crate::transactions::data::RecordRef::Entity(crate::identity::data::EntityId::new(
            crate::identity::data::PartitionId::main(),
            3,
            1,
        )),
        Some(crate::transactions::data::RecordRef::Entity(
            crate::identity::data::EntityId::new(crate::identity::data::PartitionId::main(), 3, 1),
        )),
        deletion_class,
        readiness,
        blocked_reason,
    )
}

pub(super) fn strategy_witness(
    aspect_policy_rows: Vec<RelationalMergeAspectPolicyWitnessRow>,
    topology_rows: Vec<RelationalMergeTopologyStrategyWitnessRow>,
    deletion_rows: Vec<RelationalMergeDeletionStrategyWitnessRow>,
    execution_authority_contract: MergeExecutionAuthorityContract,
) -> RelationalMergeStrategyWitness {
    RelationalMergeStrategyWitness::retained(
        "abababababababababababababababababababababababababababababababab".to_string(),
        "cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd".to_string(),
        execution_authority_contract,
        Arc::from(aspect_policy_rows),
        Arc::from(topology_rows),
        Arc::from(deletion_rows),
    )
}

fn runtime_with_schema_declared_entity_policy_builder(
    policy: AspectMergePolicyKind,
    root_path: Option<std::path::PathBuf>,
) -> RelationalRuntime {
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
            .with_merge_policy_declarations(vec![AspectMergePolicyDeclaration {
                aspect_key: status_key,
                policy,
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
        .expect("strategy witness registry");
    let mut builder = RelationalRuntimeApi::builder().schema_registry(registry);
    if let Some(root_path) = root_path {
        builder = builder
            .durability_mode(DurabilityMode::PersistedSegmentedLocalFs)
            .durable_store_layout(DurableStoreLayout {
                root_path,
                segment_commit_capacity: 2,
            });
    }
    builder.build()
}

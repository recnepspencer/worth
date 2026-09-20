use super::*;
use crate::facade::transactions::{CreatedEntityRef, EntityReference, EntitySpec, RelationSpec};
use std::sync::Arc;

use crate::validation::data::{
    CustomInvariantDescriptor, CustomInvariantExecutionContext, CustomInvariantExecutionError,
    CustomInvariantOperationalMetadata, CustomInvariantPreparationError,
    CustomInvariantRegistration, CustomInvariantRule, CustomInvariantRuleId,
    CustomInvariantScopePlanner, CustomInvariantSemanticIdentity, CustomInvariantSemanticVersion,
    CustomInvariantVerdict, InvariantCostClass, InvariantExecutionPoint, InvariantFailureEffect,
    InvariantGroup, InvariantGroupSet,
};

struct NativeCheckpointRule;

impl CustomInvariantRule for NativeCheckpointRule {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("native.checkpoint.rule"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Native Checkpoint Rule"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: std::num::NonZeroU64::new(u64::MAX).unwrap(),
                access: crate::validation::data::CustomInvariantAccessContract::default(),
                execution_point: InvariantExecutionPoint::CommitBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance),
                cost_class: InvariantCostClass::Touched,
                failure_effect: InvariantFailureEffect::BlockCommit,
            },
        }
    }

    fn prepare_scope(
        &self,
        _: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        Ok(())
    }

    fn evaluate(
        &self,
        _: &CustomInvariantExecutionContext<'_>,
        _: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        Ok(CustomInvariantVerdict::Pass)
    }
}

#[test]
fn native_checkpoint_round_trip_restores_a_live_editable_world() {
    let runtime = persisted_runtime_with_test_schema();
    let committed = create_entity_outcome(&runtime, "native-checkpoint");
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint()
        .expect("the canonical committed world encodes as one opaque checkpoint");

    let mut recovered = persisted_runtime_with_test_schema();
    let pre_recovery_identity = recovered.runtime_instance_id();
    let outcome = recovered
        .durability_recovery()
        .restore_native_checkpoint(&checkpoint)
        .expect("the opaque native checkpoint restores through durability authority");

    assert_ne!(recovered.runtime_instance_id(), pre_recovery_identity);
    assert_eq!(outcome.latest_commit, Some(committed.commit.clone()));
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("main".to_owned())),
        Some(committed.commit.clone())
    );
    let later = create_entity_outcome(&recovered, "post-restore-edit");
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("main".to_owned())),
        Some(later.commit.clone())
    );
}

#[test]
fn native_checkpoint_round_trip_restores_current_relation_adjacency() {
    let runtime = persisted_runtime_with_test_schema();
    let source_key = crate::symbols::data::ClientKey::raw("native-relation-source");
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("native-relation")
                .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: source_key.clone(),
                    fields: single_string_aspect_field_patch(
                        aspect_key("name"),
                        field_key("name"),
                        "native-relation-source",
                    ),
                })))
                .push(MutationIntent::Create(CreateIntent::Relation(
                    RelationSpec {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(2),
                        client_key: crate::symbols::data::ClientKey::raw("native-relation"),
                        source: EntityReference::Created(CreatedEntityRef {
                            partition_id: PartitionId::main(),
                            kind_id: KindId(1),
                            client_key: source_key.clone(),
                        }),
                        target: EntityReference::Created(CreatedEntityRef {
                            partition_id: PartitionId::main(),
                            kind_id: KindId(1),
                            client_key: source_key,
                        }),
                        fields: Default::default(),
                    },
                ))),
        )
        .unwrap();
    let created = transaction.commit(&runtime).unwrap();
    let source = changed_entities(&created)[0];
    let target = source;
    let relation = changed_relations(&created)[0];
    let checkpoint = runtime.durability_authority().native_checkpoint().unwrap();

    let mut recovered = persisted_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .restore_native_checkpoint(&checkpoint)
        .unwrap();
    let rows = recovered
        .read_truth()
        .bounded_outgoing_relations_of_kind_at_version(
            source,
            crate::facade::identity::KindId(2),
            recovered.current_version_id(),
            4,
        )
        .unwrap()
        .into_records();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].relation_id, relation);
    assert_eq!(rows[0].target, target);
    let snapshot = snapshot_for_owner_branch(&recovered, &BranchId("main".to_owned()));
    let projected = recovered
        .read_truth()
        .project_snapshot(&snapshot)
        .unwrap()
        .bounded_outgoing_relations_for_frontier(
            &std::collections::BTreeSet::from([source]),
            KindId(2),
            4,
        )
        .unwrap()
        .into_records();
    assert_eq!(projected.len(), 1);
    assert_eq!(projected[0].relation_id, relation);
}

#[test]
fn repeated_native_captures_do_not_retain_checkpoint_images() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, "native-checkpoint-first");
    let retained_before = runtime.durability.checkpoints().len();

    for label in ["second", "third", "fourth"] {
        runtime
            .durability_authority()
            .native_checkpoint()
            .expect("each settled world encodes without retaining an internal checkpoint");
        create_entity_outcome(&runtime, label);
    }
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint()
        .expect("the latest settled world encodes");

    assert_eq!(runtime.durability.checkpoints().len(), retained_before);
    let mut recovered = persisted_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .restore_native_checkpoint(&checkpoint)
        .expect("the final non-retained checkpoint restores");
    create_entity_outcome(&recovered, "post-bounded-retention-edit");
}

#[test]
fn corrupt_native_checkpoint_is_denied_without_publishing_recovered_state() {
    let corrupt = crate::facade::durability::RelationalNativeCheckpoint::from_untrusted_bytes(
        vec![0xc1, 0x00, 0xff].into_boxed_slice(),
    );
    let mut recovered = persisted_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .restore_native_checkpoint(&corrupt)
        .expect_err("corrupt native bytes cannot become runtime authority");

    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}

#[test]
fn native_checkpoint_rejects_a_foreign_runtime_name_without_publishing_state() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, "native-checkpoint-runtime-name");
    let checkpoint = runtime.durability_authority().native_checkpoint().unwrap();
    let mut recovered = RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .runtime_name("foreign-runtime")
        .durability_mode(DurabilityMode::PersistedSegmentedLocalFs)
        .durable_store_layout(DurableStoreLayout {
            root_path: unique_test_store_path("foreign-native-checkpoint"),
            segment_commit_capacity: 2,
        })
        .build();

    let error = recovered
        .durability_recovery()
        .restore_native_checkpoint(&checkpoint)
        .expect_err("a foreign runtime checkpoint cannot be admitted");

    assert_eq!(error.class, RecoveryFailureClass::RuntimeNameMismatch);
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}

#[test]
fn native_checkpoint_preserves_unsealed_builder_invariant_authority() {
    let source = persisted_runtime_with_builder_invariant();
    create_entity_outcome(&source, "native-checkpoint-builder-invariant");
    let checkpoint = source.durability_authority().native_checkpoint().unwrap();
    let mut recovered = persisted_runtime_with_builder_invariant();

    recovered
        .durability_recovery()
        .restore_native_checkpoint(&checkpoint)
        .expect("native recovery preserves target runtime extensions");

    assert_eq!(
        recovered
            .schema_contract_runtime
            .custom_invariant_registries
            .iter()
            .next()
            .unwrap()
            .rule_id()
            .as_str(),
        "native.checkpoint.rule"
    );
    assert!(
        !recovered
            .schema_contract_runtime
            .initial_custom_invariants_sealed
    );
}

fn persisted_runtime_with_builder_invariant() -> crate::runtime::RelationalRuntime {
    RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .custom_invariant(CustomInvariantRegistration::new(NativeCheckpointRule).unwrap())
        .durability_mode(DurabilityMode::PersistedSegmentedLocalFs)
        .durable_store_layout(DurableStoreLayout {
            root_path: unique_test_store_path("native-checkpoint-invariant"),
            segment_commit_capacity: 2,
        })
        .build()
}

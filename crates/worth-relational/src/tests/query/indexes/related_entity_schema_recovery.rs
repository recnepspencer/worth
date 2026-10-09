use super::*;
use crate::facade::durability::{DurabilityMode, DurableStoreLayout, RecoveryVerificationMode};
use crate::facade::indexes::{
    BoundedIndexParityMode, BoundedRelatedEntityOrderedLookupRequest, RelatedEntityEndpoint,
    RelatedEntityOrderingDirection, RelatedEntityOrderingField,
};
use crate::facade::schema::{
    HistoricalInterpretationSensitivity, ProposedSchemaTransition, SchemaDiffAtom,
    SchemaDiffDetail, SchemaElementKind, SchemaElementRef, SchemaId, SchemaPublicationImpact,
    SchemaReconciliationPolicy, SchemaStratum, SchemaSubscriberImpact, SchemaVersionId,
};

#[test]
fn recovered_related_ordering_uses_each_exact_roots_schema_contract() {
    let (runtime, fixture) = v1_ordering_runtime();
    let recovered = transition_checkpoint_and_recover(runtime, fixture.index_id);
    let RelatedOrderingFixture {
        parent,
        alpha,
        beta,
        index_id,
    } = fixture;

    let legacy = recovered
        .branch_identity(&BranchId("v1-ordering".to_owned()))
        .unwrap();
    let (_, legacy_basis) = recovered.observe_branch(&legacy).unwrap();
    let legacy_snapshot = recovered
        .snapshots()
        .snapshot_for_observation(&legacy_basis.observation())
        .unwrap();
    assert_eq!(
        recovered
            .read_truth()
            .snapshot_schema_version(&legacy_snapshot),
        Some(SchemaVersionId(1))
    );
    for parity in [
        BoundedIndexParityMode::Production,
        BoundedIndexParityMode::Certification,
    ] {
        let old = recovered
            .index_access()
            .execute_bounded_related_entity_ordered_lookup(
                related_request(legacy_snapshot.clone(), index_id, parent),
                parity,
            )
            .unwrap();
        assert_eq!(old.child_entity_ids(), &[alpha, beta]);
    }

    let current_snapshot = recovered.visibility_authority().snapshot();
    assert_eq!(
        recovered
            .read_truth()
            .snapshot_schema_version(&current_snapshot),
        Some(SchemaVersionId(2))
    );
    let current = recovered
        .index_access()
        .execute_bounded_related_entity_ordered_lookup(
            related_request(current_snapshot, index_id, parent),
            BoundedIndexParityMode::Certification,
        )
        .unwrap();
    assert!(current.child_entity_ids().is_empty());
}

struct RelatedOrderingFixture {
    parent: crate::facade::identity::EntityId,
    alpha: crate::facade::identity::EntityId,
    beta: crate::facade::identity::EntityId,
    index_id: DerivedIndexId,
}

fn v1_ordering_runtime() -> (RelationalRuntime, RelatedOrderingFixture) {
    let v1_registry = AspectSchemaFixture::with_default_declared_aspects(
        CascadeDeletePolicy::CascadeDeleteRelations,
    )
    .build_registry();
    let store_layout = DurableStoreLayout {
        root_path: unique_test_store_path("related-ordering-schema-carrier"),
        segment_commit_capacity: 2,
    };
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(v1_registry)
        .durability_mode(DurabilityMode::PersistedSegmentedLocalFs)
        .durable_store_layout(store_layout)
        .build();
    let parent = changed_entities(&create_entity_outcome(&runtime, "parent"))[0];
    let alpha = changed_entities(&create_entity_outcome(&runtime, "alpha"))[0];
    let beta = changed_entities(&create_entity_outcome(&runtime, "beta"))[0];
    create_relation_outcome(&runtime, parent, beta, "owns-beta");
    let v1_commit = create_relation_outcome(&runtime, parent, alpha, "owns-alpha");
    create_branch_from_main(&runtime, "v1-ordering");
    let index = register_related_ordering_index(&runtime);
    let v1_branch = runtime
        .branch_identity(&BranchId("v1-ordering".to_owned()))
        .unwrap();
    let (_, v1_basis) = runtime.observe_branch(&v1_branch).unwrap();
    let v1_build = runtime.index_authority().build_for_basis(
        DerivedIndexBuildRequest {
            source_commit_id: v1_commit.commit.commit_id,
            branch_id: BranchId("v1-ordering".to_owned()),
            index_ids: vec![index.index_id],
        },
        &v1_basis,
    );
    assert!(v1_build.failed_indexes.is_empty());

    (
        runtime,
        RelatedOrderingFixture {
            parent,
            alpha,
            beta,
            index_id: index.index_id,
        },
    )
}

fn transition_checkpoint_and_recover(
    mut runtime: RelationalRuntime,
    index_id: DerivedIndexId,
) -> RelationalRuntime {
    let v2_registry = AspectSchemaFixture {
        schema_version_id: SchemaVersionId(2),
        entity_aspects: vec![
            entity_field_aspect(aspect_key("display"), field_key("display")),
            lifecycle_aspect(),
        ],
        ..AspectSchemaFixture::with_default_declared_aspects(
            CascadeDeletePolicy::CascadeDeleteRelations,
        )
    }
    .build_registry();
    runtime.set_schema_registry_for_test(v2_registry.clone());
    let mut transaction = {
        let transaction_validation_input =
            test_owner_transaction_validation_input_for_main(&runtime).with_schema_transition(
                schema_v2_transition(),
                Some(SchemaReconciliationPolicy::PreserveInformation),
            );
        runtime
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    transaction
        .push_batch(
            batch_create("v2-current"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let v2_commit = transaction
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("v2 schema transition commits");
    let v2_build = runtime
        .index_authority()
        .build_for_commit(DerivedIndexBuildRequest {
            source_commit_id: v2_commit.commit.commit_id,
            branch_id: BranchId("main".to_owned()),
            index_ids: vec![index_id],
        });
    assert!(v2_build.failed_indexes.is_empty());

    runtime
        .durability_authority()
        .checkpoint()
        .expect("checkpoint preserves both schema-qualified generations");
    let plan = runtime
        .durability()
        .recovery_plan(RecoveryVerificationMode::NormalRecoveryVerification);
    let mut recovered = RelationalRuntimeApi::builder()
        .schema_registry(v2_registry)
        .build();
    recovered
        .durability_recovery()
        .recover(plan)
        .expect("both exact root schema carriers recover");
    recovered
}

fn register_related_ordering_index(runtime: &RelationalRuntime) -> DerivedIndexDefinition {
    runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "relation.owns.schema-qualified-child-name".to_owned(),
        kind: DerivedIndexKind::RelatedEntityOrdering {
            relation_kind: KindId(2),
            parent_endpoint: RelatedEntityEndpoint::SourceParent,
            child_kind: KindId(1),
            ordering: vec![RelatedEntityOrderingField::new(
                aspect_field_locator(aspect_key("name"), field_key("name")),
                RelatedEntityOrderingDirection::Ascending,
            )],
        },
        branch_scoped: true,
    })
}

fn related_request(
    snapshot: crate::facade::snapshots::SnapshotHandle,
    index_id: DerivedIndexId,
    parent: crate::facade::identity::EntityId,
) -> BoundedRelatedEntityOrderedLookupRequest {
    BoundedRelatedEntityOrderedLookupRequest::new(snapshot, index_id, parent, KindId(1), None, 2)
        .unwrap()
}

fn schema_v2_transition() -> ProposedSchemaTransition {
    ProposedSchemaTransition {
        source_schema_id: SchemaId("test".to_owned()),
        source_schema_version_id: SchemaVersionId(1),
        target_schema_id: SchemaId("test".to_owned()),
        target_schema_version_id: SchemaVersionId(2),
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::Field,
                SchemaId("test".to_owned()),
                SchemaVersionId(2),
                Some(KindId(1)),
                "display",
            ),
            vec![
                SchemaStratum::StructuralShape,
                SchemaStratum::PublicationContract,
            ],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            SchemaSubscriberImpact::ConsumableSurfaceChanged,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::AddedField {
                field: field_key("display"),
                required: false,
                default_expression: Some("null".into()),
            },
        )
        .with_boundary_visibility_proof(
            crate::schema::data::SubscriberBoundaryVisibility::VisibleSemanticallyIgnorable,
        )],
    }
}

use super::*;
use crate::facade::indexes::{
    DerivedIndexEntries, DerivedIndexMaintenanceBudget, RelatedEntityEndpoint,
    RelatedEntityOrderingDirection, RelatedEntityOrderingField, RelationJoinDefinition,
    RelationJoinLeg, RelationJoinSharedEndpoint,
};
use crate::facade::transactions::{EntityReference, EntitySpec};
use worth_execution::ExecutionAllocationPolicy as Allocation;

#[test]
fn kind_routing_skips_unrelated_records_and_preserves_endpoint_retirement() {
    let fixture = AspectSchemaFixture::with_default_declared_aspects(
        CascadeDeletePolicy::CascadeDeleteRelations,
    );
    let registry = fixture
        .build_registry()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(3),
            kind_name: "test.external".into(),
            schema_id: fixture.schema_id.clone(),
            schema_version_id: fixture.schema_version_id,
            aspect_contract_declarations: KindAspectContractDeclarations::new(
                fixture.entity_aspects.clone(),
            ),
        })
        .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build();
    let shared = create_entity(&runtime, "shared");
    let unrelated = create_entity(&runtime, "unrelated");
    let external = |name: &str| {
        let mut transaction = test_owner_begin_transaction_for_main(&runtime);
        transaction
            .push_batch(
                WorkerIntentBatch::new(name).push(MutationIntent::Create(CreateIntent::Entity(
                    EntitySpec {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(3),
                        client_key: crate::symbols::data::ClientKey::raw(name),
                        fields: name_field_patch(name),
                    },
                ))),
                Allocation::SystemAllocation,
            )
            .unwrap();
        let committed = transaction
            .commit(&runtime, Allocation::SystemAllocation)
            .unwrap();
        let id = changed_entities(&committed)[0];
        release_test_commit_snapshot(&runtime, &committed);
        id
    };
    let left = external("left");
    let right = external("right");
    let replacement = external("replacement");
    create_relation(&runtime, left, shared, "left-shared");
    let initial = create_relation_outcome(&runtime, shared, right, "shared-right");
    let moved_relation = changed_relations(&initial)[0];
    let definitions = [
        DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "routing.ordering".into(),
            branch_scoped: true,
            kind: DerivedIndexKind::RelatedEntityOrdering {
                relation_kind: KindId(2),
                parent_endpoint: RelatedEntityEndpoint::SourceParent,
                child_kind: KindId(3),
                ordering: vec![RelatedEntityOrderingField::new(
                    aspect_field_locator(aspect_key("name"), field_key("name")),
                    RelatedEntityOrderingDirection::Ascending,
                )],
            },
        },
        DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "routing.join".into(),
            branch_scoped: true,
            kind: DerivedIndexKind::RelationJoin(RelationJoinDefinition::new(
                RelationJoinLeg::new(KindId(2), RelationJoinSharedEndpoint::Target, KindId(3)),
                RelationJoinLeg::new(KindId(2), RelationJoinSharedEndpoint::Source, KindId(3)),
                KindId(1),
            )),
        },
    ];
    let ids = definitions
        .into_iter()
        .map(|d| runtime.index_authority().register(d).index_id)
        .collect::<Vec<_>>();
    let request = |commit_id| DerivedIndexBuildRequest {
        source_commit_id: commit_id,
        branch_id: BranchId("main".into()),
        index_ids: ids.clone(),
    };
    let built = runtime
        .index_authority()
        .build_for_commit(request(initial.commit.commit_id));
    assert!(built.failed_indexes.is_empty());
    let retained = built.generations.clone();
    assert_selected_rows(&retained, shared, left, right, moved_relation);
    let edited = update_entity(&runtime, unrelated, "unrelated-edited");
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let local = runtime
        .index_authority()
        .refresh_for_basis(
            request(edited.commit.commit_id),
            &basis,
            Some(&initial.snapshot),
            budget(),
        )
        .unwrap();
    // Both indexes have populated relevant rows, but the changed entity has
    // neither the ordering child kind nor a changed join membership kind.
    assert_eq!(local.work.record_reads, 2);
    assert_eq!(local.work.entry_edits, 0);
    assert_eq!(local.work.adjacency_work_units, 0);
    assert_rebuild(
        &runtime,
        &basis,
        request(edited.commit.commit_id),
        &local.generations,
    );

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("rewire-and-retire")
                .push(MutationIntent::Relation(
                    RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                        relation_id: moved_relation,
                        kind_id: KindId(2),
                        source: EntityReference::Existing(shared),
                        target: EntityReference::Existing(replacement),
                    }),
                ))
                .push(MutationIntent::Entity(EntityMutationIntent::Delete(
                    DeleteEntityIntent { entity_id: right },
                ))),
            Allocation::SystemAllocation,
        )
        .unwrap();
    let rewired = transaction
        .commit(&runtime, Allocation::SystemAllocation)
        .unwrap();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let local = runtime
        .index_authority()
        .refresh_for_basis(
            request(rewired.commit.commit_id),
            &basis,
            Some(&edited.snapshot),
            budget(),
        )
        .unwrap();
    assert!(local.work.entry_edits > 0);
    assert_selected_rows(
        &local.generations,
        shared,
        left,
        replacement,
        moved_relation,
    );
    assert_rebuild(
        &runtime,
        &basis,
        request(rewired.commit.commit_id),
        &local.generations,
    );
    assert_ne!(retained[0].entries, local.generations[0].entries);
    assert_ne!(retained[1].entries, local.generations[1].entries);
    let old = runtime
        .index_authority()
        .build_for_commit(request(initial.commit.commit_id));
    assert!(old.failed_indexes.is_empty());
    for (original, rebuilt) in retained.iter().zip(&old.generations) {
        assert_eq!(original.entries, rebuilt.entries);
    }

    // Replace changes kind by retiring the old full ID and creating a new ID.
    // Routing must preserve both changes rather than infer kind from its slot.
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("replace-indexed-child-kind").push(MutationIntent::Entity(
                EntityMutationIntent::Replace(ReplaceEntityIntent {
                    entity_id: replacement,
                    replacement: EntitySpec {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(1),
                        client_key: crate::symbols::data::ClientKey::raw("replacement-kind-one"),
                        fields: name_field_patch("replacement-kind-one"),
                    },
                }),
            )),
            Allocation::SystemAllocation,
        )
        .unwrap();
    let replaced = transaction
        .commit(&runtime, Allocation::SystemAllocation)
        .unwrap();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let local = runtime
        .index_authority()
        .refresh_for_basis(
            request(replaced.commit.commit_id),
            &basis,
            Some(&rewired.snapshot),
            budget(),
        )
        .unwrap();
    assert_rebuild(
        &runtime,
        &basis,
        request(replaced.commit.commit_id),
        &local.generations,
    );
    for generation in &local.generations {
        match &generation.entries {
            DerivedIndexEntries::RelatedEntityOrdering(entries) => assert!(entries.is_empty()),
            DerivedIndexEntries::RelationJoin(entries) => assert!(entries.is_empty()),
            _ => panic!("ordering or join"),
        }
    }
    release_test_commit_snapshot(&runtime, &initial);
    release_test_commit_snapshot(&runtime, &edited);
    release_test_commit_snapshot(&runtime, &rewired);
    release_test_commit_snapshot(&runtime, &replaced);
}

fn assert_selected_rows(
    generations: &[crate::indexes::data::DerivedIndexGeneration],
    shared: crate::identity::data::EntityId,
    left: crate::identity::data::EntityId,
    right: crate::identity::data::EntityId,
    relation: crate::identity::data::RelationId,
) {
    let DerivedIndexEntries::RelatedEntityOrdering(ordering) = &generations[0].entries else {
        panic!("ordering")
    };
    assert_eq!(ordering.len(), 1);
    let rows = ordering.get(&shared).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows.get(0).unwrap().child_entity_id(), right);
    assert_eq!(rows.get(0).unwrap().relation_id(), relation);
    let DerivedIndexEntries::RelationJoin(join) = &generations[1].entries else {
        panic!("join")
    };
    assert_eq!(join.len(), 1);
    let key = crate::indexes::data::RelationJoinKey::new(left, right);
    let rows = join.get(&key).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows.get(0).unwrap().shared_entity_id(), shared);
}

fn budget() -> DerivedIndexMaintenanceBudget {
    DerivedIndexMaintenanceBudget {
        maximum_work_units: 100_000,
        maximum_cold_record_slots: 0,
        maximum_derived_rows: 100_000,
    }
}

fn assert_rebuild(
    runtime: &RelationalRuntime,
    basis: &crate::branch::AdmittedRelationalBranchBasis,
    request: DerivedIndexBuildRequest,
    generations: &[crate::indexes::data::DerivedIndexGeneration],
) {
    let rebuilt = runtime.index_authority().build_for_basis(request, basis);
    assert!(rebuilt.failed_indexes.is_empty());
    assert_eq!(generations.len(), rebuilt.generations.len());
    for (local, rebuilt) in generations.iter().zip(&rebuilt.generations) {
        assert_eq!(local.index_id, rebuilt.index_id);
        assert_eq!(local.entries, rebuilt.entries);
    }
}

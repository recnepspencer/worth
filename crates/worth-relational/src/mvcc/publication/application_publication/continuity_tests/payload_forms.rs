//! Real installed publication of generic Query-shaped string struct outputs.
use super::*;
use worth_foundational::facade::{
    ContractValidatedAspectValueView, PortableAspectContractBasis, PortableAspectPatchOperation,
    PortableRecordAspectPatch,
};
#[path = "payload_forms/refusal.rs"]
mod refusal;
#[path = "payload_forms/schema.rs"]
mod schema;

fn succession(
    owner: &worth_proof::InstalledApplicationPublicationOwner,
    previous: RelationalRequiredOutputPublication,
    members: &[(EntityId, KindId)],
    dependency: u8,
) -> RequiredOutputPublication {
    let admission = owner
        .admit_required_output_succession(
            owner.admit_required_output("generic-preserver", [dependency; 32]),
            previous.authority(),
        )
        .unwrap();
    let meaning = previous
        .envelope
        .required_output_origin
        .as_ref()
        .unwrap()
        .meaning
        .clone();
    RequiredOutputPublication::Preserve {
        admission,
        predecessor: previous,
        meaning,
        roles: crate::mvcc::native_roles_fixture(members, RequiredOutputRolePosture::Preserve),
    }
}

#[test]
fn query_shaped_struct_birth_optional_clear_and_fieldless_incidence_keep_warm_and_cold_custody() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let mut runtime = schema::runtime();
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(crate::schema::data::RelationalSchemaRegistry::new())
        .unwrap();
    let (owner, installed) = receipt.issue_application_publication_owner();
    let targets = [
        ("container", 3),
        ("empty-a", 1),
        ("empty-b", 1),
        ("single", 6),
        ("nominal-present", 5),
        ("nominal-absent", 5),
    ]
    .map(|(key, kind)| CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(kind),
        client_key: crate::symbols::data::ClientKey::raw(key),
    });
    let fields = [
        string_aspect_field_patch([
            (aspect_key("identity"), field_key("key"), "generic identity"),
            (aspect_key("facts"), field_key("a"), "a"),
            (aspect_key("facts"), field_key("b"), "b"),
            (aspect_key("facts"), field_key("c"), "c"),
            (aspect_key("facts"), field_key("note"), "clear later"),
        ]),
        AspectFieldPatch::default(),
        AspectFieldPatch::default(),
        string_aspect_field_patch([(aspect_key("value"), field_key("value"), "single")]),
        string_aspect_field_patch([
            (aspect_key("geometry"), field_key("value"), "geometry"),
            (aspect_key("nominal"), field_key("value"), "nominal"),
        ]),
        string_aspect_field_patch([(aspect_key("geometry"), field_key("value"), "geometry")]),
    ];
    let mut batch = WorkerIntentBatch::new("generic-query-shaped-birth");
    for (target, fields) in targets.iter().zip(fields) {
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: target.partition_id,
            kind_id: target.kind_id,
            client_key: target.client_key.clone(),
            fields,
        })));
    }
    let edges = [(1, 2), (0, 1)].map(|(from, to)| CreatedRelationRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(2),
        client_key: crate::symbols::data::ClientKey::raw(format!("incidence-{from}-{to}")),
        source: EntityReference::Created(targets[from].clone()),
        target: EntityReference::Created(targets[to].clone()),
    });
    for edge in &edges {
        batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: edge.partition_id,
                kind_id: edge.kind_id,
                client_key: edge.client_key.clone(),
                source: edge.source.clone(),
                target: edge.target.clone(),
                fields: AspectFieldPatch::default(),
            },
        )));
    }
    let birth = publish(
        &runtime,
        execution,
        &installed,
        batch,
        native_birth_fixture(
            owner.admit_required_output("generic-creator", [1; 32]),
            targets.to_vec(),
        ),
    )
    .unwrap();
    let members = targets
        .iter()
        .map(|target| (birth.created_entity(target).unwrap(), target.kind_id))
        .collect::<Vec<_>>();
    let relations = edges
        .iter()
        .map(|edge| birth.created_relation(edge).unwrap())
        .collect::<Vec<_>>();
    let born_at = birth.commit.commit_id;
    let envelope = &birth.publication().envelope;
    assert_eq!(envelope.record_allocations.len(), 8);
    assert_eq!(envelope.patch.authoritative_record_patches.len(), 8);
    for patch in &envelope.patch.authoritative_record_patches {
        assert_eq!(
            patch.structural_change,
            crate::publication::patch::data::RecordStructuralChange::Created
        );
        assert!(!patch.contains_opaque_aspect);
        for operation in patch.authoritative_patch.full_grammar_operations() {
            assert!(matches!(operation, crate::publication::patch::data::PublishedAuthoritativePatchOperation::WholeAspectSet {
                binding: worth_foundational::facade::AspectBinding::EntityField { .. },
                value: crate::publication::patch::data::PublishedAuthoritativePatchValue::Struct(_), ..
            }));
        }
    }
    for ((id, _), count) in members.iter().zip([2, 0, 0, 1, 2, 1]) {
        let patch = envelope
            .patch
            .authoritative_record_patches
            .iter()
            .find(|patch| patch.target == RecordRef::Entity(*id))
            .unwrap();
        assert_eq!(
            patch.authoritative_patch.full_grammar_operation_count(),
            count
        );
    }
    let previous = birth.required_output_publication(&installed).unwrap();
    assert_eq!(
        previous
            .envelope
            .required_output_origin
            .as_ref()
            .unwrap()
            .relations
            .len(),
        2
    );
    release_test_commit_snapshot(&runtime, &birth);
    let no_data = publish(
        &runtime,
        execution,
        &installed,
        WorkerIntentBatch::new("actual-no-data-preserve"),
        succession(&owner, previous, &members, 2),
    )
    .unwrap();
    assert!(no_data.changed_records.is_empty());
    assert_ne!(no_data.commit.commit_id, born_at);
    let previous = no_data.required_output_publication(&installed).unwrap();
    let origin = previous.envelope.required_output_origin.as_ref().unwrap();
    assert_eq!(origin.birth, born_at);
    assert_eq!(origin.members.len(), 6);
    assert_eq!(origin.relations.len(), 2);
    release_test_commit_snapshot(&runtime, &no_data);
    let clear = publish(
        &runtime,
        execution,
        &installed,
        WorkerIntentBatch::new("later-optional-field-clear").push(MutationIntent::Entity(
            EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                entity_id: members[0].0,
                aspect_patch: PortableRecordAspectPatch::new([
                    PortableAspectPatchOperation::PatchFields {
                        basis: PortableAspectContractBasis::from_contract(
                            &schema::facts().contract,
                        ),
                        selected_fields: vec![field_key("note")],
                        field_sets: vec![],
                        field_clears: vec![field_key("note")],
                    },
                ]),
            }),
        )),
        succession(&owner, previous, &members, 3),
    )
    .unwrap();
    assert_eq!(clear.changed_records, [RecordRef::Entity(members[0].0)]);
    assert!(
        matches!(&clear.publication().envelope.patch.authoritative_record_patches[0]
        .authoritative_patch.full_grammar_operations()[0],
        crate::publication::patch::data::PublishedAuthoritativePatchOperation::FieldLevelPatch { field_clears, .. }
        if field_clears == &[field_key("note")])
    );
    let completed_at = clear.commit.commit_id;
    release_test_commit_snapshot(&runtime, &clear);
    let saved = runtime
        .durability_authority()
        .native_checkpoint(execution.allocation_policy())
        .unwrap();
    let mut cold = schema::runtime();
    let receipt = cold
        .prepare_initial_schema_installation()
        .unwrap()
        .install(crate::schema::data::RelationalSchemaRegistry::new())
        .unwrap();
    cold.durability_recovery()
        .restore_native_checkpoint(execution, &saved)
        .unwrap();
    let receipt = cold
        .readmit_recovered_initial_schema_installation(receipt)
        .unwrap();
    let (_, installed) = receipt.issue_application_publication_owner();
    let basis = test_owner_main_basis(&cold).unwrap();
    let partition = basis
        .inner
        .root
        .partition_state(PartitionId::main())
        .unwrap();
    for (id, kind) in &members {
        let row = partition.entity_arena.get(id).unwrap();
        assert!(row.is_live());
        assert_eq!(row.kind_id(), Some(*kind));
        assert_eq!(row.generation(), id.generation_value());
        if *kind == KindId(1) {
            assert!(row.extra().authoritative_aspect_state.is_none());
        }
    }
    let row = partition.entity_arena.get(&members[0].0).unwrap();
    let aspects = row.extra().authoritative_aspect_state.as_ref().unwrap();
    assert!(aspects.get(&aspect_key("attachment")).is_none());
    let ContractValidatedAspectValueView::Struct(facts) =
        aspects.get(&aspect_key("facts")).unwrap().view()
    else {
        panic!("struct")
    };
    assert_eq!(facts.len(), 3);
    assert!(facts.get(&field_key("note")).is_none());
    let row = partition.entity_arena.get(&members[5].0).unwrap();
    assert!(row
        .extra()
        .authoritative_aspect_state
        .as_ref()
        .unwrap()
        .get(&aspect_key("nominal"))
        .is_none());
    let readmitted = cold
        .readmit_required_output_publication(
            execution,
            &installed,
            &basis,
            "generic-preserver",
            &[3; 32],
            &crate::mvcc::native_meaning_fixture("generic-creator"),
            &crate::mvcc::native_roles_fixture(&members, RequiredOutputRolePosture::Preserve),
        )
        .unwrap();
    assert_eq!(readmitted.envelope.commit.commit_id, completed_at);
    let origin = readmitted.envelope.required_output_origin.as_ref().unwrap();
    assert_eq!(origin.birth, born_at);
    assert_eq!(origin.members.len(), 6);
    assert_eq!(origin.outputs, {
        let mut ids = members.iter().map(|(id, _)| *id).collect::<Vec<_>>();
        ids.sort();
        ids
    });
    assert_eq!(
        origin
            .relations
            .iter()
            .map(|relation| relation.id)
            .collect::<Vec<_>>(),
        {
            let mut ids = relations;
            ids.sort();
            ids
        }
    );
    let ordinary = create_entity_outcome(execution, &cold, "ordinary-fieldless-generic");
    let ordinary_id = changed_entities(&ordinary)[0];
    assert!(ordinary.required_output_publication(&installed).is_none());
    release_test_commit_snapshot(&cold, &ordinary);
    let basis = test_owner_main_basis(&cold).unwrap();
    assert!(cold
        .readmit_required_output_publication(
            execution,
            &installed,
            &basis,
            "generic-creator",
            &[1; 32],
            &crate::mvcc::native_meaning_fixture("generic-creator"),
            &crate::mvcc::native_roles_fixture(
                &[(ordinary_id, KindId(1))],
                RequiredOutputRolePosture::Create
            )
        )
        .is_err());
}

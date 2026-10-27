//! The raw Native door must enforce field authority independently of typed writers.
use super::*;
use fixture::relation::{Initialize, LinkOnly};
use worth_query_decl::facade::application_schema::ApplicationScalarValueBinding;
#[test]
fn native_initialized_relation_requires_link_and_actual_field_authority() {
    check(|execution| {
        let world = World::new(execution, false, &[]);
        let before = world
            .native
            .observe_branch(&world.native.main_branch_identity())
            .unwrap()
            .0;
        let batch = initialized_batch(&world);
        let (_, basis) = world
            .native
            .observe_branch(&world.native.main_branch_identity())
            .unwrap();
        let mut transaction = world
            .native
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .unwrap();
        transaction
            .push_batch(batch.clone(), execution.allocation_policy())
            .unwrap();
        let candidate = transaction.validate(execution, &world.native).unwrap();
        let link = world
            .schema
            .installed_operation(LinkOnly::reference())
            .unwrap();
        let initialized = world
            .schema
            .installed_operation(Initialize::reference())
            .unwrap();
        let admit = |contracts: &worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts| {
            admit_validated_application_touches(execution, &candidate, &world.layout,
                contracts.graph_reads(), contracts.touches(), contracts.read_touch_overlap())
        };
        assert!(matches!(
            admit(link.contracts()),
            Err(TouchAdmissionFailure::Contract)
        ));
        admit(initialized.contracts()).expect("actual Link and initialized-field authority");
        assert_eq!(
            world
                .native
                .observe_branch(&world.native.main_branch_identity())
                .unwrap()
                .0,
            before
        );
        assert_eq!(world.truth(execution), vec![None, None]);
        drop(candidate);
    });
}

#[test]
fn field_locus_preserves_legacy_entity_sequence_and_explicit_relation_owner() {
    use worth_query_decl::facade::application_schema::ApplicationFieldBindingLocus as Locus;
    let legacy = serde_json::json!(["Edge", "Payload", "Weight"]);
    let entity: Locus = serde_json::from_value(legacy).unwrap();
    assert_eq!(entity, Locus::new("Edge", "Payload", "Weight"));
    assert_eq!(
        serde_json::to_value(&entity).unwrap(),
        serde_json::json!({"entity":"Edge", "aspect":"Payload", "field":"Weight"})
    );
    let relation: Locus =
        serde_json::from_value(serde_json::json!(["Edge", "Payload", "Weight", "Relation"]))
            .unwrap();
    assert_eq!(relation, Locus::for_relation("Edge", "Payload", "Weight"));
    assert_ne!(entity, relation);
    let serialized = serde_json::to_value(&relation).unwrap();
    assert_eq!(serialized["owner_kind"], "Relation");
    assert_eq!(
        serde_json::from_value::<Locus>(serialized).unwrap(),
        relation
    );
}

fn initialized_batch(world: &World) -> WorkerIntentBatch {
    let mut fields = std::collections::BTreeMap::new();
    fields.insert(
        world
            .layout
            .relation_field_locator("Edge", "Payload", "Weight")
            .unwrap()
            .clone(),
        worth_query_decl::facade::application_schema::U64ApplicationValueBinding::encode(&7)
            .unwrap(),
    );
    WorkerIntentBatch::new("raw-initialized-relation").push(MutationIntent::Create(
        CreateIntent::Relation(RelationSpec {
            partition_id: world.entity.partition_id,
            kind_id: world.layout.relation("Edge").unwrap().kind,
            client_key: worth_relational::facade::symbols::ClientKey::raw("edge"),
            source: EntityReference::Existing(world.entity),
            target: EntityReference::Existing(world.entity),
            fields: AspectFieldPatch::from(fields),
        }),
    ))
}

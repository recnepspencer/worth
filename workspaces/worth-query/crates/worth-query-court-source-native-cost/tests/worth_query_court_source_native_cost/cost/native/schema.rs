use worth_foundational::facade::AspectValue;
use worth_relational::facade::{
    config::*, identity::*, indexes::*, runtime::*, schema::*, transactions::*,
};
use worth_testkit_relational::aspect_storage::{
    self as aspects,
    definition::{aspect_field_locator, aspect_key, field_key},
};

pub fn runtime(installed: usize) -> (RelationalRuntime, Vec<DerivedIndexId>) {
    let fixture = aspects::AspectSchemaFixture::with_default_declared_aspects(
        CascadeDeletePolicy::CascadeDeleteRelations,
    );
    let extra = (0..installed)
        .map(|i| {
            let name = format!("bystander-{i}");
            aspects::entity_field_aspect(aspect_key(&name), field_key(&name))
        })
        .collect();
    let registry = fixture
        .build_registry()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(3),
            kind_name: "unrelated".into(),
            schema_id: SchemaId("test".into()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(extra),
        })
        .unwrap();
    let runtime = RelationalRuntimeBuilder::new()
        .schema_registry(registry)
        .build()
        .unwrap();
    let mut kinds = vec![
        DerivedIndexKind::EntityField {
            field_locator: locator("name"),
        },
        DerivedIndexKind::RelationJoin(RelationJoinDefinition::new(
            RelationJoinLeg::new(KindId(2), RelationJoinSharedEndpoint::Target, KindId(1)),
            RelationJoinLeg::new(KindId(2), RelationJoinSharedEndpoint::Source, KindId(1)),
            KindId(1),
        )),
    ];
    kinds.extend((0..installed).map(|i| DerivedIndexKind::EntityField {
        field_locator: locator(&format!("bystander-{i}")),
    }));
    let ids = kinds
        .into_iter()
        .enumerate()
        .map(|(i, kind)| {
            runtime
                .index_authority()
                .register(DerivedIndexDefinition {
                    index_id: DerivedIndexId(0),
                    name: format!("scale-{i}"),
                    kind,
                    branch_scoped: true,
                })
                .unwrap()
                .index_id
        })
        .collect();
    (runtime, ids)
}

pub fn locator(name: &str) -> worth_foundational::facade::AspectFieldLocator {
    aspect_field_locator(aspect_key(name), field_key(name))
}
pub fn patch(name: &str, value: &str) -> AspectFieldPatch {
    AspectFieldPatch::from_locator(
        planned_single_field_locator(aspect_key(name), field_key(name)),
        AspectValue::String(value.into()),
    )
}

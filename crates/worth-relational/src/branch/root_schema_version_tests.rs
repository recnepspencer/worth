//! Primary-version equivalence at immutable authority capture and readmission.
use super::*;
use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy};
use crate::identity::data::KindId;
use crate::schema::data::{
    EntityKindRegistration, RelationKindRegistration, SchemaId, SchemaVersionId,
};

fn registry(bases: &[(&str, u32)]) -> RelationalSchemaRegistry {
    bases.iter().enumerate().fold(
        RelationalSchemaRegistry::new(),
        |registry, (i, (schema, version))| {
            registry
                .register_entity_kind(EntityKindRegistration {
                    kind_id: KindId(i as u32 + 1),
                    kind_name: format!("entity-{i}"),
                    schema_id: SchemaId((*schema).into()),
                    schema_version_id: SchemaVersionId(*version),
                    aspect_contract_declarations: Default::default(),
                })
                .unwrap()
        },
    )
}

fn with_relation(
    registry: RelationalSchemaRegistry,
    schema: &str,
    version: u32,
) -> RelationalSchemaRegistry {
    registry
        .register_relation_kind(RelationKindRegistration {
            kind_id: KindId(100),
            kind_name: "relation".into(),
            schema_id: SchemaId(schema.into()),
            schema_version_id: SchemaVersionId(version),
            cross_context_policy: CrossContextPolicy::AllowExplicit,
            cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
            aspect_contract_declarations: Default::default(),
            relation_integrity: Default::default(),
        })
        .unwrap()
}

#[test]
fn primary_version_matches_full_projection_across_registry_shapes_and_rebinding() {
    let semantics =
        crate::schema::data::runtime_descriptor_semantics_policy().current_write_version();
    let mut previous = None;
    for registry in [
        registry(&[]),
        registry(&[("one", 7)]),
        registry(&[("one", 7), ("one", 7)]),
        registry(&[("one", 7), ("two", 9)]),
        registry(&[("one", 7), ("one", 9)]),
        registry(&[("one", 8), ("one", 8)]),
        with_relation(registry(&[]), "one", 7),
        with_relation(registry(&[("one", 7)]), "one", 7),
        with_relation(registry(&[("one", 7)]), "two", 7),
        with_relation(registry(&[("one", 7)]), "one", 9),
    ] {
        let expected = registry.authority_snapshot();
        let version = expected
            .primary_schema_version_id
            .unwrap_or(SchemaVersionId(0));
        let root = RelationalBranchRootSchemaAuthority::capture(
            1,
            &registry,
            &expected,
            semantics,
            previous.as_deref(),
        )
        .unwrap();
        assert_eq!(root.schema_version(), version, "registry: {registry:?}");
        assert_eq!(root.authority_digest(), registry.authority_digest_bytes());
        assert_eq!(registry.primary_schema_version_id(), version);
        let restored = RelationalBranchRootSchemaAuthority::readmit_exact(
            2,
            registry,
            root.retained_aspect_contracts().to_vec(),
            &expected,
            semantics,
        )
        .unwrap();
        assert_eq!(restored.schema_version(), version);
        assert_eq!(restored.authority_digest(), root.authority_digest());
        previous = Some(root);
    }
}

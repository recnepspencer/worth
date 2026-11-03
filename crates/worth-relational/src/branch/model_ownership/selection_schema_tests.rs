//! Pure schema-admission policy: registry commitments, with no runtime roots.
use super::require_same_schema;
use crate::tests::support::{aspect_key, entity_field_aspect, field_key, AspectSchemaFixture};
use crate::{branch::RelationalModelOwnershipDenial as Denial, schema::data::SchemaVersionId};

#[test]
fn entity_boundary_refuses_changed_schema_meaning_before_structural_selection() {
    let declared = AspectSchemaFixture::default();
    let original = declared.build_registry();
    let current = original.authority_digest_bytes();
    assert!(
        require_same_schema(current, declared.build_registry().authority_digest_bytes()).is_ok()
    );
    let mut changed_contract = declared.clone();
    changed_contract
        .entity_aspects
        .push(entity_field_aspect(aspect_key("tag"), field_key("tag")));
    let changed_revision = AspectSchemaFixture {
        schema_version_id: SchemaVersionId(2),
        ..declared
    };
    for changed in [
        changed_contract.build_registry(),
        changed_revision.build_registry(),
    ] {
        // Kind ids/counts alone are insufficient: neither alteration changes
        // the entity/relation identity sets. Use actual recorded schema meaning.
        assert_eq!(
            original
                .authority_snapshot()
                .entity_kinds
                .iter()
                .map(|kind| kind.kind_id)
                .collect::<Vec<_>>(),
            changed
                .authority_snapshot()
                .entity_kinds
                .iter()
                .map(|kind| kind.kind_id)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            original
                .authority_snapshot()
                .relation_kinds
                .iter()
                .map(|kind| kind.kind_id)
                .collect::<Vec<_>>(),
            changed
                .authority_snapshot()
                .relation_kinds
                .iter()
                .map(|kind| kind.kind_id)
                .collect::<Vec<_>>()
        );
        assert!(matches!(
            require_same_schema(current, changed.authority_digest_bytes()),
            Err(Denial::SchemaChanged)
        ));
    }
}

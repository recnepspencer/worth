//! Root-free admission over the actual installed program roster.
use super::*;
use crate::identity::data::KindId;
use crate::validation::candidate_source::SourcePreparationGroup;
use worth_foundational::facade::{AspectBinding, AspectKey, FieldKey};

#[test]
fn source_only_codec_consequences_refuse_affected_support_and_allow_an_unrelated_kind() {
    let (mut registry, program) = crate::tests::support::managed_cache_schema();
    let unrelated_kind = KindId(
        registry
            .entity_kinds
            .keys()
            .chain(registry.relation_kinds.keys())
            .map(|kind| kind.0)
            .max()
            .unwrap()
            + 1,
    );
    let schema = registry.entity_kinds[&KindId(1)].schema_id.clone();
    registry = registry
        .register_entity_kind(crate::schema::data::EntityKindRegistration {
            kind_id: unrelated_kind,
            kind_name: "outside-managed-source".into(),
            schema_id: schema,
            schema_version_id: crate::schema::data::SchemaVersionId(1),
            aspect_contract_declarations: Default::default(),
        })
        .unwrap();
    let groups = SourcePreparationGroup::for_public_sources(&registry, vec![program]).unwrap();
    let installed = FrozenCustomInvariantRegistry::from_registrations(vec![], groups).unwrap();
    assert_eq!(installed.len(), 0, "no invariant supplies this consequence");
    let change = |kind_id| CodecFieldLocus {
        kind_id,
        aspect_key: AspectKey::new("codec-change").unwrap(),
        field_key: FieldKey::new("value").unwrap(),
        binding: AspectBinding::EntityField {
            field: FieldKey::new("value").unwrap(),
        },
    };
    assert_eq!(
        admit_supported_consequences(&installed, [], &[change(KindId(1))]),
        Err("codec transition with managed Source consequences is unsupported".into()),
    );
    assert_eq!(
        admit_supported_consequences(&installed, [], &[change(unrelated_kind)]),
        Ok(()),
    );
}

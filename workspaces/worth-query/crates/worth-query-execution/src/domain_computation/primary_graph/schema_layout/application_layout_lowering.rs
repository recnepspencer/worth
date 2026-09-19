use std::collections::{BTreeMap, BTreeSet};

use worth_foundational::facade::{AspectKey, FieldKey};
use worth_query_installation::facade::{
    ApplicationSchemaMember, ErasedApplicationSchemaDeclaration,
};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::indexes::DerivedIndexId;

use super::{
    planned_field_locator, required_kind, WorthQueryPrimaryFieldLayout,
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryRelationLayout,
};

pub(super) fn field_capability_keys<'a>(
    fields: impl IntoIterator<Item = &'a WorthQueryPrimaryFieldLayout>,
) -> BTreeMap<AspectKey, BTreeSet<FieldKey>> {
    let mut keys = BTreeMap::<AspectKey, BTreeSet<FieldKey>>::new();
    for field in fields {
        let aspect = field.locator.aspect().aspect_key().clone();
        let field_key = field
            .locator
            .field_path()
            .fields()
            .first()
            .expect("primary application fields always have a single canonical field")
            .clone();
        keys.entry(aspect).or_default().insert(field_key);
    }
    keys
}

pub(super) fn lower_relation_layouts(
    schema: &ErasedApplicationSchemaDeclaration,
    entity_kinds: &BTreeMap<String, KindId>,
    relation_kinds: &BTreeMap<String, KindId>,
) -> Result<
    BTreeMap<String, WorthQueryPrimaryRelationLayout>,
    WorthQueryPrimaryGraphInstallationDenial,
> {
    schema
        .members()
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::Relation {
                relation, from, to, ..
            } => Some((relation, from, to)),
            _ => None,
        })
        .map(|(relation, from, to)| {
            Ok((
                relation.clone(),
                WorthQueryPrimaryRelationLayout {
                    kind: required_kind(relation_kinds, relation)?,
                    from: required_kind(entity_kinds, from)?,
                    to: required_kind(entity_kinds, to)?,
                },
            ))
        })
        .collect()
}

pub(super) fn lower_fields(
    schema: &ErasedApplicationSchemaDeclaration,
    entity_kinds: &BTreeMap<String, KindId>,
) -> Result<
    BTreeMap<(String, String, String), WorthQueryPrimaryFieldLayout>,
    WorthQueryPrimaryGraphInstallationDenial,
> {
    schema
        .members()
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::Field {
                entity,
                aspect,
                field,
                equality_queryable,
                ..
            } => Some((entity, aspect, field, equality_queryable)),
            _ => None,
        })
        .map(|(entity, aspect, field, equality_queryable)| {
            Ok((
                (entity.clone(), aspect.clone(), field.clone()),
                WorthQueryPrimaryFieldLayout {
                    entity_kind: required_kind(entity_kinds, entity)?,
                    locator: planned_field_locator(aspect, field)?,
                    equality_index_id: equality_queryable.then_some(DerivedIndexId(0)),
                },
            ))
        })
        .collect()
}

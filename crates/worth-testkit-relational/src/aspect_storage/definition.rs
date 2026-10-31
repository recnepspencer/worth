//! Shared schema meaning; no Native runtime or Native nominal types cross here.
use worth_foundational::facade::*;

pub type DeclaredAspect = (AspectBinding, AspectContract);

pub struct AspectSchemaNames {
    pub entity_kind: u32,
    pub relation_kind: u32,
    pub entity_name: &'static str,
    pub relation_name: &'static str,
    pub schema_name: &'static str,
    pub schema_revision: u32,
}
impl Default for AspectSchemaNames {
    fn default() -> Self {
        Self {
            entity_kind: 1,
            relation_kind: 2,
            entity_name: "test.entity",
            relation_name: "test.relation",
            schema_name: "test",
            schema_revision: 1,
        }
    }
}

pub fn aspect_key(name: &str) -> AspectKey {
    AspectKey::new(name).unwrap()
}
pub fn field_key(name: &str) -> FieldKey {
    FieldKey::new(name).unwrap()
}
pub fn aspect_field_locator(aspect: AspectKey, field: FieldKey) -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Planned,
        aspect,
        CanonicalFieldPath::single(field),
    )
}

pub fn entity_field_aspect(aspect: AspectKey, field: FieldKey) -> DeclaredAspect {
    (
        AspectBinding::EntityField { field },
        scalar_contract(aspect, ScalarAspectType::String),
    )
}
pub fn entity_u64_field_aspect(aspect: AspectKey, field: FieldKey) -> DeclaredAspect {
    (
        AspectBinding::EntityField { field },
        scalar_contract(aspect, ScalarAspectType::UInt64),
    )
}
pub fn entity_i64_field_aspect(aspect: AspectKey, field: FieldKey) -> DeclaredAspect {
    (
        AspectBinding::EntityField { field },
        scalar_contract(aspect, ScalarAspectType::Int64),
    )
}
pub fn entity_bool_field_aspect(aspect: AspectKey, field: FieldKey) -> DeclaredAspect {
    (
        AspectBinding::EntityField { field },
        scalar_contract(aspect, ScalarAspectType::Bool),
    )
}
pub fn relation_field_aspect(aspect: AspectKey, field: FieldKey) -> DeclaredAspect {
    (
        AspectBinding::RelationField { field },
        scalar_contract(aspect, ScalarAspectType::String),
    )
}
pub fn lifecycle_aspect() -> DeclaredAspect {
    (
        AspectBinding::LifecycleTransition,
        scalar_contract(aspect_key("lifecycle"), ScalarAspectType::String),
    )
}
pub fn relation_source_aspect() -> DeclaredAspect {
    (
        AspectBinding::RelationSourceEndpoint,
        reference_contract(aspect_key("source")),
    )
}
pub fn relation_target_aspect() -> DeclaredAspect {
    (
        AspectBinding::RelationTargetEndpoint,
        reference_contract(aspect_key("target")),
    )
}
pub fn entity_summary_struct_aspect(aspect: AspectKey, field: FieldKey) -> DeclaredAspect {
    let shape = aspects()
        .struct_fields()
        .required("title", ScalarAspectType::String)
        .optional("status", ScalarAspectType::String)
        .finish()
        .unwrap();
    (
        AspectBinding::EntityField { field },
        aspects()
            .contract()
            .for_key(aspect.clone())
            .identified_by(AspectIdentity(contract_identity(&aspect)))
            .at_revision(aspects().vocabulary().revision(1))
            .struct_aspect(shape),
    )
}
pub fn default_entity_aspects() -> Vec<DeclaredAspect> {
    vec![
        entity_field_aspect(aspect_key("name"), field_key("name")),
        lifecycle_aspect(),
    ]
}
pub fn default_relation_aspects() -> Vec<DeclaredAspect> {
    vec![
        relation_field_aspect(aspect_key("label"), field_key("label")),
        lifecycle_aspect(),
        relation_source_aspect(),
        relation_target_aspect(),
    ]
}
pub fn optional_string_contract(name: &str) -> AspectContract {
    aspects()
        .contract()
        .for_key(aspect_key(name))
        .identified_by(AspectIdentity(991))
        .at_revision(aspects().vocabulary().revision(1))
        .scalar_with(
            ScalarAspectType::String,
            AspectMaskContract::scalar(),
            AbsenceLaw::Optional,
            AspectEquivalenceBasis::ExactCanonicalValue,
            AspectEvolutionPolicy::ExplicitBreakRequired,
        )
}
fn scalar_contract(aspect: AspectKey, scalar: ScalarAspectType) -> AspectContract {
    aspects()
        .contract()
        .for_key(aspect.clone())
        .identified_by(AspectIdentity(contract_identity(&aspect)))
        .at_revision(aspects().vocabulary().revision(1))
        .scalar(scalar)
}
fn reference_contract(aspect: AspectKey) -> AspectContract {
    aspects()
        .contract()
        .for_key(aspect.clone())
        .identified_by(AspectIdentity(contract_identity(&aspect)))
        .at_revision(aspects().vocabulary().revision(1))
        .reference_entity()
}
fn contract_identity(key: &AspectKey) -> u64 {
    let mut hash = 14695981039346656037_u64;
    for byte in key.as_str().as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211_u64);
    }
    hash
}

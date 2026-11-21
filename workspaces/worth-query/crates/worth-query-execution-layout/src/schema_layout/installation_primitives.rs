//! Shared validation primitives for lowering application schema declarations.

use std::collections::BTreeMap;

use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, CanonicalFieldPath, FieldKey, LocatorAuthority,
};
use worth_relational::facade::identity::KindId;

use crate::denial::{WorthQuerySchemaLayoutDenial, WorthQuerySchemaLayoutDenialKind};

pub fn required_kind(
    kinds: &BTreeMap<String, KindId>,
    name: &str,
) -> Result<KindId, WorthQuerySchemaLayoutDenial> {
    kinds.get(name).copied().ok_or_else(|| invalid_member(name))
}

pub fn planned_field_locator(
    aspect: &str,
    field: &str,
) -> Result<AspectFieldLocator, WorthQuerySchemaLayoutDenial> {
    Ok(AspectFieldLocator::new(
        LocatorAuthority::Planned,
        valid_aspect_key(aspect)?,
        CanonicalFieldPath::single(valid_field_key(field)?),
    ))
}

pub fn valid_aspect_key(value: &str) -> Result<AspectKey, WorthQuerySchemaLayoutDenial> {
    AspectKey::new(value).ok_or_else(|| invalid_member(value))
}

pub fn valid_field_key(value: &str) -> Result<FieldKey, WorthQuerySchemaLayoutDenial> {
    FieldKey::new(value).ok_or_else(|| invalid_member(value))
}

pub fn invalid_member(subject: &str) -> WorthQuerySchemaLayoutDenial {
    WorthQuerySchemaLayoutDenial::new(
        WorthQuerySchemaLayoutDenialKind::InvalidSchemaMember,
        subject,
    )
}

pub fn kind_space_exhausted() -> WorthQuerySchemaLayoutDenial {
    invalid_member("application schema exhausts Relational kind identity space")
}

pub fn contract_space_exhausted() -> WorthQuerySchemaLayoutDenial {
    invalid_member("application schema exhausts Relational aspect-contract identity space")
}

pub fn relational_schema_denial(
    denial: worth_relational::facade::schema::SchemaRegistryError,
) -> WorthQuerySchemaLayoutDenial {
    WorthQuerySchemaLayoutDenial::new(
        WorthQuerySchemaLayoutDenialKind::RelationalSchemaRejected,
        format!("{denial:?}"),
    )
}

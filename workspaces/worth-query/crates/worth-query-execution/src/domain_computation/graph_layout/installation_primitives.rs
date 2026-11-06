//! Shared validation primitives for lowering application schema declarations.

use std::collections::BTreeMap;

use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, CanonicalFieldPath, FieldKey, LocatorAuthority,
};
use worth_relational::facade::identity::KindId;

use {crate::domain_computation::graph_layout::denial::WorthQueryPrimaryGraphInstallationDenial, crate::domain_computation::graph_layout::denial::WorthQueryPrimaryGraphInstallationDenialKind};

pub(in crate::domain_computation) fn required_kind(
    kinds: &BTreeMap<String, KindId>,
    name: &str,
) -> Result<KindId, WorthQueryPrimaryGraphInstallationDenial> {
    kinds.get(name).copied().ok_or_else(|| invalid_member(name))
}

pub(in crate::domain_computation) fn planned_field_locator(
    aspect: &str,
    field: &str,
) -> Result<AspectFieldLocator, WorthQueryPrimaryGraphInstallationDenial> {
    Ok(AspectFieldLocator::new(
        LocatorAuthority::Planned,
        valid_aspect_key(aspect)?,
        CanonicalFieldPath::single(valid_field_key(field)?),
    ))
}

pub(in crate::domain_computation) fn valid_aspect_key(
    value: &str,
) -> Result<AspectKey, WorthQueryPrimaryGraphInstallationDenial> {
    AspectKey::new(value).ok_or_else(|| invalid_member(value))
}

pub(in crate::domain_computation) fn valid_field_key(
    value: &str,
) -> Result<FieldKey, WorthQueryPrimaryGraphInstallationDenial> {
    FieldKey::new(value).ok_or_else(|| invalid_member(value))
}

pub(in crate::domain_computation) fn invalid_member(
    subject: &str,
) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(
        WorthQueryPrimaryGraphInstallationDenialKind::InvalidSchemaMember,
        subject,
    )
}

pub(in crate::domain_computation) fn kind_space_exhausted(
) -> WorthQueryPrimaryGraphInstallationDenial {
    invalid_member("application schema exhausts Relational kind identity space")
}

pub(in crate::domain_computation) fn contract_space_exhausted(
) -> WorthQueryPrimaryGraphInstallationDenial {
    invalid_member("application schema exhausts Relational aspect-contract identity space")
}

pub(in crate::domain_computation) fn relational_schema_denial(
    denial: worth_relational::facade::schema::SchemaRegistryError,
) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(
        WorthQueryPrimaryGraphInstallationDenialKind::RelationalSchemaRejected,
        format!("{denial:?}"),
    )
}

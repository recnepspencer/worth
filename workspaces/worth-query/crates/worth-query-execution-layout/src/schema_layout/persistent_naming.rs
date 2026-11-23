//! Declared persistent-name schemes bind to the exact kinds this layout lowered.
//!
//! Relational's commit finalization reads schemes only from its installed
//! schema registry, so a declared scheme that is not installed here would let
//! a schemed kind commit unnamed records. A schemaless declaration returns the
//! registry untouched, keeping its authority digest bit-identical.

use std::collections::BTreeMap;

use worth_query_declaration::facade::application_naming::ApplicationNameSubjectKind;
use worth_query_execution_naming::facade::LoweredExecutionNameSchemeCatalog;
use worth_query_installation::facade::{
    install_schema_name_schemes, ErasedApplicationSchemaDeclaration,
};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::lineage::PersistentNameSubjectKind;
use worth_relational::facade::schema::RelationalSchemaRegistry;

use super::{relational_schema_denial, required_kind};
use crate::denial::{WorthQuerySchemaLayoutDenial, WorthQuerySchemaLayoutDenialKind};

/// The installed naming contract an installed world mints births against:
/// the lowered scheme catalog and the scheme bound to each schemed kind.
#[derive(Debug)]
pub struct WorthQueryPersistentNameLayout {
    catalog: LoweredExecutionNameSchemeCatalog,
    kind_schemes: BTreeMap<(PersistentNameSubjectKind, KindId), String>,
}

impl WorthQueryPersistentNameLayout {
    pub fn catalog(&self) -> &LoweredExecutionNameSchemeCatalog {
        &self.catalog
    }

    /// The scheme Relational requires every birth of `kind` to name.
    pub fn scheme_for_kind(
        &self,
        subject: PersistentNameSubjectKind,
        kind: KindId,
    ) -> Option<&str> {
        self.kind_schemes.get(&(subject, kind)).map(String::as_str)
    }
}

/// Installs every declared entity and relation scheme of `schema` into
/// `registry`, bound to the kind identities in `entity_kinds` and
/// `relation_kinds`. Group-subject schemes name aggregates, not kinds, and
/// install without a kind binding.
///
/// Complexity: O(S * R log R) for S declared schemes of R roles each.
pub(super) fn lower(
    registry: RelationalSchemaRegistry,
    schema: &ErasedApplicationSchemaDeclaration,
    entity_kinds: &BTreeMap<String, KindId>,
    relation_kinds: &BTreeMap<String, KindId>,
) -> Result<
    (
        RelationalSchemaRegistry,
        Option<WorthQueryPersistentNameLayout>,
    ),
    WorthQuerySchemaLayoutDenial,
> {
    let catalog = install_schema_name_schemes(schema).map_err(|denial| {
        WorthQuerySchemaLayoutDenial::new(
            WorthQuerySchemaLayoutDenialKind::InvalidSchemaMember,
            format!("persistent-name scheme installation refused: {denial:?}"),
        )
    })?;
    if catalog.schemes().len() == 0 {
        return Ok((registry, None));
    }
    let lowered =
        LoweredExecutionNameSchemeCatalog::lower_from_installed(&catalog).map_err(|denial| {
            WorthQuerySchemaLayoutDenial::new(
                WorthQuerySchemaLayoutDenialKind::InvalidSchemaMember,
                format!("persistent-name scheme lowering refused: {denial:?}"),
            )
        })?;
    let mut bindings = BTreeMap::new();
    for layout in catalog.schemes() {
        let (subject, kinds) = match layout.subject_kind() {
            ApplicationNameSubjectKind::Entity => (PersistentNameSubjectKind::Entity, entity_kinds),
            ApplicationNameSubjectKind::Relation => {
                (PersistentNameSubjectKind::Relation, relation_kinds)
            }
            ApplicationNameSubjectKind::Group => continue,
        };
        let kind = required_kind(kinds, layout.subject_name())?;
        bindings.insert((subject, kind), layout.scheme_id().to_owned());
    }
    let registry = registry
        .install_persistent_name_schemes(lowered.relational_registry().clone(), bindings.clone())
        .map_err(relational_schema_denial)?;
    Ok((
        registry,
        Some(WorthQueryPersistentNameLayout {
            catalog: lowered,
            kind_schemes: bindings,
        }),
    ))
}

#[cfg(test)]
#[path = "persistent_naming_tests.rs"]
mod tests;

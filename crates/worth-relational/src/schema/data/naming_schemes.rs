//! Installed persistent-name scheme registry binding onto
//! [`RelationalSchemaRegistry`].

use std::collections::BTreeMap;

use crate::identity::data::KindId;
use crate::lineage::data::{
    InstalledPersistentNameScheme, InstalledPersistentNameSchemeRegistry,
    PersistentNameSubjectKind,
};
use crate::facade::schema::SchemaRegistryError;

use super::RelationalSchemaRegistry;

impl RelationalSchemaRegistry {
    /// Installs validated persistent-name schemes and binds each schemed entity
    /// or relation `KindId` to its scheme identifier on this schema registry.
    ///
    /// Once installed, Relational commit finalization reads schemes exclusively
    /// from this registry and fails closed with `LineageNameDenial::MissingBirth`
    /// if a commit creates a record of any bound kind without a birth or
    /// posture covering it.
    pub fn install_persistent_name_schemes(
        mut self,
        registry: InstalledPersistentNameSchemeRegistry,
        kind_bindings: BTreeMap<(PersistentNameSubjectKind, KindId), String>,
    ) -> Result<Self, SchemaRegistryError> {
        for (&(subject_kind, kind_id), scheme_id) in &kind_bindings {
            let expected_kind_name = match subject_kind {
                PersistentNameSubjectKind::Entity => {
                    self.entity_registration(kind_id)?.kind_name.as_str()
                }
                PersistentNameSubjectKind::Relation => {
                    self.relation_registration(kind_id)?.kind_name.as_str()
                }
                PersistentNameSubjectKind::Group => {
                    return Err(SchemaRegistryError::inconsistent_schema_basis(format!(
                        "group persistent-name scheme `{scheme_id}` cannot bind to entity/relation kind {kind_id:?}"
                    )));
                }
            };
            let Some(scheme) = registry.get(scheme_id) else {
                return Err(SchemaRegistryError::inconsistent_schema_basis(format!(
                    "kind {kind_id:?} references unknown persistent-name scheme `{scheme_id}`"
                )));
            };
            if scheme.subject_kind() != subject_kind {
                return Err(SchemaRegistryError::inconsistent_schema_basis(format!(
                    "scheme `{scheme_id}` subject kind {:?} does not match binding {:?}",
                    scheme.subject_kind(),
                    subject_kind
                )));
            }
            if scheme.subject_name() != expected_kind_name {
                return Err(SchemaRegistryError::inconsistent_schema_basis(format!(
                    "scheme `{scheme_id}` subject `{}` does not match registered kind `{expected_kind_name}`",
                    scheme.subject_name()
                )));
            }
        }
        self.persistent_name_schemes = registry;
        self.kind_name_schemes = kind_bindings;
        Ok(self)
    }

    pub fn persistent_name_schemes(&self) -> &InstalledPersistentNameSchemeRegistry {
        &self.persistent_name_schemes
    }

    pub fn persistent_name_scheme(&self, scheme_id: &str) -> Option<&InstalledPersistentNameScheme> {
        self.persistent_name_schemes.get(scheme_id)
    }

    pub fn scheme_id_for_kind(
        &self,
        subject_kind: PersistentNameSubjectKind,
        kind_id: KindId,
    ) -> Option<&str> {
        self.kind_name_schemes
            .get(&(subject_kind, kind_id))
            .map(String::as_str)
    }

    pub fn has_kind_name_schemes(&self) -> bool {
        !self.kind_name_schemes.is_empty()
    }
}

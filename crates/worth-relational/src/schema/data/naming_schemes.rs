//! Installed persistent-name scheme registry binding and durable canonical
//! snapshot layout on [`RelationalSchemaRegistry`].

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::facade::schema::SchemaRegistryError;
use crate::identity::data::KindId;
use crate::lineage::data::{
    InstalledPersistentNameScheme, InstalledPersistentNameSchemeRegistry,
    PersistentNameSubjectKind,
};

use super::RelationalSchemaRegistry;

/// Canonical durable snapshot of one installed persistent-name scheme.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaAuthorityNamingSchemeSnapshot {
    pub scheme_id: String,
    pub subject_kind: PersistentNameSubjectKind,
    pub subject_name: String,
    pub ordered_roles: Vec<String>,
    pub split_capable: bool,
    pub require_authored_occurrence: bool,
}

impl SchemaAuthorityNamingSchemeSnapshot {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        let roles_vec = (self.ordered_roles.capacity() as u64)
            .saturating_mul(std::mem::size_of::<String>() as u64);
        let roles_text: u64 = self
            .ordered_roles
            .iter()
            .map(|role| role.capacity() as u64)
            .sum();
        (self.scheme_id.capacity() as u64)
            .saturating_add(self.subject_name.capacity() as u64)
            .saturating_add(roles_vec)
            .saturating_add(roles_text)
    }
}

/// Canonical durable snapshot of one `(subject_kind, KindId) -> scheme_id` binding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaAuthorityKindNamingBindingSnapshot {
    pub subject_kind: PersistentNameSubjectKind,
    pub kind_id: KindId,
    pub scheme_id: String,
}

impl SchemaAuthorityKindNamingBindingSnapshot {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        self.scheme_id.capacity() as u64
    }
}

pub(crate) fn persistent_name_subject_kind_tag(kind: PersistentNameSubjectKind) -> u8 {
    match kind {
        PersistentNameSubjectKind::Entity => 1,
        PersistentNameSubjectKind::Relation => 2,
        PersistentNameSubjectKind::Group => 3,
    }
}

/// Hashes canonical naming scheme snapshots and kind bindings into `hasher`
/// under `domain_tag` ONLY when at least one is non-empty, keeping schemaless
/// root digests 100% bit-identical.
pub(crate) fn update_naming_layout_digest(
    hasher: &mut Sha256,
    domain_tag: &[u8],
    schemes: &[SchemaAuthorityNamingSchemeSnapshot],
    bindings: &[SchemaAuthorityKindNamingBindingSnapshot],
) {
    if schemes.is_empty() && bindings.is_empty() {
        return;
    }
    hasher.update(domain_tag);
    hasher.update((schemes.len() as u64).to_le_bytes());
    for scheme in schemes {
        update_len_bytes(hasher, scheme.scheme_id.as_bytes());
        hasher.update([persistent_name_subject_kind_tag(scheme.subject_kind)]);
        update_len_bytes(hasher, scheme.subject_name.as_bytes());
        hasher.update((scheme.ordered_roles.len() as u64).to_le_bytes());
        for role in &scheme.ordered_roles {
            update_len_bytes(hasher, role.as_bytes());
        }
        hasher.update([u8::from(scheme.split_capable)]);
        hasher.update([u8::from(scheme.require_authored_occurrence)]);
    }
    hasher.update((bindings.len() as u64).to_le_bytes());
    for binding in bindings {
        hasher.update([persistent_name_subject_kind_tag(binding.subject_kind)]);
        hasher.update(binding.kind_id.0.to_le_bytes());
        update_len_bytes(hasher, binding.scheme_id.as_bytes());
    }
}

fn update_len_bytes(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update((bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

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

    /// Composes `additions` with this registry's installed schemes and kind
    /// bindings, then revalidates every binding against the composed kinds.
    ///
    /// Empty additions return `self` unchanged. A scheme identifier already
    /// installed with different content, or a kind already bound to a
    /// scheme, refuses: composition never replaces an installed meaning.
    ///
    /// Complexity: O((S + B) log (S + B)) for S schemes and B bindings.
    pub(crate) fn extend_persistent_name_schemes(
        self,
        additions: InstalledPersistentNameSchemeRegistry,
        bindings: BTreeMap<(PersistentNameSubjectKind, KindId), String>,
    ) -> Result<Self, SchemaRegistryError> {
        if additions.is_empty() && bindings.is_empty() {
            return Ok(self);
        }
        let mut registry = self.persistent_name_schemes.clone();
        for scheme in additions.iter() {
            match registry.get(scheme.scheme_id()) {
                Some(installed) if installed == scheme => {}
                Some(_) => {
                    return Err(SchemaRegistryError::inconsistent_schema_basis(format!(
                        "persistent-name scheme `{}` is already installed with different meaning",
                        scheme.scheme_id()
                    )))
                }
                None => registry
                    .register(scheme.clone())
                    .map_err(SchemaRegistryError::inconsistent_schema_basis)?,
            }
        }
        let mut kind_map = self.kind_name_schemes.clone();
        for (key, scheme_id) in bindings {
            if kind_map.insert(key, scheme_id).is_some() {
                return Err(SchemaRegistryError::inconsistent_schema_basis(format!(
                    "kind {:?} is already bound to a persistent-name scheme",
                    key.1
                )));
            }
        }
        self.install_persistent_name_schemes(registry, kind_map)
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

    pub(crate) fn kind_name_schemes_map(
        &self,
    ) -> &BTreeMap<(PersistentNameSubjectKind, KindId), String> {
        &self.kind_name_schemes
    }

    pub(crate) fn persistent_name_scheme_snapshots(
        &self,
    ) -> Vec<SchemaAuthorityNamingSchemeSnapshot> {
        self.persistent_name_schemes
            .iter()
            .map(|scheme| SchemaAuthorityNamingSchemeSnapshot {
                scheme_id: scheme.scheme_id().to_owned(),
                subject_kind: scheme.subject_kind(),
                subject_name: scheme.subject_name().to_owned(),
                ordered_roles: scheme.ordered_roles(),
                split_capable: scheme.split_capable(),
                require_authored_occurrence: scheme.require_authored_occurrence(),
            })
            .collect()
    }

    pub(crate) fn kind_name_scheme_snapshots(
        &self,
    ) -> Vec<SchemaAuthorityKindNamingBindingSnapshot> {
        self.kind_name_schemes
            .iter()
            .map(|(&(subject_kind, kind_id), scheme_id)| SchemaAuthorityKindNamingBindingSnapshot {
                subject_kind,
                kind_id,
                scheme_id: scheme_id.clone(),
            })
            .collect()
    }

    pub(crate) fn readmit_persistent_name_schemes(
        self,
        schemes: &[SchemaAuthorityNamingSchemeSnapshot],
        bindings: &[SchemaAuthorityKindNamingBindingSnapshot],
    ) -> Result<Self, String> {
        if schemes.is_empty() && bindings.is_empty() {
            return Ok(self);
        }
        if schemes.windows(2).any(|pair| pair[0].scheme_id >= pair[1].scheme_id) {
            return Err("branch-root persistent name schemes are not canonical".to_owned());
        }
        if bindings.windows(2).any(|pair| {
            (pair[0].subject_kind, pair[0].kind_id) >= (pair[1].subject_kind, pair[1].kind_id)
        }) {
            return Err("branch-root persistent name kind bindings are not canonical".to_owned());
        }
        let mut registry = InstalledPersistentNameSchemeRegistry::new();
        for snapshot in schemes {
            let scheme = InstalledPersistentNameScheme::try_from_ordered_roles_for_subject(
                snapshot.scheme_id.clone(),
                snapshot.subject_kind,
                snapshot.subject_name.clone(),
                snapshot.ordered_roles.iter().cloned(),
                snapshot.split_capable,
                snapshot.require_authored_occurrence,
            )?;
            registry.register(scheme)?;
        }
        let mut kind_map = BTreeMap::new();
        for binding in bindings {
            if kind_map
                .insert((binding.subject_kind, binding.kind_id), binding.scheme_id.clone())
                .is_some()
            {
                return Err("duplicate branch-root persistent name kind binding".to_owned());
            }
        }
        self.install_persistent_name_schemes(registry, kind_map)
            .map_err(|error| format!("persistent name scheme readmission denied: {error:?}"))
    }
}

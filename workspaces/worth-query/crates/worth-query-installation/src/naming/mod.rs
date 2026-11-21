//! Installation-time validation and lowering of declared persistent name schemes.
//!
//! Enforces acyclic role-order DAGs, split-discriminator presence, scheme
//! uniqueness, and subject closure before runtime starts, lowering accepted
//! declarations into an immutable [`InstalledNameSchemeCatalog`] ready for
//! Relational minting.

mod validation;

use std::collections::{BTreeMap, BTreeSet};

use worth_query_declaration::facade::application_naming::{
    ApplicationNameSchemeDenial, ApplicationNameSubjectKind, ErasedApplicationNameScheme,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchemaMember, ErasedApplicationSchemaDeclaration,
};

use validation::validate_and_lower_scheme;

/// One topologically ranked role within an [`InstalledNameSchemeLayout`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstalledNameRoleLayout {
    pub(super) role: String,
    pub(super) topological_rank: u32,
    pub(super) depends_on: Vec<String>,
}

impl InstalledNameRoleLayout {
    pub fn role(&self) -> &str {
        &self.role
    }

    pub const fn topological_rank(&self) -> u32 {
        self.topological_rank
    }

    pub fn depends_on(&self) -> &[String] {
        &self.depends_on
    }
}

/// Validated, immutable naming contract lowered for Relational birth minting.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstalledNameSchemeLayout {
    pub(super) scheme_id: String,
    pub(super) subject_kind: ApplicationNameSubjectKind,
    pub(super) subject_name: String,
    pub(super) roles: Vec<InstalledNameRoleLayout>,
    pub(super) role_ranks: BTreeMap<String, u32>,
    pub(super) split_capable: bool,
    pub(super) split_discriminator_identity: Option<String>,
    pub(super) require_authored_occurrence: bool,
    pub(super) layout_digest: [u8; 32],
}

impl InstalledNameSchemeLayout {
    pub fn scheme_id(&self) -> &str {
        &self.scheme_id
    }

    pub const fn subject_kind(&self) -> ApplicationNameSubjectKind {
        self.subject_kind
    }

    pub fn subject_name(&self) -> &str {
        &self.subject_name
    }

    pub fn roles(&self) -> &[InstalledNameRoleLayout] {
        &self.roles
    }

    pub fn role_order(&self) -> impl ExactSizeIterator<Item = &str> {
        self.roles.iter().map(InstalledNameRoleLayout::role)
    }

    pub fn role_rank(&self, role: &str) -> Option<u32> {
        self.role_ranks.get(role).copied()
    }

    pub const fn split_capable(&self) -> bool {
        self.split_capable
    }

    pub fn split_discriminator_identity(&self) -> Option<&str> {
        self.split_discriminator_identity.as_deref()
    }

    pub const fn require_authored_occurrence(&self) -> bool {
        self.require_authored_occurrence
    }

    pub const fn layout_digest(&self) -> &[u8; 32] {
        &self.layout_digest
    }
}

/// Complete installed naming layout catalog for one application schema.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InstalledNameSchemeCatalog {
    by_scheme_id: BTreeMap<String, InstalledNameSchemeLayout>,
    by_subject: BTreeMap<(ApplicationNameSubjectKind, String), String>,
}

impl InstalledNameSchemeCatalog {
    pub fn is_empty(&self) -> bool {
        self.by_scheme_id.is_empty()
    }

    pub fn len(&self) -> usize {
        self.by_scheme_id.len()
    }

    pub fn schemes(&self) -> impl ExactSizeIterator<Item = &InstalledNameSchemeLayout> {
        self.by_scheme_id.values()
    }

    pub fn scheme_by_id(&self, scheme_id: &str) -> Option<&InstalledNameSchemeLayout> {
        self.by_scheme_id.get(scheme_id)
    }

    pub fn scheme_for_subject(
        &self,
        subject_kind: ApplicationNameSubjectKind,
        subject_name: &str,
    ) -> Option<&InstalledNameSchemeLayout> {
        let scheme_id = self
            .by_subject
            .get(&(subject_kind, subject_name.to_owned()))?;
        self.by_scheme_id.get(scheme_id)
    }
}

/// Validates and lowers a slice of declared persistent name schemes into an
/// immutable [`InstalledNameSchemeCatalog`].
///
/// Refuses duplicate scheme IDs or subjects, cyclic role-order DAGs, undeclared
/// role dependencies, and split-capable schemes lacking a split discriminator.
pub fn install_name_schemes(
    schemes: &[ErasedApplicationNameScheme],
) -> Result<InstalledNameSchemeCatalog, ApplicationNameSchemeDenial> {
    let mut by_scheme_id = BTreeMap::new();
    let mut by_subject = BTreeMap::new();

    for scheme in schemes {
        let layout = validate_and_lower_scheme(scheme)?;
        let subject_key = (layout.subject_kind, layout.subject_name.clone());
        if by_scheme_id.contains_key(&layout.scheme_id) || by_subject.contains_key(&subject_key) {
            return Err(ApplicationNameSchemeDenial::DuplicateScheme {
                scheme_id: layout.scheme_id,
                subject: format!(
                    "{}:{}",
                    layout.subject_kind.canonical_name(),
                    layout.subject_name
                ),
            });
        }
        by_subject.insert(subject_key, layout.scheme_id.clone());
        by_scheme_id.insert(layout.scheme_id.clone(), layout);
    }

    Ok(InstalledNameSchemeCatalog {
        by_scheme_id,
        by_subject,
    })
}

/// Validates and lowers every declared name scheme in an application schema,
/// also verifying that entity and relation targets exist in the schema.
pub(crate) fn compile_name_scheme_catalog(
    schema: &ErasedApplicationSchemaDeclaration,
) -> Result<InstalledNameSchemeCatalog, ApplicationNameSchemeDenial> {
    let mut entities = BTreeSet::new();
    let mut relations = BTreeSet::new();
    let mut schemes = Vec::new();

    for member in schema.members() {
        match member {
            ApplicationSchemaMember::Entity { entity } => {
                entities.insert(entity.as_str());
            }
            ApplicationSchemaMember::Relation { relation, .. } => {
                relations.insert(relation.as_str());
            }
            ApplicationSchemaMember::ApplicationNameScheme { definition } => {
                schemes.push(definition.clone());
            }
            _ => {}
        }
    }

    let catalog = install_name_schemes(&schemes)?;
    for layout in catalog.schemes() {
        let missing = match layout.subject_kind() {
            ApplicationNameSubjectKind::Entity => !entities.contains(layout.subject_name()),
            ApplicationNameSubjectKind::Relation => !relations.contains(layout.subject_name()),
            ApplicationNameSubjectKind::Group => false,
        };
        if missing {
            return Err(ApplicationNameSchemeDenial::MissingSubjectTarget {
                scheme_id: layout.scheme_id().to_owned(),
                subject_kind: layout.subject_kind(),
                subject_name: layout.subject_name().to_owned(),
            });
        }
    }
    Ok(catalog)
}

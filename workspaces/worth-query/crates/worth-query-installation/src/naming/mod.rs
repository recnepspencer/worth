//! Installation-time validation and lowering of declared persistent name schemes.
//!
//! Enforces acyclic role-order DAGs, split-discriminator presence, scheme
//! uniqueness, continuity-role validity, and subject closure before runtime
//! starts, lowering accepted declarations into an immutable
//! [`InstalledNameSchemeCatalog`] ready for Relational minting.

mod group_validation;
mod validation;

use std::collections::{BTreeMap, BTreeSet};

use worth_query_declaration::facade::application_naming::{
    ApplicationGroupContinuityRule, ApplicationGroupMemberRole, ApplicationGroupSchemeDenial,
    ApplicationNameSchemeDenial, ApplicationNameSubjectKind, ErasedApplicationGroupNameScheme,
    ErasedApplicationNameScheme,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchemaMember, ErasedApplicationSchemaDeclaration,
};

use group_validation::validate_and_lower_group_scheme;
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
    pub(super) discriminator_referenced_roles: Vec<String>,
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

    pub fn discriminator_referenced_roles(&self) -> &[String] {
        &self.discriminator_referenced_roles
    }

    pub const fn require_authored_occurrence(&self) -> bool {
        self.require_authored_occurrence
    }

    pub const fn layout_digest(&self) -> &[u8; 32] {
        &self.layout_digest
    }
}

/// Validated, immutable group naming contract lowered for Relational aggregate
/// group naming and continuity-rule evaluation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstalledGroupNameSchemeLayout {
    pub(super) name_scheme: InstalledNameSchemeLayout,
    pub(super) continuity_rule: ApplicationGroupContinuityRule,
    pub(super) anchor_role: Option<String>,
    pub(super) boundary_role: Option<String>,
    pub(super) layout_digest: [u8; 32],
}

impl InstalledGroupNameSchemeLayout {
    pub fn scheme_id(&self) -> &str {
        self.name_scheme.scheme_id()
    }

    pub fn group_name(&self) -> &str {
        self.name_scheme.subject_name()
    }

    pub fn name_scheme(&self) -> &InstalledNameSchemeLayout {
        &self.name_scheme
    }

    pub const fn continuity_rule(&self) -> ApplicationGroupContinuityRule {
        self.continuity_rule
    }

    pub fn anchor_role(&self) -> Option<&str> {
        self.anchor_role.as_deref()
    }

    pub fn boundary_role(&self) -> Option<&str> {
        self.boundary_role.as_deref()
    }

    pub fn member_role(&self, role: &str) -> ApplicationGroupMemberRole {
        if self.anchor_role.as_deref() == Some(role) {
            ApplicationGroupMemberRole::Anchor
        } else if self.boundary_role.as_deref() == Some(role) {
            ApplicationGroupMemberRole::Boundary
        } else {
            ApplicationGroupMemberRole::Ordinary
        }
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
    by_group_scheme_id: BTreeMap<String, InstalledGroupNameSchemeLayout>,
    by_group_subject: BTreeMap<String, String>,
}

impl InstalledNameSchemeCatalog {
    pub fn is_empty(&self) -> bool {
        self.by_scheme_id.is_empty() && self.by_group_scheme_id.is_empty()
    }

    pub fn len(&self) -> usize {
        self.by_scheme_id.len() + self.by_group_scheme_id.len()
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

    pub fn group_schemes(&self) -> impl ExactSizeIterator<Item = &InstalledGroupNameSchemeLayout> {
        self.by_group_scheme_id.values()
    }

    pub fn group_scheme_by_id(&self, scheme_id: &str) -> Option<&InstalledGroupNameSchemeLayout> {
        self.by_group_scheme_id.get(scheme_id)
    }

    pub fn group_scheme_for_subject(
        &self,
        group_name: &str,
    ) -> Option<&InstalledGroupNameSchemeLayout> {
        let scheme_id = self.by_group_subject.get(group_name)?;
        self.by_group_scheme_id.get(scheme_id)
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
        by_group_scheme_id: BTreeMap::new(),
        by_group_subject: BTreeMap::new(),
    })
}

/// Validates and lowers a slice of declared persistent group name schemes into
/// an immutable [`InstalledNameSchemeCatalog`].
pub fn install_group_name_schemes(
    schemes: &[ErasedApplicationGroupNameScheme],
) -> Result<InstalledNameSchemeCatalog, ApplicationGroupSchemeDenial> {
    let mut catalog = InstalledNameSchemeCatalog::default();
    insert_group_schemes(&mut catalog, schemes)?;
    Ok(catalog)
}

fn insert_group_schemes(
    catalog: &mut InstalledNameSchemeCatalog,
    group_schemes: &[ErasedApplicationGroupNameScheme],
) -> Result<(), ApplicationGroupSchemeDenial> {
    for scheme in group_schemes {
        let layout = validate_and_lower_group_scheme(scheme)?;
        let scheme_id = layout.scheme_id().to_owned();
        let group_name = layout.group_name().to_owned();
        let subject_key = (ApplicationNameSubjectKind::Group, group_name.clone());
        if catalog.by_scheme_id.contains_key(&scheme_id)
            || catalog.by_group_scheme_id.contains_key(&scheme_id)
            || catalog.by_subject.contains_key(&subject_key)
            || catalog.by_group_subject.contains_key(&group_name)
        {
            return Err(ApplicationGroupSchemeDenial::Scheme(
                ApplicationNameSchemeDenial::DuplicateScheme {
                    scheme_id,
                    subject: format!("group:{group_name}"),
                },
            ));
        }
        catalog
            .by_group_subject
            .insert(group_name, scheme_id.clone());
        catalog.by_group_scheme_id.insert(scheme_id, layout);
    }
    Ok(())
}

/// Validates and lowers every declared entity, relation, and group name scheme
/// in an application schema, verifying subject closure and uniqueness.
pub fn install_schema_name_schemes(
    schema: &ErasedApplicationSchemaDeclaration,
) -> Result<InstalledNameSchemeCatalog, ApplicationGroupSchemeDenial> {
    compile_name_scheme_catalog(schema)
}

/// Validates and lowers every declared entity, relation, and group name scheme
/// in an application schema, verifying subject closure and uniqueness.
pub(crate) fn compile_name_scheme_catalog(
    schema: &ErasedApplicationSchemaDeclaration,
) -> Result<InstalledNameSchemeCatalog, ApplicationGroupSchemeDenial> {
    let mut entities = BTreeSet::new();
    let mut relations = BTreeSet::new();
    let mut schemes = Vec::new();
    let mut group_schemes = Vec::new();

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
            ApplicationSchemaMember::ApplicationGroupNameScheme { definition } => {
                group_schemes.push(definition.clone());
            }
            _ => {}
        }
    }

    let mut catalog = install_name_schemes(&schemes)?;
    insert_group_schemes(&mut catalog, &group_schemes)?;
    for layout in catalog.schemes() {
        let missing = match layout.subject_kind() {
            ApplicationNameSubjectKind::Entity => !entities.contains(layout.subject_name()),
            ApplicationNameSubjectKind::Relation => !relations.contains(layout.subject_name()),
            ApplicationNameSubjectKind::Group => false,
        };
        if missing {
            return Err(ApplicationGroupSchemeDenial::Scheme(
                ApplicationNameSchemeDenial::MissingSubjectTarget {
                    scheme_id: layout.scheme_id().to_owned(),
                    subject_kind: layout.subject_kind(),
                    subject_name: layout.subject_name().to_owned(),
                },
            ));
        }
    }
    Ok(catalog)
}

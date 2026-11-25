use crate::archive_decode_authority::ArchiveDecodeAuthority;
use std::fmt;
use std::marker::PhantomData;

use crate::application_schema::{ApplicationEntityMarkerIdentity, ApplicationRelationRef};

use super::denial::{
    validate_discriminator_identity, validate_role_name, validate_scheme_identifier,
    validate_subject_name,
};
use super::{ApplicationNameDiscriminator, ApplicationNameSchemeDenial};

/// Subject family governed by an [`ApplicationNameScheme`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ApplicationNameSubjectKind {
    Entity,
    Relation,
    Group,
}

impl ApplicationNameSubjectKind {
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::Entity => "entity",
            Self::Relation => "relation",
            Self::Group => "group",
        }
    }
}

/// Directed same-occurrence role reference edge: `role` references a name minted
/// by `depends_on` within the same occurrence.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ApplicationNameRoleDependency {
    role: String,
    depends_on: String,
}

impl ApplicationNameRoleDependency {
    pub fn new(role: impl Into<String>, depends_on: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            depends_on: depends_on.into(),
        }
    }

    pub fn role(&self) -> &str {
        &self.role
    }

    pub fn depends_on(&self) -> &str {
        &self.depends_on
    }
}

/// Portable, type-erased name scheme declaration carried inside an
/// `ApplicationSchemaDeclaration` and validated/lowered at installation.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ErasedApplicationNameScheme {
    scheme_id: String,
    subject_kind: ApplicationNameSubjectKind,
    subject_name: String,
    role_order: Vec<String>,
    role_dependencies: Vec<ApplicationNameRoleDependency>,
    split_capable: bool,
    split_discriminator_identity: Option<String>,
    discriminator_referenced_roles: Vec<String>,
    require_authored_occurrence: bool,
}

impl ErasedApplicationNameScheme {
    #[allow(clippy::too_many_arguments)]
    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        scheme_id: String,
        subject_kind: ApplicationNameSubjectKind,
        subject_name: String,
        role_order: Vec<String>,
        role_dependencies: Vec<ApplicationNameRoleDependency>,
        split_capable: bool,
        split_discriminator_identity: Option<String>,
        discriminator_referenced_roles: Vec<String>,
        require_authored_occurrence: bool,
    ) -> Self {
        Self {
            scheme_id,
            subject_kind,
            subject_name,
            role_order,
            role_dependencies,
            split_capable,
            split_discriminator_identity,
            discriminator_referenced_roles,
            require_authored_occurrence,
        }
    }

    pub fn scheme_id(&self) -> &str {
        &self.scheme_id
    }

    pub const fn subject_kind(&self) -> ApplicationNameSubjectKind {
        self.subject_kind
    }

    pub fn subject_name(&self) -> &str {
        &self.subject_name
    }

    pub fn role_order(&self) -> &[String] {
        &self.role_order
    }

    pub fn role_dependencies(&self) -> &[ApplicationNameRoleDependency] {
        &self.role_dependencies
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
}

/// Schema-typed persistent name scheme declaration for one entity, relation, or
/// group kind `K` in schema `S`.
pub struct ApplicationNameScheme<S, K> {
    erased: ErasedApplicationNameScheme,
    _marker: PhantomData<fn() -> (S, K)>,
}

impl<S, K> Clone for ApplicationNameScheme<S, K> {
    fn clone(&self) -> Self {
        Self {
            erased: self.erased.clone(),
            _marker: PhantomData,
        }
    }
}

impl<S, K> fmt::Debug for ApplicationNameScheme<S, K> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ApplicationNameScheme")
            .field("erased", &self.erased)
            .finish_non_exhaustive()
    }
}

impl<S, K> PartialEq for ApplicationNameScheme<S, K> {
    fn eq(&self, other: &Self) -> bool {
        self.erased == other.erased
    }
}

impl<S, K> Eq for ApplicationNameScheme<S, K> {}

impl<S, K> ApplicationNameScheme<S, K> {
    pub fn erased(&self) -> &ErasedApplicationNameScheme {
        &self.erased
    }

    pub fn into_erased(self) -> ErasedApplicationNameScheme {
        self.erased
    }
}

/// Builder for declaring a persistent naming policy for entity, relation, or
/// group kind `K` inside schema `S`.
pub struct ApplicationNameSchemeBuilder<S, K> {
    scheme_id: String,
    subject_kind: ApplicationNameSubjectKind,
    subject_name: String,
    role_order: Vec<String>,
    role_dependencies: Vec<ApplicationNameRoleDependency>,
    split_capable: bool,
    split_discriminator_identity: Option<String>,
    discriminator_referenced_roles: Vec<String>,
    require_authored_occurrence: bool,
    _marker: PhantomData<fn() -> (S, K)>,
}

impl<S, K> ApplicationNameSchemeBuilder<S, K>
where
    K: ApplicationEntityMarkerIdentity<S>,
{
    /// Starts a persistent name scheme builder for declared entity kind `K`.
    pub fn new(scheme_id: impl Into<String>) -> Self {
        Self::with_subject(scheme_id, ApplicationNameSubjectKind::Entity, K::IDENTIFIER)
    }
}

impl<S, K> ApplicationNameSchemeBuilder<S, K> {
    /// Starts a persistent name scheme builder for a declared relation kind.
    pub fn for_relation<From, To>(
        scheme_id: impl Into<String>,
        relation: ApplicationRelationRef<S, K, From, To>,
    ) -> Self {
        Self::with_subject(
            scheme_id,
            ApplicationNameSubjectKind::Relation,
            relation.name(),
        )
    }

    /// Starts a persistent name scheme builder for a named group kind.
    pub fn for_group(scheme_id: impl Into<String>, group_name: impl Into<String>) -> Self {
        Self::with_subject(scheme_id, ApplicationNameSubjectKind::Group, group_name)
    }

    fn with_subject(
        scheme_id: impl Into<String>,
        subject_kind: ApplicationNameSubjectKind,
        subject_name: impl Into<String>,
    ) -> Self {
        Self {
            scheme_id: scheme_id.into(),
            subject_kind,
            subject_name: subject_name.into(),
            role_order: Vec::new(),
            role_dependencies: Vec::new(),
            split_capable: false,
            split_discriminator_identity: None,
            discriminator_referenced_roles: Vec::new(),
            require_authored_occurrence: false,
            _marker: PhantomData,
        }
    }

    /// Declares the ordered roles emitted under this scheme.
    ///
    /// Earlier roles precede later roles in the same occurrence.
    pub fn role_order(mut self, roles: &[&str]) -> Self {
        self.role_order = roles.iter().map(|role| (*role).to_owned()).collect();
        self
    }

    /// Declares an explicit same-occurrence dependency stating that `role`
    /// references a name minted under `depends_on`.
    pub fn role_depends_on(
        mut self,
        role: impl Into<String>,
        depends_on: impl Into<String>,
    ) -> Self {
        self.role_dependencies
            .push(ApplicationNameRoleDependency::new(role, depends_on));
        self
    }

    /// Marks this scheme as split-capable.
    ///
    /// Installation refuses any split-capable scheme that does not also declare
    /// a [`Self::split_discriminator`].
    pub fn allow_split(mut self) -> Self {
        self.split_capable = true;
        self
    }

    /// Declares the typed split discriminator for this scheme and marks it
    /// split-capable.
    pub fn split_discriminator<D: ApplicationNameDiscriminator>(mut self) -> Self {
        self.split_capable = true;
        self.split_discriminator_identity = Some(D::DISCRIMINATOR_IDENTITY.to_owned());
        self.discriminator_referenced_roles = D::referenced_roles()
            .iter()
            .map(|role| (*role).to_owned())
            .collect();
        self
    }

    /// Requires every birth under this scheme to carry an authored occurrence
    /// key from an authored identity family (never a bare operation identity).
    pub fn require_authored_occurrence(mut self) -> Self {
        self.require_authored_occurrence = true;
        self
    }

    /// Materializes the descriptive scheme declaration after checking basic
    /// identifier syntax.
    ///
    /// Acyclicity of the role DAG, split-discriminator presence, subject
    /// closure, and cross-scheme uniqueness are enforced at installation.
    pub fn build(self) -> Result<ApplicationNameScheme<S, K>, ApplicationNameSchemeDenial> {
        validate_scheme_identifier(&self.scheme_id)?;
        validate_subject_name(&self.scheme_id, &self.subject_name)?;
        if self.role_order.is_empty() {
            return Err(ApplicationNameSchemeDenial::EmptyRoleOrder {
                scheme_id: self.scheme_id,
            });
        }
        for role in &self.role_order {
            validate_role_name(&self.scheme_id, role)?;
        }
        for dependency in &self.role_dependencies {
            validate_role_name(&self.scheme_id, dependency.role())?;
            validate_role_name(&self.scheme_id, dependency.depends_on())?;
        }
        if let Some(disc_id) = &self.split_discriminator_identity {
            validate_discriminator_identity(&self.scheme_id, disc_id)?;
        }
        Ok(ApplicationNameScheme {
            erased: ErasedApplicationNameScheme {
                scheme_id: self.scheme_id,
                subject_kind: self.subject_kind,
                subject_name: self.subject_name,
                role_order: self.role_order,
                role_dependencies: self.role_dependencies,
                split_capable: self.split_capable,
                split_discriminator_identity: self.split_discriminator_identity,
                discriminator_referenced_roles: self.discriminator_referenced_roles,
                require_authored_occurrence: self.require_authored_occurrence,
            },
            _marker: PhantomData,
        })
    }
}

pub(crate) fn validate_erased_scheme_identifiers(
    definition: &ErasedApplicationNameScheme,
) -> Result<(), crate::application_schema::ApplicationSchemaDeclarationDenial> {
    validate_scheme_identifier(definition.scheme_id())
        .and_then(|()| validate_subject_name(definition.scheme_id(), definition.subject_name()))
        .and_then(|()| {
            for role in definition.role_order() {
                validate_role_name(definition.scheme_id(), role)?;
            }
            for dependency in definition.role_dependencies() {
                validate_role_name(definition.scheme_id(), dependency.role())?;
                validate_role_name(definition.scheme_id(), dependency.depends_on())?;
            }
            for role in definition.discriminator_referenced_roles() {
                validate_role_name(definition.scheme_id(), role)?;
            }
            if let Some(disc_id) = definition.split_discriminator_identity() {
                validate_discriminator_identity(definition.scheme_id(), disc_id)?;
            }
            Ok(())
        })
        .map_err(|_| {
            crate::application_schema::ApplicationSchemaDeclarationDenial::InvalidIdentifier
        })
}

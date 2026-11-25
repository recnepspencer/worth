//! Group name schemes: one group kind, its continuity rule, and its roles.
//!
//! A group's persistent name is derived by Relational from `(scheme, birth
//! anchor name, continuity rule)` and never from allocation order. This module
//! declares which rule a group kind follows and which declared roles carry the
//! anchor (or producer) and boundary meaning that rule reads.
use crate::archive_decode_authority::ArchiveDecodeAuthority;
use std::fmt;
use std::marker::PhantomData;

use super::{
    ApplicationNameScheme, ApplicationNameSchemeBuilder, ApplicationNameSchemeDenial,
    ErasedApplicationNameScheme,
};

/// Closed consumer-driven continuity rule for group split/merge keeper selection.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum ApplicationGroupContinuityRule {
    /// Follows its anchor carrier (e.g., a loop follows its host face).
    Anchor,
    /// Follows its enclosing boundary members (e.g., a space follows its walls).
    Boundary,
    /// Follows its owning producer role (e.g., a shell follows its owning body feature).
    ProducerRole,
}

impl ApplicationGroupContinuityRule {
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::Anchor => "anchor",
            Self::Boundary => "boundary",
            Self::ProducerRole => "producer-role",
        }
    }

    /// Canonical byte Relational hashes into the v2 group identity domain.
    pub const fn canonical_tag(self) -> u8 {
        match self {
            Self::Anchor => 1,
            Self::Boundary => 2,
            Self::ProducerRole => 3,
        }
    }
}

/// Continuity meaning of one member's declared role within its group.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum ApplicationGroupMemberRole {
    Ordinary,
    Anchor,
    Boundary,
}

/// Typed refusal of a group scheme declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationGroupSchemeDenial {
    /// The underlying name scheme violates a naming law.
    Scheme(ApplicationNameSchemeDenial),
    /// The rule needs a role the scheme does not declare.
    MissingContinuityRole {
        scheme_id: String,
        rule: ApplicationGroupContinuityRule,
    },
    /// A continuity role is not one of the scheme's declared roles.
    UndeclaredContinuityRole { scheme_id: String, role: String },
    /// One role was declared as both anchor and boundary.
    AmbiguousContinuityRole { scheme_id: String, role: String },
}

impl From<ApplicationNameSchemeDenial> for ApplicationGroupSchemeDenial {
    fn from(denial: ApplicationNameSchemeDenial) -> Self {
        Self::Scheme(denial)
    }
}

impl fmt::Display for ApplicationGroupSchemeDenial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scheme(denial) => denial.fmt(formatter),
            Self::MissingContinuityRole { scheme_id, rule } => write!(
                formatter,
                "group scheme `{scheme_id}` follows `{}` but declares no role for it",
                rule.canonical_name()
            ),
            Self::UndeclaredContinuityRole { scheme_id, role } => write!(
                formatter,
                "group scheme `{scheme_id}` names undeclared continuity role `{role}`"
            ),
            Self::AmbiguousContinuityRole { scheme_id, role } => write!(
                formatter,
                "group scheme `{scheme_id}` uses role `{role}` as both anchor and boundary"
            ),
        }
    }
}

impl std::error::Error for ApplicationGroupSchemeDenial {}

/// Portable validated group scheme: a group name scheme plus continuity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ErasedApplicationGroupNameScheme {
    name_scheme: ErasedApplicationNameScheme,
    continuity_rule: ApplicationGroupContinuityRule,
    anchor_role: Option<String>,
    boundary_role: Option<String>,
}

impl ErasedApplicationGroupNameScheme {
    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        name_scheme: ErasedApplicationNameScheme,
        continuity_rule: ApplicationGroupContinuityRule,
        anchor_role: Option<String>,
        boundary_role: Option<String>,
    ) -> Self {
        Self {
            name_scheme,
            continuity_rule,
            anchor_role,
            boundary_role,
        }
    }

    pub fn name_scheme(&self) -> &ErasedApplicationNameScheme {
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

    /// The continuity meaning of a member carrying declared `role`.
    pub fn member_role(&self, role: &str) -> ApplicationGroupMemberRole {
        if self.anchor_role.as_deref() == Some(role) {
            ApplicationGroupMemberRole::Anchor
        } else if self.boundary_role.as_deref() == Some(role) {
            ApplicationGroupMemberRole::Boundary
        } else {
            ApplicationGroupMemberRole::Ordinary
        }
    }
}

/// Schema-typed group scheme for group kind `G` in schema `S`.
pub struct ApplicationGroupNameScheme<S, G> {
    erased: ErasedApplicationGroupNameScheme,
    _marker: PhantomData<fn() -> (S, G)>,
}

impl<S, G> Clone for ApplicationGroupNameScheme<S, G> {
    fn clone(&self) -> Self {
        Self {
            erased: self.erased.clone(),
            _marker: PhantomData,
        }
    }
}

impl<S, G> fmt::Debug for ApplicationGroupNameScheme<S, G> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ApplicationGroupNameScheme")
            .field("erased", &self.erased)
            .finish_non_exhaustive()
    }
}

impl<S, G> PartialEq for ApplicationGroupNameScheme<S, G> {
    fn eq(&self, other: &Self) -> bool {
        self.erased == other.erased
    }
}

impl<S, G> Eq for ApplicationGroupNameScheme<S, G> {}

impl<S, G> ApplicationGroupNameScheme<S, G> {
    pub fn erased(&self) -> &ErasedApplicationGroupNameScheme {
        &self.erased
    }

    pub fn into_erased(self) -> ErasedApplicationGroupNameScheme {
        self.erased
    }
}

/// Builder for a group kind's persistent naming and continuity policy.
pub struct ApplicationGroupNameSchemeBuilder<S, G> {
    scheme: ApplicationNameSchemeBuilder<S, G>,
    continuity_rule: ApplicationGroupContinuityRule,
    anchor_role: Option<String>,
    boundary_role: Option<String>,
}

impl<S, G> ApplicationGroupNameSchemeBuilder<S, G> {
    /// Starts a group scheme for `group_name` following `continuity_rule`.
    pub fn new(
        scheme_id: impl Into<String>,
        group_name: impl Into<String>,
        continuity_rule: ApplicationGroupContinuityRule,
    ) -> Self {
        Self {
            scheme: ApplicationNameSchemeBuilder::for_group(scheme_id, group_name),
            continuity_rule,
            anchor_role: None,
            boundary_role: None,
        }
    }

    /// Declares the ordered member roles of this group kind.
    pub fn role_order(mut self, roles: &[&str]) -> Self {
        self.scheme = self.scheme.role_order(roles);
        self
    }

    /// Declares the role whose least-named member anchors a newborn group;
    /// under [`ApplicationGroupContinuityRule::ProducerRole`] it is the producer.
    pub fn anchor_role(mut self, role: impl Into<String>) -> Self {
        self.anchor_role = Some(role.into());
        self
    }

    /// Declares the enclosing role read by [`ApplicationGroupContinuityRule::Boundary`].
    pub fn boundary_role(mut self, role: impl Into<String>) -> Self {
        self.boundary_role = Some(role.into());
        self
    }

    /// Requires an authored occurrence key on every group birth.
    pub fn require_authored_occurrence(mut self) -> Self {
        self.scheme = self.scheme.require_authored_occurrence();
        self
    }

    /// Validates the scheme, its roles, and the continuity rule's needs.
    pub fn build(self) -> Result<ApplicationGroupNameScheme<S, G>, ApplicationGroupSchemeDenial> {
        let name_scheme: ApplicationNameScheme<S, G> = self.scheme.build()?;
        let name_scheme = name_scheme.into_erased();
        let scheme_id = name_scheme.scheme_id().to_owned();
        for role in [&self.anchor_role, &self.boundary_role]
            .into_iter()
            .flatten()
        {
            if !name_scheme.role_order().contains(role) {
                return Err(ApplicationGroupSchemeDenial::UndeclaredContinuityRole {
                    scheme_id,
                    role: role.clone(),
                });
            }
        }
        if let (Some(anchor), Some(boundary)) = (&self.anchor_role, &self.boundary_role)
            && anchor == boundary
        {
            return Err(ApplicationGroupSchemeDenial::AmbiguousContinuityRole {
                scheme_id,
                role: anchor.clone(),
            });
        }
        let required = match self.continuity_rule {
            ApplicationGroupContinuityRule::Anchor => true,
            ApplicationGroupContinuityRule::Boundary => self.boundary_role.is_some(),
            ApplicationGroupContinuityRule::ProducerRole => self.anchor_role.is_some(),
        };
        if !required {
            return Err(ApplicationGroupSchemeDenial::MissingContinuityRole {
                scheme_id,
                rule: self.continuity_rule,
            });
        }
        Ok(ApplicationGroupNameScheme {
            erased: ErasedApplicationGroupNameScheme {
                name_scheme,
                continuity_rule: self.continuity_rule,
                anchor_role: self.anchor_role,
                boundary_role: self.boundary_role,
            },
            _marker: PhantomData,
        })
    }
}

pub(crate) fn validate_erased_group_scheme_identifiers(
    definition: &ErasedApplicationGroupNameScheme,
) -> Result<(), crate::application_schema::ApplicationSchemaDeclarationDenial> {
    super::validate_erased_scheme_identifiers(definition.name_scheme())?;
    let scheme_id = definition.name_scheme().scheme_id();
    for role in [definition.anchor_role(), definition.boundary_role()]
        .into_iter()
        .flatten()
    {
        super::validate_role_name(scheme_id, role).map_err(|_| {
            crate::application_schema::ApplicationSchemaDeclarationDenial::InvalidIdentifier
        })?;
    }
    Ok(())
}

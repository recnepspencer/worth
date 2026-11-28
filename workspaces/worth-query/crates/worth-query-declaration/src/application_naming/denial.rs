use std::fmt;

use super::ApplicationNameSubjectKind;

/// Typed refusal produced when an [`ApplicationNameScheme`](super::ApplicationNameScheme)
/// violates declaration or installation naming laws.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationNameSchemeDenial {
    /// Scheme identifier is empty or contains invalid characters.
    InvalidSchemeIdentifier { scheme_id: String },
    /// Subject name is empty or invalid.
    InvalidSubjectName {
        scheme_id: String,
        subject_name: String,
    },
    /// Scheme declares no roles.
    EmptyRoleOrder { scheme_id: String },
    /// A role name is empty or contains invalid characters.
    InvalidRoleName { scheme_id: String, role: String },
    /// Role precedence and/or role dependency edges contain a cycle.
    CyclicRoleOrder {
        scheme_id: String,
        cycle: Vec<String>,
    },
    /// A role dependency references a role not declared in the scheme.
    UndeclaredRoleDependency {
        scheme_id: String,
        role: String,
        depends_on: String,
    },
    /// A split-capable scheme lacks a declared split discriminator type.
    MissingSplitDiscriminator { scheme_id: String },
    /// A declared split discriminator identity string is empty or malformed.
    InvalidDiscriminatorIdentity {
        scheme_id: String,
        discriminator_identity: String,
    },
    /// Two schemes share a scheme identifier or govern the same subject.
    DuplicateScheme { scheme_id: String, subject: String },
    /// The scheme targets an entity or relation not declared in the schema.
    MissingSubjectTarget {
        scheme_id: String,
        subject_kind: ApplicationNameSubjectKind,
        subject_name: String,
    },
}

impl fmt::Display for ApplicationNameSchemeDenial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSchemeIdentifier { scheme_id } => {
                write!(formatter, "invalid name scheme identifier `{scheme_id}`")
            }
            Self::InvalidSubjectName {
                scheme_id,
                subject_name,
            } => write!(
                formatter,
                "invalid subject name `{subject_name}` in name scheme `{scheme_id}`"
            ),
            Self::EmptyRoleOrder { scheme_id } => {
                write!(formatter, "name scheme `{scheme_id}` declares no roles")
            }
            Self::InvalidRoleName { scheme_id, role } => {
                write!(
                    formatter,
                    "invalid role name `{role}` in name scheme `{scheme_id}`"
                )
            }
            Self::CyclicRoleOrder { scheme_id, cycle } => write!(
                formatter,
                "cyclic role order in name scheme `{scheme_id}`: {}",
                cycle.join(" -> ")
            ),
            Self::UndeclaredRoleDependency {
                scheme_id,
                role,
                depends_on,
            } => write!(
                formatter,
                "undeclared role dependency `{role}` -> `{depends_on}` in name scheme `{scheme_id}`"
            ),
            Self::MissingSplitDiscriminator { scheme_id } => write!(
                formatter,
                "split-capable name scheme `{scheme_id}` is missing a split discriminator"
            ),
            Self::InvalidDiscriminatorIdentity {
                scheme_id,
                discriminator_identity,
            } => write!(
                formatter,
                "invalid split discriminator identity `{discriminator_identity}` in name scheme `{scheme_id}`"
            ),
            Self::DuplicateScheme { scheme_id, subject } => write!(
                formatter,
                "duplicate name scheme `{scheme_id}` for subject `{subject}`"
            ),
            Self::MissingSubjectTarget {
                scheme_id,
                subject_kind,
                subject_name,
            } => write!(
                formatter,
                "name scheme `{scheme_id}` targets undeclared {} `{subject_name}`",
                subject_kind.canonical_name()
            ),
        }
    }
}

impl std::error::Error for ApplicationNameSchemeDenial {}

/// Validates a dotted scheme identifier (e.g. `worth.example.face.v1`).
pub fn validate_scheme_identifier(scheme_id: &str) -> Result<(), ApplicationNameSchemeDenial> {
    if !worth_foundational::is_valid_dotted_identifier(scheme_id) {
        return Err(ApplicationNameSchemeDenial::InvalidSchemeIdentifier {
            scheme_id: scheme_id.to_owned(),
        });
    }
    Ok(())
}

pub(crate) fn validate_subject_name(
    scheme_id: &str,
    subject_name: &str,
) -> Result<(), ApplicationNameSchemeDenial> {
    if !worth_foundational::is_valid_simple_identifier(subject_name) {
        return Err(ApplicationNameSchemeDenial::InvalidSubjectName {
            scheme_id: scheme_id.to_owned(),
            subject_name: subject_name.to_owned(),
        });
    }
    Ok(())
}

/// Validates a single role name inside a name scheme.
pub fn validate_role_name(scheme_id: &str, role: &str) -> Result<(), ApplicationNameSchemeDenial> {
    if !worth_foundational::is_valid_role_identifier(role) {
        return Err(ApplicationNameSchemeDenial::InvalidRoleName {
            scheme_id: scheme_id.to_owned(),
            role: role.to_owned(),
        });
    }
    Ok(())
}

/// Validates a declared split discriminator identity string.
pub fn validate_discriminator_identity(
    scheme_id: &str,
    discriminator_identity: &str,
) -> Result<(), ApplicationNameSchemeDenial> {
    if !worth_foundational::is_valid_role_identifier(discriminator_identity) {
        return Err(ApplicationNameSchemeDenial::InvalidDiscriminatorIdentity {
            scheme_id: scheme_id.to_owned(),
            discriminator_identity: discriminator_identity.to_owned(),
        });
    }
    Ok(())
}

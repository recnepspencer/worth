//! Why a declared ReBAC rule refused, in the words its author wrote.
use super::{ApplicationSchemaDeclarationDenial, ApplicationValueEncodeDenial};

/// Each variant names the authored condition or path (for example
/// `User -UserAssignments-> Assignment [Assignment.role one of [...]]`), never
/// a lowered path index.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationAuthorizationRuleDenial {
    /// The condition's field is not part of its record.
    FieldUnavailable {
        condition: String,
        cause: Box<ApplicationSchemaDeclarationDenial>,
    },
    /// The condition's field is not equality-queryable, so the kernel cannot read it.
    FieldNotQueryable { condition: String },
    /// The field's binding refused a condition value.
    ValueRejected {
        condition: String,
        cause: ApplicationValueEncodeDenial,
    },
    /// `one_of` with no values would match nothing.
    EmptyOneOf { condition: String },
    /// The authored path lowers to more kernel paths than one path may install.
    ExpansionLimit {
        path: String,
        expanding: Vec<String>,
        lowered: usize,
        limit: usize,
    },
    /// A deny condition reads an optional field. An absent value never
    /// matches, so the deny would silently fail open.
    DenyConditionOnOptionalField { path: String, condition: String },
    /// The rule has no allow path, so it would admit nobody.
    NoAllowPath,
    /// No role of the matrix grants this ability; installing it would admit nobody.
    RoleLessAbility { ability: String },
}

impl std::fmt::Display for ApplicationAuthorizationRuleDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FieldUnavailable { condition, cause } => {
                write!(formatter, "`{condition}`: field unavailable: {cause}")
            }
            Self::FieldNotQueryable { condition } => {
                write!(formatter, "`{condition}`: field is not equality-queryable")
            }
            Self::ValueRejected { condition, cause } => {
                write!(formatter, "`{condition}`: value rejected: {cause:?}")
            }
            Self::EmptyOneOf { condition } => {
                write!(formatter, "`{condition}`: one_of needs at least one value")
            }
            Self::ExpansionLimit {
                path,
                expanding,
                lowered,
                limit,
            } => write!(
                formatter,
                "`{path}` lowers to {lowered} paths (limit {limit}); expanding conditions: {}",
                expanding.join("; ")
            ),
            Self::DenyConditionOnOptionalField { path, condition } => write!(
                formatter,
                "deny path `{path}` reads optional `{condition}` and would fail open when absent"
            ),
            Self::NoAllowPath => formatter.write_str("the rule has no allow path"),
            Self::RoleLessAbility { ability } => {
                write!(formatter, "no role grants ability `{ability}`")
            }
        }
    }
}

impl std::error::Error for ApplicationAuthorizationRuleDenial {}

impl From<ApplicationAuthorizationRuleDenial> for ApplicationSchemaDeclarationDenial {
    fn from(denial: ApplicationAuthorizationRuleDenial) -> Self {
        Self::InvalidAuthorizationRule(denial)
    }
}

//! Typed refusals of namespace roots, overrides and derivations.
use super::ApplicationIdentityRole;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationIdentityNamespaceDenial {
    /// The root is empty, has an empty segment, or a byte outside printable ASCII.
    InvalidRoot,
    /// The override is not a dotted identity of printable ASCII.
    InvalidOverride { role: ApplicationIdentityRole },
    /// The slot field is empty, contains `.`, or a byte outside printable ASCII.
    InvalidSlotField,
    /// A derived identity or override would exceed the identity bound.
    Overlong { required: usize, maximum: usize },
    /// The role already carries an explicit override.
    DuplicateOverride { role: ApplicationIdentityRole },
    /// The override equals the root, another role's identity, or falls in
    /// the namespace's slot space.
    OverrideCollision { role: ApplicationIdentityRole },
}

impl ApplicationIdentityNamespaceDenial {
    /// A static explanation, also used for compile-time refusals.
    pub const fn message(&self) -> &'static str {
        match self {
            Self::InvalidRoot => "identity namespace root must be non-empty dotted printable ASCII",
            Self::InvalidOverride { .. } => {
                "identity override must be non-empty dotted printable ASCII"
            }
            Self::InvalidSlotField => {
                "identity slot field must be one non-empty printable ASCII segment without `.`"
            }
            Self::Overlong { .. } => "derived identity exceeds the application identity bound",
            Self::DuplicateOverride { .. } => "identity role already has an explicit override",
            Self::OverrideCollision { .. } => {
                "identity override collides with the root, another role or the slot space"
            }
        }
    }
}

impl std::fmt::Display for ApplicationIdentityNamespaceDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidOverride { role }
            | Self::DuplicateOverride { role }
            | Self::OverrideCollision { role } => {
                write!(formatter, "{} (role `{}`)", self.message(), role.segment())
            }
            Self::Overlong { required, maximum } => write!(
                formatter,
                "{}: {required} bytes, maximum {maximum}",
                self.message()
            ),
            Self::InvalidRoot | Self::InvalidSlotField => formatter.write_str(self.message()),
        }
    }
}

impl std::error::Error for ApplicationIdentityNamespaceDenial {}

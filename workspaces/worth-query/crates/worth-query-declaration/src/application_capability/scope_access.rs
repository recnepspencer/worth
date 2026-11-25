//! The typed access level of a scope grant.

/// Access level granted to a collaborator on a single application scope.
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum ScopeAccess {
    ReadOnly,
    ReadAndWatch,
}

impl ScopeAccess {
    pub const fn read_only() -> Self {
        Self::ReadOnly
    }

    pub const fn read_and_watch() -> Self {
        Self::ReadAndWatch
    }

    pub const fn allows_read(self) -> bool {
        match self {
            Self::ReadOnly | Self::ReadAndWatch => true,
        }
    }

    pub const fn allows_watch(self) -> bool {
        match self {
            Self::ReadOnly => false,
            Self::ReadAndWatch => true,
        }
    }

    pub const fn wire_code(self) -> u8 {
        match self {
            Self::ReadOnly => 1,
            Self::ReadAndWatch => 2,
        }
    }

    pub const fn from_wire_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::ReadOnly),
            2 => Some(Self::ReadAndWatch),
            _ => None,
        }
    }
}

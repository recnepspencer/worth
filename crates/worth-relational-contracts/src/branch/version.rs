use serde::{Deserialize, Serialize};

/// Truth-bearing version local to one runtime-affine branch reference.
///
/// This is deliberately distinct from the runtime-wide commit/version
/// allocator.  Equal branch versions on different references are expected;
/// they do not establish currentness across branches or runtimes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RelationalBranchVersion(u64);

impl RelationalBranchVersion {
    pub const fn initial() -> Self {
        Self(0)
    }

    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

/// Operations used by the Relational owners.
pub trait RelationalBranchVersionOwnerPort: Sized {
    fn checked_advance(self) -> Option<Self>;
}

impl RelationalBranchVersionOwnerPort for RelationalBranchVersion {
    fn checked_advance(self) -> Option<Self> {
        self.0.checked_add(1).map(Self)
    }
}

use super::PhysicalByteGuardScope;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteGuardReleaseReceipt {
    scope: PhysicalByteGuardScope,
    guarded_bytes: u64,
}

impl ByteGuardReleaseReceipt {
    pub(crate) const fn new(scope: PhysicalByteGuardScope, guarded_bytes: u64) -> Self {
        Self {
            scope,
            guarded_bytes,
        }
    }

    pub const fn scope(self) -> PhysicalByteGuardScope {
        self.scope
    }

    pub const fn guarded_bytes(self) -> u64 {
        self.guarded_bytes
    }
}

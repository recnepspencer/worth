use super::super::StoreSecurityScopePropagationDenial;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoverySecurityScopePropagationDenial {
    store_denial: StoreSecurityScopePropagationDenial,
}

impl RecoverySecurityScopePropagationDenial {
    pub const fn from_store_denial(store_denial: StoreSecurityScopePropagationDenial) -> Self {
        Self { store_denial }
    }

    pub const fn store_denial(self) -> StoreSecurityScopePropagationDenial {
        self.store_denial
    }
}

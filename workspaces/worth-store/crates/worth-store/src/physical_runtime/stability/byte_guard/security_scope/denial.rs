use worth_store_security::StoreSecurityScopePropagationDenial;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StableReadSecurityScopePropagationDenial {
    store_denial: StoreSecurityScopePropagationDenial,
}

impl StableReadSecurityScopePropagationDenial {
    pub const fn from_store_denial(store_denial: StoreSecurityScopePropagationDenial) -> Self {
        Self { store_denial }
    }

    pub const fn store_denial(self) -> StoreSecurityScopePropagationDenial {
        self.store_denial
    }
}

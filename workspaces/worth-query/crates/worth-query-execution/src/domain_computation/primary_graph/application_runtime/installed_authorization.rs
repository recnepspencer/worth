//! Funded immutable custody of the compiled application authorization vocabulary.
use crate::domain_computation::authorization::WorthQueryInstalledAuthorizationRegistry;
use worth_execution::{
    ExecutionAllocationDenial, ExecutionAllocationPolicy, ExecutionArray, ExecutionArrayBuilder,
};

/// Keeps the compiled Signal graph out of by-value application runtime transfers.
/// This owner is neither cloneable admission nor mutable authorization state.
pub(in crate::domain_computation) struct InstalledApplicationAuthorization {
    registry: ExecutionArray<WorthQueryInstalledAuthorizationRegistry>,
}

impl InstalledApplicationAuthorization {
    pub(super) fn capture(
        registry: WorthQueryInstalledAuthorizationRegistry,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, ExecutionAllocationDenial> {
        let mut backing = ExecutionArrayBuilder::allocate(1, policy)?;
        backing.push(registry)?;
        Ok(Self {
            registry: backing.seal()?,
        })
    }
}

impl std::ops::Deref for InstalledApplicationAuthorization {
    type Target = WorthQueryInstalledAuthorizationRegistry;

    fn deref(&self) -> &Self::Target {
        &self.registry[0]
    }
}

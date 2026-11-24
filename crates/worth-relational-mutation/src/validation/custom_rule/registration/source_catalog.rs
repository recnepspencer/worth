//! Borrow the rule registration’s declared managed source programs.
use super::{CustomInvariantRegistration, ManagedIndexSourceProgram};

#[doc(hidden)]
pub trait CustomInvariantSourceCatalogOwnerPort {
    fn managed_sources(&self) -> &[ManagedIndexSourceProgram];
}

impl CustomInvariantSourceCatalogOwnerPort for CustomInvariantRegistration {
    fn managed_sources(&self) -> &[ManagedIndexSourceProgram] {
        &self.managed_sources
    }
}

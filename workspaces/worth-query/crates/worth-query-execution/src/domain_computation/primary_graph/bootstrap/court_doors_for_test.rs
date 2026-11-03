//! Sealed test doors over this module's private members.
use super::*;

impl<Schema> WorthQueryPrimaryGraphBootstrap<Schema> {
    #[doc(hidden)]
    pub fn managed_indexes_for_test(&self) -> &super::super::application_query::managed_index::ManagedIndexProviderCatalog<Schema> {
        &self.managed_indexes
    }
}

impl<Schema> WorthQueryPrimaryGraphBootstrap<Schema> {
    #[doc(hidden)]
    pub fn managed_indexes_mut_for_test(&mut self) -> &mut super::super::application_query::managed_index::ManagedIndexProviderCatalog<Schema> {
        &mut self.managed_indexes
    }
}

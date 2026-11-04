//! Sealed test doors over this module's private members.
use super::*;

impl<Schema> WorthQueryApplicationContractCatalog<Schema> {
    #[doc(hidden)]
    pub fn conditionals_for_test(&self) -> &BTreeMap<String, DeclaredConditionalBinding> {
        &self.conditionals
    }
}

impl<Schema> WorthQueryApplicationContractCatalog<Schema> {
    #[doc(hidden)]
    pub fn producers_for_test(&self) -> &BTreeMap<String, DeclaredProducerBinding> {
        &self.producers
    }
}

impl<Schema> WorthQueryApplicationContractCatalog<Schema>
where
    Schema: ApplicationSchema,
{
    #[doc(hidden)]
    pub fn compose_package_for_test(
        &self,
        package: WorthQueryPortableDomainPackage,
    ) -> WorthQueryPortableDomainPackage {
        Self::compose_package(self, package)
    }
}

impl<Schema> WorthQueryApplicationContractCatalog<Schema>
where
    Schema: ApplicationSchema,
{
    #[doc(hidden)]
    pub fn validate_for_test(&self) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        Self::validate(self)
    }
}

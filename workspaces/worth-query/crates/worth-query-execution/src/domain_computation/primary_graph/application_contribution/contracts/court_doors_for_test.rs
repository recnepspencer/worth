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
    pub fn validate_for_test(&self) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        Self::validate(self)
    }
}

impl<Schema> WorthQueryApplicationContractCatalog<Schema> {
    #[doc(hidden)]
    pub fn conditionals_mut_for_test(
        &mut self,
    ) -> &mut BTreeMap<String, DeclaredConditionalBinding> {
        &mut self.conditionals
    }
}

impl<Schema> WorthQueryApplicationContractCatalog<Schema> {
    #[doc(hidden)]
    pub fn producers_mut_for_test(&mut self) -> &mut BTreeMap<String, DeclaredProducerBinding> {
        &mut self.producers
    }
}

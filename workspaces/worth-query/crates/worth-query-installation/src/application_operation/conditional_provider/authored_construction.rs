//! Portable constructors bind source and provider at the same installed node.
use super::*;
use crate::domain_operation::{
    WorthQueryAuthoredHostConditionalOutputComparatorProvider,
    WorthQueryAuthoredHostConditionalOutputVersionProvider,
    WorthQueryAuthoredHostConditionalPredicateProvider,
};
impl<Schema, ApplicationOperation, Input, D, O, F, N>
    WorthQueryInstalledApplicationConditionalNode<Schema, ApplicationOperation, Input, D, O, F, N>
{
    pub fn bind_authored_host_predicate_provider<Provider>(
        self,
        provider: Provider,
    ) -> Result<
        WorthQueryInstalledHostConditionalProvider<
            Schema,
            ApplicationOperation,
            Input,
            D,
            O,
            F,
            N,
            Provider,
        >,
        WorthQueryConditionalApplicationOperationDenial,
    >
    where
        Provider: WorthQueryAuthoredHostConditionalPredicateProvider<N>,
    {
        let mut installed = self.bind_host_predicate_provider(provider)?;
        installed.authored_predicate = Some(WorthQueryAuthoredHostPredicateBinding::from_provider(
            installed.provider.clone(),
        ));
        Ok(installed)
    }
}
impl<Schema, ApplicationOperation, Input, D, O, F, N, Provider>
    WorthQueryInstalledHostConditionalProvider<
        Schema,
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        N,
        Provider,
    >
where
    Provider: WorthQueryHostConditionalPredicateProvider<N>,
{
    pub fn bind_authored_host_output_comparator_provider<P>(
        mut self,
        provider: P,
    ) -> Result<Self, WorthQueryConditionalApplicationOperationDenial>
    where
        P: WorthQueryAuthoredHostConditionalOutputComparatorProvider<N>,
    {
        validate_semantic_identity(provider.semantic_identity())?;
        self.output_comparator = Some(WorthQueryBoundHostOutputComparator::Authored(
            std::sync::Arc::new(provider),
        ));
        Ok(self)
    }
    pub fn bind_authored_host_output_version_provider<P>(
        mut self,
        provider: P,
    ) -> Result<Self, WorthQueryConditionalApplicationOperationDenial>
    where
        P: WorthQueryAuthoredHostConditionalOutputVersionProvider<N>,
    {
        validate_semantic_identity(provider.semantic_identity())?;
        self.output_version = Some(WorthQueryBoundHostOutputVersion::Authored(
            std::sync::Arc::new(provider),
        ));
        Ok(self)
    }
}

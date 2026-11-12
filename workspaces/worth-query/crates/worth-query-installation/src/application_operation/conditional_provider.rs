mod authored_construction;
mod authored_predicate;
mod output_comparator;
mod output_version;
use crate::domain_operation::{
    WorthQueryConditionalObservationView, WorthQueryHostConditionalOutputComparatorProvider,
    WorthQueryHostConditionalOutputVersionProvider, WorthQueryHostConditionalPredicateProvider,
    WorthQueryHostPredicateDecision, WorthQueryHostPredicateFailure,
};
pub use authored_predicate::WorthQueryAuthoredHostPredicateBinding;
pub use output_comparator::WorthQueryBoundHostOutputComparator;
pub use output_version::WorthQueryBoundHostOutputVersion;

use super::{
    WorthQueryConditionalApplicationOperationDenial,
    WorthQueryConditionalApplicationOperationDenialKind,
    WorthQueryInstalledApplicationConditionalNode,
};

/// Move-only installed host predicate bound to one exact conditional node.
pub struct WorthQueryInstalledHostConditionalProvider<
    Schema,
    ApplicationOperation,
    Input,
    D,
    O,
    F,
    N,
    Provider,
> {
    node: WorthQueryInstalledApplicationConditionalNode<
        Schema,
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        N,
    >,
    provider: std::sync::Arc<Provider>,
    authored_predicate: Option<WorthQueryAuthoredHostPredicateBinding<N, Provider>>,
    output_comparator: Option<WorthQueryBoundHostOutputComparator<N>>,
    output_version: Option<WorthQueryBoundHostOutputVersion<N>>,
}

impl<Schema, ApplicationOperation, Input, D, O, F, N>
    WorthQueryInstalledApplicationConditionalNode<Schema, ApplicationOperation, Input, D, O, F, N>
{
    pub fn bind_host_predicate_provider<Provider>(
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
        Provider: WorthQueryHostConditionalPredicateProvider<N>,
    {
        validate_provider_identity::<Provider, N>()?;
        Ok(WorthQueryInstalledHostConditionalProvider {
            node: self,
            provider: std::sync::Arc::new(provider),
            authored_predicate: None,
            output_comparator: None,
            output_version: None,
        })
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
    pub fn bind_host_output_comparator_provider<OutputComparator>(
        mut self,
        provider: OutputComparator,
    ) -> Result<Self, WorthQueryConditionalApplicationOperationDenial>
    where
        OutputComparator: WorthQueryHostConditionalOutputComparatorProvider<N>,
    {
        let identity = provider.semantic_identity();
        validate_semantic_identity(identity)?;
        self.output_comparator = Some(WorthQueryBoundHostOutputComparator::Memory(
            std::sync::Arc::new(provider),
        ));
        Ok(self)
    }

    pub fn bind_host_output_version_provider<OutputVersion>(
        mut self,
        provider: OutputVersion,
    ) -> Result<Self, WorthQueryConditionalApplicationOperationDenial>
    where
        OutputVersion: WorthQueryHostConditionalOutputVersionProvider<N>,
    {
        let identity = provider.semantic_identity();
        validate_semantic_identity(identity)?;
        self.output_version = Some(WorthQueryBoundHostOutputVersion::Memory(
            std::sync::Arc::new(provider),
        ));
        Ok(self)
    }

    pub fn node(
        &self,
    ) -> &WorthQueryInstalledApplicationConditionalNode<
        Schema,
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        N,
    > {
        &self.node
    }

    pub fn provider_semantic_identity(&self) -> &'static str {
        Provider::SEMANTIC_IDENTITY
    }

    #[doc(hidden)]
    pub fn evaluate_for_runtime(
        &self,
        observation: WorthQueryConditionalObservationView<'_>,
    ) -> Result<WorthQueryHostPredicateDecision, WorthQueryHostPredicateFailure> {
        self.provider.evaluate(observation)
    }

    #[doc(hidden)]
    pub fn retain_provider_for_runtime(&self) -> std::sync::Arc<Provider> {
        std::sync::Arc::clone(&self.provider)
    }

    #[doc(hidden)]
    pub fn retain_authored_predicate_for_runtime(
        &self,
    ) -> Option<WorthQueryAuthoredHostPredicateBinding<N, Provider>> {
        self.authored_predicate.clone()
    }
    #[doc(hidden)]
    pub fn retain_output_comparator_for_runtime(
        &self,
    ) -> Option<WorthQueryBoundHostOutputComparator<N>> {
        self.output_comparator.clone()
    }
    #[doc(hidden)]
    pub fn retain_output_version_for_runtime(&self) -> Option<WorthQueryBoundHostOutputVersion<N>> {
        self.output_version.clone()
    }
}

fn validate_provider_identity<Provider, Node>(
) -> Result<(), WorthQueryConditionalApplicationOperationDenial>
where
    Provider: WorthQueryHostConditionalPredicateProvider<Node>,
{
    validate_semantic_identity(Provider::SEMANTIC_IDENTITY)
}

fn validate_semantic_identity(
    identity: &str,
) -> Result<(), WorthQueryConditionalApplicationOperationDenial> {
    if identity.is_empty()
        || identity.trim() != identity
        || identity.chars().any(char::is_whitespace)
    {
        Err(WorthQueryConditionalApplicationOperationDenial::new(
            WorthQueryConditionalApplicationOperationDenialKind::ProviderIdentityInvalid,
            identity,
        ))
    } else {
        Ok(())
    }
}

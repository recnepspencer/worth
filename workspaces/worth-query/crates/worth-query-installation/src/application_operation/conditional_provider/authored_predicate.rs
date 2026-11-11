//! An installed predicate source retains its original fresh decoder across successors.
use crate::domain_operation::{
    AuthoredProviderReconstructionDenial, BridgeAuthoredProviderDescription,
    WorthQueryAuthoredHostConditionalPredicateProvider, WorthQueryHostConditionalPredicateProvider,
};
use std::{marker::PhantomData, sync::Arc};

pub struct WorthQueryAuthoredHostPredicateBinding<Node, Provider> {
    fresh: Arc<Provider>,
    provider: Arc<Provider>,
    describe: fn(&Provider) -> BridgeAuthoredProviderDescription,
    decode: fn(
        &Provider,
        &BridgeAuthoredProviderDescription,
    ) -> Result<Provider, AuthoredProviderReconstructionDenial>,
    marker: PhantomData<fn() -> Node>,
}
impl<Node, Provider> Clone for WorthQueryAuthoredHostPredicateBinding<Node, Provider> {
    fn clone(&self) -> Self {
        Self {
            fresh: Arc::clone(&self.fresh),
            provider: Arc::clone(&self.provider),
            describe: self.describe,
            decode: self.decode,
            marker: PhantomData,
        }
    }
}
impl<Node, Provider> WorthQueryAuthoredHostPredicateBinding<Node, Provider>
where
    Provider: WorthQueryHostConditionalPredicateProvider<Node>,
{
    pub fn from_provider(provider: Arc<Provider>) -> Self
    where
        Provider: WorthQueryAuthoredHostConditionalPredicateProvider<Node>,
    {
        Self {
            fresh: Arc::clone(&provider),
            provider,
            describe: Provider::authored_provider_description,
            decode: Provider::readmit_authored_provider,
            marker: PhantomData,
        }
    }
    pub fn provider(&self) -> &Arc<Provider> {
        &self.provider
    }
    pub fn retained_heap_bytes(
        &self,
    ) -> Result<
        crate::domain_operation::WorthQueryHostProviderHeapRetention,
        crate::domain_operation::WorthQueryHostProviderRetentionOverflow,
    > {
        use crate::domain_operation::WorthQueryHostProviderHeapRetention as Heap;
        let mut parts = vec![
            Heap::arc_allocation_bytes(self.provider.as_ref()),
            self.provider.retained_heap_bytes()?.bytes(),
        ];
        if !Arc::ptr_eq(&self.fresh, &self.provider) {
            parts.push(Heap::arc_allocation_bytes(self.fresh.as_ref()));
            parts.push(self.fresh.retained_heap_bytes()?.bytes());
        }
        Heap::try_from_parts(parts)
    }

    pub fn description(&self) -> BridgeAuthoredProviderDescription {
        (self.describe)(&self.provider)
    }
    pub fn readmit(
        &self,
        source: &BridgeAuthoredProviderDescription,
    ) -> Result<Self, AuthoredProviderReconstructionDenial> {
        let provider = Arc::new((self.decode)(&self.fresh, source)?);
        if &(self.describe)(&provider) != source {
            return Err(AuthoredProviderReconstructionDenial::MalformedPayload);
        }
        Ok(Self {
            fresh: Arc::clone(&self.fresh),
            provider,
            describe: self.describe,
            decode: self.decode,
            marker: PhantomData,
        })
    }
}

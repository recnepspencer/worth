//! Complete host-owned provider parameters and reconstruction from fresh installation.
use super::host_provider_contract::{
    WorthQueryHostConditionalOutputComparatorProvider,
    WorthQueryHostConditionalOutputVersionProvider, WorthQueryHostConditionalPredicateProvider,
};
use std::sync::Arc;
pub use worth_foundational::facade::{
    AuthoredProviderReconstructionDenial, BridgeAuthoredProviderDescription,
};

pub trait WorthQueryAuthoredHostConditionalPredicateProvider<Node>:
    WorthQueryHostConditionalPredicateProvider<Node> + Sized
{
    fn authored_provider_description(&self) -> BridgeAuthoredProviderDescription;
    fn readmit_authored_provider(
        &self,
        source: &BridgeAuthoredProviderDescription,
    ) -> Result<Self, AuthoredProviderReconstructionDenial>;
}

pub trait WorthQueryAuthoredHostConditionalOutputComparatorProvider<Node>:
    WorthQueryHostConditionalOutputComparatorProvider<Node>
{
    fn authored_provider_description(&self) -> BridgeAuthoredProviderDescription;
    fn readmit_authored_provider(
        &self,
        source: &BridgeAuthoredProviderDescription,
    ) -> Result<
        Arc<dyn WorthQueryAuthoredHostConditionalOutputComparatorProvider<Node>>,
        AuthoredProviderReconstructionDenial,
    >;
}

pub trait WorthQueryAuthoredHostConditionalOutputVersionProvider<Node>:
    WorthQueryHostConditionalOutputVersionProvider<Node>
{
    fn authored_provider_description(&self) -> BridgeAuthoredProviderDescription;
    fn readmit_authored_provider(
        &self,
        source: &BridgeAuthoredProviderDescription,
    ) -> Result<
        Arc<dyn WorthQueryAuthoredHostConditionalOutputVersionProvider<Node>>,
        AuthoredProviderReconstructionDenial,
    >;
}

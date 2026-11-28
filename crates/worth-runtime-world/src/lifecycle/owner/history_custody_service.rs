use super::RuntimeWorldOwnerRoot;
use crate::history::{CompositeHistoryCatalogDenial, RuntimeWorldPerformedPublicationProtection};
use crate::identity::CompositeCommitIdentity;
use crate::lifecycle::RuntimeWorldHistoryCustodyService;

impl<D, I, E, Ctx, T> RuntimeWorldHistoryCustodyService for RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    fn protect_performed_publication(
        &self,
        identity: &CompositeCommitIdentity,
    ) -> Result<RuntimeWorldPerformedPublicationProtection, CompositeHistoryCatalogDenial> {
        self.state.history.protect_performed_publication(identity)
    }
}

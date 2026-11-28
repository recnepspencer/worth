use std::sync::{Arc, Weak};

use crate::history::{CompositeHistoryCatalogDenial, RuntimeWorldPerformedPublicationProtection};
use crate::identity::CompositeCommitIdentity;
use crate::lifecycle::availability::RuntimeWorldAvailability;
use crate::lifecycle::{RuntimeWorldOwnerUnavailable, RuntimeWorldServiceDenial};

pub(crate) trait RuntimeWorldHistoryCustodyService:
    RuntimeWorldAvailability + Send + Sync
{
    fn protect_performed_publication(
        &self,
        identity: &CompositeCommitIdentity,
    ) -> Result<RuntimeWorldPerformedPublicationProtection, CompositeHistoryCatalogDenial>;
}

/// Weak authority to acquire independent retention for a performed publication.
#[derive(Clone)]
pub struct RuntimeWorldHistoryCustodyPort {
    owner: Weak<dyn RuntimeWorldHistoryCustodyService>,
}

impl RuntimeWorldHistoryCustodyPort {
    pub(crate) fn from_service(owner: Arc<dyn RuntimeWorldHistoryCustodyService>) -> Self {
        Self {
            owner: Arc::downgrade(&owner),
        }
    }

    /// Retain the exact performed publication against reclamation until the guard drops.
    pub fn protect_performed_publication(
        &self,
        identity: &CompositeCommitIdentity,
    ) -> Result<
        RuntimeWorldPerformedPublicationProtection,
        RuntimeWorldServiceDenial<CompositeHistoryCatalogDenial>,
    > {
        let owner = self
            .owner
            .upgrade()
            .ok_or_else(RuntimeWorldOwnerUnavailable::new)?;
        if !owner.is_available() || !owner.is_bootstrapped() {
            return Err(RuntimeWorldOwnerUnavailable::new().into());
        }
        owner
            .protect_performed_publication(identity)
            .map_err(RuntimeWorldServiceDenial::Denied)
    }
}

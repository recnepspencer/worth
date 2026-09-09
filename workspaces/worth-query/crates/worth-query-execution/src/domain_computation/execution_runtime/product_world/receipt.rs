use std::sync::{Arc, Mutex};
use worth_runtime_world::facade::ConsumedCompositePublication;

use super::{runtime::WorthQueryProductRootIdentity, WorthQueryPerformedRelationalProductChange};

/// Named read-only custody shared by committed Query result observers after
/// the linear World handoff was consumed exactly once.
#[derive(Clone, Debug)]
pub(crate) struct WorthQueryProductPublicationReceipt {
    publication: Arc<ConsumedCompositePublication>,
    fresh_delivery: Arc<Mutex<Option<Arc<ConsumedCompositePublication>>>>,
    root_identity: Arc<WorthQueryProductRootIdentity>,
}

impl WorthQueryProductPublicationReceipt {
    pub(crate) fn new(
        publication: ConsumedCompositePublication,
        root_identity: Arc<WorthQueryProductRootIdentity>,
    ) -> Self {
        let publication = Arc::new(publication);
        Self {
            publication: Arc::clone(&publication),
            fresh_delivery: Arc::new(Mutex::new(Some(publication))),
            root_identity,
        }
    }

    pub(crate) fn publication(&self) -> &ConsumedCompositePublication {
        &self.publication
    }

    pub(crate) fn take_fresh_delivery(&self) -> Option<WorthQueryPerformedRelationalProductChange> {
        let publication = self
            .fresh_delivery
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()?;
        Some(WorthQueryPerformedRelationalProductChange::new(
            Arc::clone(&self.root_identity),
            publication,
        ))
    }
}

impl PartialEq for WorthQueryProductPublicationReceipt {
    fn eq(&self, other: &Self) -> bool {
        self.publication.commit().identity() == other.publication.commit().identity()
            && self.publication.attempt_identity() == other.publication.attempt_identity()
    }
}

impl Eq for WorthQueryProductPublicationReceipt {}

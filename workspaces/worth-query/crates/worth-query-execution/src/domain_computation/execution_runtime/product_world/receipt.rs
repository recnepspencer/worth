use std::sync::{Arc, Mutex, OnceLock};

use worth_runtime_world::facade::{ConsumedCompositePublication, ProductUnpublishedRecoveryHandle};

use super::{runtime::WorthQueryProductRootIdentity, WorthQueryPerformedRelationalProductChange};

struct WorthQueryProductPublicationTerminal {
    publication: ConsumedCompositePublication,
    conditional_definition_generation: Option<u64>,
}

struct WorthQueryProductPublicationCustody {
    terminal: OnceLock<WorthQueryProductPublicationTerminal>,
    fresh_delivery_available: Mutex<bool>,
    root_identity: Arc<WorthQueryProductRootIdentity>,
    _recovery_handle: ProductUnpublishedRecoveryHandle,
}

/// Fill-once custody allocated before Runtime World can move either owner.
pub(crate) struct WorthQueryReservedProductPublicationReceipt {
    custody: Arc<WorthQueryProductPublicationCustody>,
}

/// Named read-only custody shared by committed Query result observers after
/// the linear World handoff was consumed exactly once.
#[derive(Clone)]
pub(crate) struct WorthQueryProductPublicationReceipt {
    custody: Arc<WorthQueryProductPublicationCustody>,
}

impl WorthQueryReservedProductPublicationReceipt {
    pub(crate) fn new(
        root_identity: Arc<WorthQueryProductRootIdentity>,
        recovery_handle: ProductUnpublishedRecoveryHandle,
    ) -> Self {
        Self {
            custody: Arc::new(WorthQueryProductPublicationCustody {
                terminal: OnceLock::new(),
                fresh_delivery_available: Mutex::new(true),
                root_identity,
                _recovery_handle: recovery_handle,
            }),
        }
    }

    pub(crate) fn fill(
        self,
        publication: ConsumedCompositePublication,
        conditional_definition_generation: Option<u64>,
    ) -> WorthQueryProductPublicationReceipt {
        let terminal = WorthQueryProductPublicationTerminal {
            publication,
            conditional_definition_generation,
        };
        self.custody.terminal.get_or_init(|| terminal);
        WorthQueryProductPublicationReceipt {
            custody: self.custody,
        }
    }
}

impl WorthQueryProductPublicationReceipt {
    fn terminal(&self) -> &WorthQueryProductPublicationTerminal {
        self.custody
            .terminal
            .get()
            .expect("committed publication custody is initialized before exposure")
    }

    pub(crate) fn publication(&self) -> &ConsumedCompositePublication {
        &self.terminal().publication
    }

    pub(crate) fn root_identity(&self) -> Arc<WorthQueryProductRootIdentity> {
        Arc::clone(&self.custody.root_identity)
    }

    pub(crate) fn conditional_definition_generation(&self) -> Option<u64> {
        self.terminal().conditional_definition_generation
    }

    pub(crate) fn take_fresh_delivery(&self) -> Option<WorthQueryPerformedRelationalProductChange> {
        let mut available = self
            .custody
            .fresh_delivery_available
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !*available {
            return None;
        }
        *available = false;
        Some(WorthQueryPerformedRelationalProductChange::new(
            self.clone(),
        ))
    }
}

impl std::fmt::Debug for WorthQueryProductPublicationReceipt {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryProductPublicationReceipt")
            .field("commit", self.publication().commit().identity())
            .field("attempt", self.publication().attempt_identity())
            .finish_non_exhaustive()
    }
}

impl PartialEq for WorthQueryProductPublicationReceipt {
    fn eq(&self, other: &Self) -> bool {
        self.publication().commit().identity() == other.publication().commit().identity()
            && self.publication().attempt_identity() == other.publication().attempt_identity()
            && self.conditional_definition_generation() == other.conditional_definition_generation()
    }
}

impl Eq for WorthQueryProductPublicationReceipt {}

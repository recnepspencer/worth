//! Terminal external-effect truth sealed only by World's performed publication.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use worth_runtime_world::facade::ProductBranchIncarnation;

use super::WorthQueryAcceptedInboundOccurrence;
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationReceipt;
use crate::domain_computation::primary_graph::WorthQueryPerformedInboundCompletion;

/// Aftermath's retained completion owner. Neither an authenticated envelope
/// nor an accepted occurrence can construct this terminal result.
pub struct WorthQueryInboundTerminalOwnerResult {
    performed: WorthQueryPerformedInboundCompletion,
    delivery_settled: AtomicBool,
    delivery_handoff: Mutex<()>,
}

impl WorthQueryInboundTerminalOwnerResult {
    pub fn publication_mut(
        &mut self,
    ) -> &mut WorthQueryProductPublicationReceipt {
        self.performed.publication_mut()
    }
    pub fn from_performed(
        performed: WorthQueryPerformedInboundCompletion,
    ) -> Self {
        Self {
            performed,
            delivery_settled: AtomicBool::new(false),
            delivery_handoff: Mutex::new(()),
        }
    }

    /// Settles this terminal's one-shot fresh delivery while the delivery
    /// remains exclusive. A retry observes the settled bit and returns early.
    pub fn settle_fresh_delivery(&self) -> bool {
        let _handoff = self
            .delivery_handoff
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        if self.delivery_settled.load(Ordering::Acquire) {
            return true;
        }
        if !self.performed.publication().discharge_fresh_delivery() {
            return false;
        }
        self.delivery_settled.store(true, Ordering::Release);
        true
    }

    pub fn delivery_settled(&self) -> bool {
        self.delivery_settled.load(Ordering::Acquire)
    }

    pub fn accepted(
        &self,
    ) -> &Arc<WorthQueryAcceptedInboundOccurrence> {
        self.performed.accepted()
    }

    pub fn original_incarnation(&self) -> ProductBranchIncarnation {
        self.performed.original_incarnation()
    }

    pub fn publication(
        &self,
    ) -> &WorthQueryProductPublicationReceipt {
        self.performed.publication()
    }
}

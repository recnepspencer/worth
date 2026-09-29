//! Terminal external-effect truth sealed only by World's performed publication.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use worth_runtime_world::facade::ProductBranchIncarnation;

use super::WorthQueryAcceptedInboundOccurrence;
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationReceipt;
use crate::domain_computation::primary_graph::WorthQueryPerformedInboundCompletion;

/// Aftermath's retained completion owner. Neither an authenticated envelope
/// nor an accepted occurrence can construct this terminal result.
pub(in crate::domain_computation) struct WorthQueryInboundTerminalOwnerResult {
    performed: WorthQueryPerformedInboundCompletion,
    delivery_settled: AtomicBool,
    delivery_handoff: Mutex<()>,
}

impl WorthQueryInboundTerminalOwnerResult {
    pub(in crate::domain_computation) fn from_performed(
        performed: WorthQueryPerformedInboundCompletion,
    ) -> Self {
        Self {
            performed,
            delivery_settled: AtomicBool::new(false),
            delivery_handoff: Mutex::new(()),
        }
    }

    /// The runtime hands the actual performed Relational commit to its
    /// conditional journal while this terminal's delivery remains exclusive.
    /// A retry observes the settled bit and cannot record the commit again.
    pub(in crate::domain_computation) fn settle_after_conditional_handoff(
        &self,
        handoff: impl FnOnce(),
    ) -> bool {
        let _handoff = self
            .delivery_handoff
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        if self.delivery_settled.load(Ordering::Acquire) {
            return true;
        }
        handoff();
        if !self
            .performed
            .publication()
            .settle_fresh_delivery_after_owner_handoff()
        {
            return false;
        }
        self.delivery_settled.store(true, Ordering::Release);
        true
    }

    pub(in crate::domain_computation) fn delivery_settled(&self) -> bool {
        self.delivery_settled.load(Ordering::Acquire)
    }

    pub(in crate::domain_computation) fn accepted(
        &self,
    ) -> &Arc<WorthQueryAcceptedInboundOccurrence> {
        self.performed.accepted()
    }

    pub(in crate::domain_computation) fn original_incarnation(&self) -> ProductBranchIncarnation {
        self.performed.original_incarnation()
    }

    pub(in crate::domain_computation) fn publication(
        &self,
    ) -> &WorthQueryProductPublicationReceipt {
        self.performed.publication()
    }
}

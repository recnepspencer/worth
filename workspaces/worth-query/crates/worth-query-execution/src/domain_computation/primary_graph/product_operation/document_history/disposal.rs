//! Disposal owns every remaining writable head and every actual retired cleanup.
use super::{
    journal::{HistoryJournal, PastHead},
    WorthQueryOwnedProductHead,
};
use crate::domain_computation::{
    execution_runtime::product_world::{
        WorthQueryProductBranchCloseReceipt, WorthQueryProductBranchOwnerCleanupReceipt,
    },
    primary_graph::{
        WorthQueryAdvancementDenial, WorthQueryAdvancementPhase,
        WorthQueryApplicationProductBranchCleanupDenial,
        WorthQueryApplicationProductBranchCleanupFailure,
        WorthQueryApplicationProductBranchCloseDenial, WorthQueryOwnedProductHeadCloseFailure,
        WorthQueryPrimaryGraphApplicationRuntime,
    },
};
use std::collections::btree_map::IntoValues;
use worth_query_installation::facade::ApplicationSchema;

enum PendingHead<'runtime, S> {
    Live {
        head: WorthQueryOwnedProductHead<'runtime, S>,
        denial: WorthQueryApplicationProductBranchCloseDenial,
    },
    Retired {
        branch: crate::basis::WorthQueryProductBranch,
        failure: WorthQueryApplicationProductBranchCleanupFailure,
    },
}

enum ClosedHead {
    Direct {
        _receipt: WorthQueryProductBranchCloseReceipt,
    },
    Cleanup {
        _receipt: WorthQueryProductBranchOwnerCleanupReceipt,
    },
}

/// All document-managed pins and head custody have been discharged.
/// This records actual owner completions; unrelated holders can retain their own history.
pub struct WorthQueryDocumentHistoryCloseReceipt {
    heads: usize,
    _final_head: ClosedHead,
}

impl WorthQueryDocumentHistoryCloseReceipt {
    pub fn closed_heads(&self) -> usize {
        self.heads
    }
}

impl std::fmt::Debug for WorthQueryDocumentHistoryCloseReceipt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryDocumentHistoryCloseReceipt")
            .field("closed_heads", &self.heads)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub enum WorthQueryDocumentHistoryClosingDenial<'a> {
    Execution(&'a WorthQueryAdvancementDenial),
    Head(&'a WorthQueryApplicationProductBranchCloseDenial),
    Cleanup(&'a WorthQueryApplicationProductBranchCleanupDenial),
}

/// A closing document cannot read, edit, navigate or become a live document again.
#[must_use = "retain and retry every refused document cleanup owner"]
pub struct WorthQueryDocumentHistoryClosing<'runtime, S> {
    application: &'runtime WorthQueryPrimaryGraphApplicationRuntime<S>,
    current: Option<WorthQueryOwnedProductHead<'runtime, S>>,
    past: IntoValues<u64, PastHead<'runtime, S>>,
    pending: Option<PendingHead<'runtime, S>>,
    opening_denial: Option<WorthQueryAdvancementDenial>,
    closed: usize,
    last: Option<ClosedHead>,
}

impl<'runtime, S: ApplicationSchema> WorthQueryDocumentHistoryClosing<'runtime, S> {
    pub(super) fn new(
        current: WorthQueryOwnedProductHead<'runtime, S>,
        journal: HistoryJournal<'runtime, S>,
    ) -> Self {
        // Drop every document-owned read pin before retiring any of its heads.
        // Native close/retry occurs only after these resource drops have finished.
        let past = journal.into_disposal();
        Self {
            application: current.application(),
            current: Some(current),
            past,
            pending: None,
            opening_denial: None,
            closed: 0,
            last: None,
        }
    }

    pub fn denial(&self) -> Option<WorthQueryDocumentHistoryClosingDenial<'_>> {
        if let Some(denial) = &self.opening_denial {
            return Some(WorthQueryDocumentHistoryClosingDenial::Execution(denial));
        }
        match self.pending.as_ref()? {
            PendingHead::Live { denial, .. } => {
                Some(WorthQueryDocumentHistoryClosingDenial::Head(denial))
            }
            PendingHead::Retired { failure, .. } => Some(
                WorthQueryDocumentHistoryClosingDenial::Cleanup(failure.denial()),
            ),
        }
    }

    pub fn retry(
        mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<WorthQueryDocumentHistoryCloseReceipt, Self> {
        if let Err(cause) = phase.checked_execution_request_for(&self.application.product_runtime) {
            self.opening_denial = Some(cause);
            return Err(self);
        }
        self.opening_denial = None;
        loop {
            if let Some(pending) = self.pending.take() {
                match pending {
                    PendingHead::Live { head, .. } => {
                        if !self.close_head(head, phase) {
                            return Err(self);
                        }
                    }
                    PendingHead::Retired { branch, failure } => {
                        match failure.into_cleanup().retry() {
                            Ok(receipt) => {
                                self.closed += 1;
                                self.last = Some(ClosedHead::Cleanup { _receipt: receipt });
                            }
                            Err(failure) => {
                                self.pending = Some(PendingHead::Retired { branch, failure });
                                return Err(self);
                            }
                        }
                    }
                }
                continue;
            }
            let next = self
                .past
                .find_map(|entry| entry.head)
                .or_else(|| self.current.take());
            let Some(head) = next else {
                return Ok(WorthQueryDocumentHistoryCloseReceipt {
                    heads: self.closed,
                    _final_head: self
                        .last
                        .expect("the document's actual current head closed"),
                });
            };
            if !self.close_head(head, phase) {
                return Err(self);
            }
        }
    }

    fn close_head(
        &mut self,
        head: WorthQueryOwnedProductHead<'runtime, S>,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> bool {
        match head.close(phase) {
            Ok(receipt) => {
                self.closed += 1;
                self.last = Some(ClosedHead::Direct { _receipt: receipt });
                true
            }
            Err(WorthQueryOwnedProductHeadCloseFailure::Retained { head, denial }) => {
                self.pending = Some(PendingHead::Live { head, denial });
                false
            }
            Err(WorthQueryOwnedProductHeadCloseFailure::Retired { branch, failure }) => {
                self.pending = Some(PendingHead::Retired { branch, failure });
                false
            }
        }
    }

    pub(super) fn into_terminal_cleanup(
        mut self,
    ) -> Result<
        (
            crate::basis::WorthQueryProductBranch,
            WorthQueryApplicationProductBranchCleanupFailure,
        ),
        Self,
    > {
        if self.current.is_none()
            && self.past.len() == 0
            && matches!(self.pending, Some(PendingHead::Retired { .. }))
        {
            let Some(PendingHead::Retired { branch, failure }) = self.pending.take() else {
                unreachable!()
            };
            Ok((branch, failure))
        } else {
            Err(self)
        }
    }
}

impl<S: ApplicationSchema> std::fmt::Debug for WorthQueryDocumentHistoryClosing<'_, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryDocumentHistoryClosing")
            .field("denial", &self.denial())
            .field("closed_heads", &self.closed)
            .finish_non_exhaustive()
    }
}

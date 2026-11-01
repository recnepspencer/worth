//! Disposal discharges the same actual custody used by bounded maintenance.
use super::{
    head_custody::{ClosedHistoryHead, HistoryHeadCustody},
    journal::{HistoryJournal, PastHead},
    WorthQueryOwnedProductHead,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementDenial, WorthQueryAdvancementPhase,
    WorthQueryApplicationProductBranchCleanupDenial,
    WorthQueryApplicationProductBranchCleanupFailure,
    WorthQueryApplicationProductBranchCloseDenial, WorthQueryPrimaryGraphApplicationRuntime,
};
use std::collections::btree_map::IntoValues;
use worth_query_installation::facade::ApplicationSchema;

/// Actual session-owned head completions; other holders retain their own rights.
pub struct WorthQueryDocumentHistoryCloseReceipt {
    heads: usize,
    _final_head: ClosedHistoryHead,
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
    Navigation(&'a super::WorthQueryHistoryNavigationDenial),
}

/// A closing document cannot become a live document again.
#[must_use = "retain and retry every refused document cleanup owner"]
pub struct WorthQueryDocumentHistoryClosing<'runtime, S> {
    application: &'runtime WorthQueryPrimaryGraphApplicationRuntime<S>,
    current: Option<WorthQueryOwnedProductHead<'runtime, S>>,
    past: IntoValues<u64, PastHead<'runtime, S>>,
    pending: Option<HistoryHeadCustody<'runtime, S>>,
    opening_denial: Option<WorthQueryAdvancementDenial>,
    closed: usize,
    last: Option<ClosedHistoryHead>,
    navigation: super::navigation::NavigationDisposal<'runtime, S>,
    navigation_denial: Option<super::WorthQueryHistoryNavigationDenial>,
}

impl<'runtime, S: ApplicationSchema> WorthQueryDocumentHistoryClosing<'runtime, S> {
    pub(super) fn new(
        current: WorthQueryOwnedProductHead<'runtime, S>,
        journal: HistoryJournal<'runtime, S>,
        navigation: super::navigation::NavigationJournal<'runtime, S>,
    ) -> Self {
        // Every controller source is released before any branch retirement.
        let (closed, past) = journal.into_disposal();
        let navigation = navigation.into_disposal();
        Self {
            application: current.application(),
            current: Some(current),
            past,
            pending: None,
            opening_denial: None,
            closed,
            last: None,
            navigation,
            navigation_denial: None,
        }
    }

    pub fn denial(&self) -> Option<WorthQueryDocumentHistoryClosingDenial<'_>> {
        use WorthQueryDocumentHistoryClosingDenial as Denial;
        if let Some(denial) = &self.opening_denial {
            return Some(Denial::Execution(denial));
        }
        if let Some(denial) = &self.navigation_denial {
            return Some(Denial::Navigation(denial));
        }
        match self.pending.as_ref()? {
            HistoryHeadCustody::Refused { denial, .. } => Some(Denial::Head(denial)),
            HistoryHeadCustody::Retired { failure, .. } => Some(Denial::Cleanup(failure.denial())),
            HistoryHeadCustody::Live(_)
            | HistoryHeadCustody::Reserved
            | HistoryHeadCustody::Advancing
            | HistoryHeadCustody::Closed(_) => None,
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
        self.navigation_denial = None;
        match self.navigation.retry(phase) {
            Ok(closed) => self.closed += closed,
            Err(cause) => {
                self.navigation_denial = Some(cause);
                return Err(self);
            }
        }
        loop {
            let next = self
                .pending
                .take()
                .or_else(|| {
                    self.past.next().map(|entry| {
                        std::mem::replace(&mut *entry.custody.lock(), HistoryHeadCustody::Reserved)
                    })
                })
                .or_else(|| self.current.take().map(HistoryHeadCustody::Live));
            let Some(head) = next else {
                return Ok(WorthQueryDocumentHistoryCloseReceipt {
                    heads: self.closed,
                    _final_head: self
                        .last
                        .expect("the document's actual current head closed"),
                });
            };
            match head.retry(phase) {
                Ok(Some(receipt)) => {
                    self.closed += 1;
                    self.last = Some(receipt);
                }
                Ok(None) => {}
                Err(head) => {
                    self.pending = Some(head);
                    return Err(self);
                }
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
            && self.navigation.is_empty()
            && self.past.len() == 0
            && matches!(self.pending, Some(HistoryHeadCustody::Retired { .. }))
        {
            let Some(HistoryHeadCustody::Retired { branch, failure }) = self.pending.take() else {
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

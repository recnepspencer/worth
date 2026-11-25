//! One guarded authority for the document's selected head and occurrence metadata.
use super::{journal::HistoryJournal, WorthQueryDocumentHistoryDenial as Denial};
use crate::{
    basis::{
        WorthQueryProductBranch, WorthQueryProductBranchReadIdentity,
        WorthQueryProductObservationLease,
    },
    domain_computation::primary_graph::{
        product_operation::DetachedOwnedProductHead, WorthQueryOwnedProductHead,
    },
};
use std::ops::Deref;
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_query_installation::facade::ApplicationSchema;

trait HistorySelectionCellBacking: Send + Sync {
    fn as_cell(&self) -> &HistorySelectionCell;
}

impl HistorySelectionCellBacking for HistorySelectionCell {
    fn as_cell(&self) -> &HistorySelectionCell {
        self
    }
}

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct SharedHistorySelectionCell(
    Arc<dyn HistorySelectionCellBacking>,
);

impl SharedHistorySelectionCell {
    pub(in crate::domain_computation::primary_graph) fn from_arc(
        cell: Arc<HistorySelectionCell>,
    ) -> Self {
        Self(cell)
    }

    pub(in crate::domain_computation::primary_graph) fn downgrade(
        &self,
    ) -> WeakHistorySelectionCell {
        WeakHistorySelectionCell(Arc::downgrade(&self.0))
    }

    pub(in crate::domain_computation::primary_graph) fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Deref for SharedHistorySelectionCell {
    type Target = HistorySelectionCell;

    fn deref(&self) -> &Self::Target {
        self.0.as_cell()
    }
}

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct WeakHistorySelectionCell(
    Weak<dyn HistorySelectionCellBacking>,
);

impl WeakHistorySelectionCell {
    pub(in crate::domain_computation::primary_graph) fn upgrade(
        &self,
    ) -> Option<SharedHistorySelectionCell> {
        self.0.upgrade().map(SharedHistorySelectionCell)
    }
}

pub(in crate::domain_computation::primary_graph) struct HistorySelection {
    head: Option<DetachedOwnedProductHead>,
    pub expected: WorthQueryProductBranchReadIdentity,
    pub physical_source: WorthQueryProductObservationLease,
    output_authority: Option<super::authority_lifetime::OwnedHistoryAuthorityLifetime>,
    pub journal: HistoryJournal,
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) navigation:
        super::navigation::NavigationJournal,
}

pub(in crate::domain_computation::primary_graph) struct HistorySelectionCell {
    state: Mutex<Option<HistorySelection>>,
    _allocation: Option<ExecutionMemoryReservation>,
}

impl HistorySelectionCell {
    pub(in crate::domain_computation::primary_graph) fn attach<'runtime, S: ApplicationSchema>(
        execution: &ExecutionRequest<'_, '_>,
        head: WorthQueryOwnedProductHead<'runtime, S>,
        expected: WorthQueryProductBranchReadIdentity,
        source: WorthQueryProductObservationLease,
        retained: Arc<
            crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
        >,
        navigation_resources: worth_query_declaration::facade::application_history::ApplicationHistoryNavigationResources,
        retention_resources: worth_query_declaration::facade::application_history::ApplicationHistoryRetentionResources,
    ) -> Result<SharedHistorySelectionCell, (WorthQueryOwnedProductHead<'runtime, S>, Denial)> {
        let allocation = match super::allocation::reserve_arc::<Self>(execution) {
            Ok(allocation) => allocation,
            Err(cause) => return Err((head, Denial::Recording(cause))),
        };
        let navigation =
            match super::navigation::NavigationJournal::new(execution, navigation_resources) {
                Ok(navigation) => navigation,
                Err(denial) => return Err((head, denial)),
            };
        let journal = match HistoryJournal::attach(
            execution,
            head.application()
                .inner
                .runtime
                .authority_identity()
                .as_u64(),
            expected.clone(),
            retained,
            retention_resources,
            Arc::clone(&navigation.capacity),
        ) {
            Ok(journal) => journal,
            Err(denial) => return Err((head, denial)),
        };
        Ok(SharedHistorySelectionCell::from_arc(Arc::new(Self {
            state: Mutex::new(Some(HistorySelection {
                head: Some(head.into_detached()),
                expected,
                physical_source: source,
                output_authority: None,
                journal,
                navigation,
            })),
            _allocation: allocation,
        })))
    }

    /// Lock order is selection → bounded Execution accounting, or selection →
    /// existing branch/Native/World adoption guards. No application callback,
    /// query, candidate work, pause or operational-owner destruction runs here.
    /// Pending reservations do not change the selected read identity.
    pub(in crate::domain_computation::primary_graph) fn lock(&self) -> HistorySelectionGuard<'_> {
        HistorySelectionGuard(
            self.state
                .lock()
                .unwrap_or_else(|poison| poison.into_inner()),
        )
    }

    pub(super) fn live(&self) -> Option<HistorySelectionGuard<'_>> {
        let guard = self
            .state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        guard.is_some().then_some(HistorySelectionGuard(guard))
    }

    /// The move-only document is the sole closing owner. Take custody under the
    /// cell, then release observations and close branches after unlocking it.
    pub(in crate::domain_computation::primary_graph) fn take(&self) -> HistorySelection {
        let mut selected = self.lock();
        selected.invalidate_output_authority();
        selected
            .0
            .take()
            .expect("the document closes its live selection once")
    }
}

impl HistorySelection {
    pub(in crate::domain_computation::primary_graph) fn head_branch(
        &self,
    ) -> WorthQueryProductBranch {
        self.head
            .as_ref()
            .expect("live history selection retains its move-only head custody")
            .branch()
    }

    pub(in crate::domain_computation::primary_graph) fn take_head(
        &mut self,
    ) -> DetachedOwnedProductHead {
        self.head
            .take()
            .expect("live history selection retains its move-only head custody")
    }

    pub(in crate::domain_computation::primary_graph) fn put_head(
        &mut self,
        head: DetachedOwnedProductHead,
    ) {
        let replaced = self.head.replace(head);
        debug_assert!(replaced.is_none());
    }

    pub(in crate::domain_computation::primary_graph) fn retain_output_authority_lifetime(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<super::authority_lifetime::HistoryAuthorityLifetime, Denial> {
        if self.output_authority.is_none() {
            self.output_authority = Some(
                super::authority_lifetime::OwnedHistoryAuthorityLifetime::reserve(execution)
                    .map_err(Denial::Recording)?,
            );
        }
        Ok(self
            .output_authority
            .as_ref()
            .expect("the selected slot owns its lifetime")
            .retain())
    }

    pub(in crate::domain_computation::primary_graph) fn invalidate_output_authority(&mut self) {
        drop(self.output_authority.take());
    }
}

pub(in crate::domain_computation::primary_graph) struct HistorySelectionGuard<'cell>(
    MutexGuard<'cell, Option<HistorySelection>>,
);

impl std::ops::Deref for HistorySelectionGuard<'_> {
    type Target = HistorySelection;
    fn deref(&self) -> &Self::Target {
        self.0
            .as_ref()
            .expect("the live document retains its selection")
    }
}

impl std::ops::DerefMut for HistorySelectionGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
            .as_mut()
            .expect("the live document retains its selection")
    }
}

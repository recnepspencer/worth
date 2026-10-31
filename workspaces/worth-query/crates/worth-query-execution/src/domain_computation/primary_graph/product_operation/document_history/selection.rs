//! One guarded authority for the document's selected head and occurrence metadata.
use super::{journal::HistoryJournal, WorthQueryDocumentHistoryDenial as Denial};
use crate::{
    basis::{WorthQueryProductBranchReadIdentity, WorthQueryProductObservationLease},
    domain_computation::primary_graph::WorthQueryOwnedProductHead,
};
use std::sync::{Arc, Mutex, MutexGuard};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_query_installation::facade::ApplicationSchema;

pub(in crate::domain_computation::primary_graph) struct HistorySelection<'runtime, S> {
    pub head: WorthQueryOwnedProductHead<'runtime, S>,
    pub expected: WorthQueryProductBranchReadIdentity,
    pub physical_source: WorthQueryProductObservationLease,
    pub journal: HistoryJournal<'runtime, S>,
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) navigation:
        super::navigation::NavigationJournal<'runtime, S>,
}

pub(in crate::domain_computation::primary_graph) struct HistorySelectionCell<'runtime, S> {
    state: Mutex<Option<HistorySelection<'runtime, S>>>,
    _allocation: Option<ExecutionMemoryReservation>,
}

impl<'runtime, S: ApplicationSchema> HistorySelectionCell<'runtime, S> {
    pub(in crate::domain_computation::primary_graph) fn attach(
        execution: &ExecutionRequest<'_, '_>,
        head: WorthQueryOwnedProductHead<'runtime, S>,
        expected: WorthQueryProductBranchReadIdentity,
        source: WorthQueryProductObservationLease,
        retained: Arc<
            crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
        >,
    ) -> Result<Arc<Self>, (WorthQueryOwnedProductHead<'runtime, S>, Denial)> {
        let allocation = match super::allocation::reserve_arc::<Self>(execution) {
            Ok(allocation) => allocation,
            Err(cause) => return Err((head, Denial::Recording(cause))),
        };
        let journal = match HistoryJournal::attach(
            execution,
            head.application().runtime.authority_identity().as_u64(),
            expected.clone(),
            retained,
        ) {
            Ok(journal) => journal,
            Err(denial) => return Err((head, denial)),
        };
        Ok(Arc::new(Self {
            state: Mutex::new(Some(HistorySelection {
                head,
                expected,
                physical_source: source,
                journal,
                navigation: super::navigation::NavigationJournal::new(),
            })),
            _allocation: allocation,
        }))
    }

    /// Lock order is selection → bounded Execution accounting, or selection →
    /// existing branch/Native/World adoption guards. No application callback,
    /// query, candidate work, pause or operational-owner destruction runs here.
    /// Pending reservations do not change the selected read identity.
    pub(in crate::domain_computation::primary_graph) fn lock(
        &self,
    ) -> HistorySelectionGuard<'_, 'runtime, S> {
        HistorySelectionGuard(
            self.state
                .lock()
                .unwrap_or_else(|poison| poison.into_inner()),
        )
    }

    pub(super) fn live(&self) -> Option<HistorySelectionGuard<'_, 'runtime, S>> {
        let guard = self
            .state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        guard.is_some().then_some(HistorySelectionGuard(guard))
    }

    /// The move-only document is the sole closing owner. Take custody under the
    /// cell, then release observations and close branches after unlocking it.
    pub(in crate::domain_computation::primary_graph) fn take(
        &self,
    ) -> HistorySelection<'runtime, S> {
        self.lock()
            .0
            .take()
            .expect("the document closes its live selection once")
    }
}

pub(in crate::domain_computation::primary_graph) struct HistorySelectionGuard<'cell, 'runtime, S>(
    MutexGuard<'cell, Option<HistorySelection<'runtime, S>>>,
);

impl<'runtime, S> std::ops::Deref for HistorySelectionGuard<'_, 'runtime, S> {
    type Target = HistorySelection<'runtime, S>;
    fn deref(&self) -> &Self::Target {
        self.0
            .as_ref()
            .expect("the live document retains its selection")
    }
}

impl<S> std::ops::DerefMut for HistorySelectionGuard<'_, '_, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
            .as_mut()
            .expect("the live document retains its selection")
    }
}

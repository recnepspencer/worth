//! Exact retained product ancestry with a finite single-parent work granule.
use std::sync::Arc;

use crate::branch::{ProductBranchObservation, RuntimeWorldBranchAdmissionDenial};
use crate::history::{
    CompositeAncestorSelectionDenial, CompositeRuntimeWorldCommit, RetainedExactHistoryPosition,
};

/// Typed refusal to open or advance exact retained product history.
/// Branch and history variants preserve their actual owner's denial; request
/// and allocation variants retain the original execution cause. An invalid
/// continuation cannot install a successor on the supplied cursor.
#[derive(Debug)]
pub enum ProductExactHistoryDenial {
    Branch(RuntimeWorldBranchAdmissionDenial),
    History(CompositeAncestorSelectionDenial),
    Request(worth_execution::WorkCeilingDenial),
    Allocation(worth_execution::ExecutionAllocationDenial),
    InvalidContinuation,
}

#[derive(Debug)]
struct ExactHistorySource {
    target: ProductBranchObservation,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

/// Pinned genuine product basis and an owner-retained parent position.
/// Opening resolves one catalog entry. Each preparation inspects one parent
/// step, with constant additional position storage, including across forks.
#[derive(Debug)]
pub struct ProductExactHistoryCursor {
    source: Arc<ExactHistorySource>,
    position: Option<Arc<RetainedExactHistoryPosition>>,
}

/// Genuine component bases, without a fabricated historical branch reference.
#[derive(Debug)]
pub struct ProductExactHistoryPage {
    position: Option<Arc<RetainedExactHistoryPosition>>,
    complete: bool,
}

impl ProductExactHistoryPage {
    pub fn commit(&self) -> Option<&CompositeRuntimeWorldCommit> {
        self.position
            .as_deref()
            .map(RetainedExactHistoryPosition::commit)
    }
    pub fn source_visits(&self) -> usize {
        usize::from(self.position.is_some())
    }
    pub fn is_complete(&self) -> bool {
        self.complete
    }
}

#[derive(Debug)]
pub struct PreparedProductExactHistoryPage<'request, 'authority> {
    source: Arc<ExactHistorySource>,
    expected: Option<Arc<RetainedExactHistoryPosition>>,
    successor: Option<Arc<RetainedExactHistoryPosition>>,
    execution: worth_execution::ExecutionRequest<'request, 'authority>,
}

impl PreparedProductExactHistoryPage<'_, '_> {
    pub fn commit(&self) -> Option<&CompositeRuntimeWorldCommit> {
        self.expected
            .as_deref()
            .map(RetainedExactHistoryPosition::commit)
    }
    pub fn source_visits(&self) -> usize {
        usize::from(self.expected.is_some())
    }
    pub fn is_complete(&self) -> bool {
        self.successor.is_none()
    }
}

impl ProductExactHistoryCursor {
    /// Newly owned source and first-position backing; canonical commit custody is shared.
    pub const fn opening_backing_bytes() -> usize {
        Self::source_backing_bytes() + Self::transition_backing_bytes()
    }

    /// One new retained parent position; existing immutable sources remain shared.
    pub const fn transition_backing_bytes() -> usize {
        RetainedExactHistoryPosition::owned_backing_bytes()
    }

    const fn source_backing_bytes() -> usize {
        std::mem::size_of::<ExactHistorySource>() + 2 * std::mem::size_of::<usize>()
    }

    pub(crate) fn owner_issued(
        execution: worth_execution::ExecutionRequest<'_, '_>,
        target: ProductBranchObservation,
        position: Arc<RetainedExactHistoryPosition>,
    ) -> Result<Self, ProductExactHistoryDenial> {
        let memory = execution
            .reserve_retained_memory(Self::source_backing_bytes() as u64)
            .map_err(ProductExactHistoryDenial::Request)?;
        Ok(Self {
            source: Arc::new(ExactHistorySource {
                target,
                _memory: memory,
            }),
            position: Some(position),
        })
    }

    pub fn target(&self) -> &ProductBranchObservation {
        &self.source.target
    }

    pub(crate) fn position(&self) -> Option<&RetainedExactHistoryPosition> {
        self.position.as_deref()
    }

    pub(crate) fn prepared<'request, 'authority>(
        &self,
        execution: worth_execution::ExecutionRequest<'request, 'authority>,
        successor: Option<Arc<RetainedExactHistoryPosition>>,
    ) -> PreparedProductExactHistoryPage<'request, 'authority> {
        PreparedProductExactHistoryPage {
            source: Arc::clone(&self.source),
            expected: self.position.clone(),
            successor,
            execution,
        }
    }

    /// The original stays unchanged until its composition owner installs the
    /// returned successor after the aggregate's final request handoff.
    pub fn accept_page(
        &self,
        prepared: PreparedProductExactHistoryPage<'_, '_>,
    ) -> Result<(ProductExactHistoryPage, Self), ProductExactHistoryDenial> {
        let expected = match (&self.position, &prepared.expected) {
            (Some(actual), Some(expected)) => Arc::ptr_eq(actual, expected),
            (None, None) => true,
            _ => false,
        };
        if !Arc::ptr_eq(&self.source, &prepared.source) || !expected {
            return Err(ProductExactHistoryDenial::InvalidContinuation);
        }
        let (accepted, _) = prepared
            .execution
            .run_native(|_| {
                let page = ProductExactHistoryPage {
                    position: prepared.expected,
                    complete: prepared.successor.is_none(),
                };
                let successor = Self {
                    source: prepared.source,
                    position: prepared.successor,
                };
                (page, successor)
            })
            .map_err(ProductExactHistoryDenial::Request)?;
        Ok(accepted)
    }
}

//! One actual historical head or retired cleanup, shared by maintenance and final disposal.
use std::sync::{Arc, Mutex, MutexGuard};
use worth_query_installation::facade::ApplicationSchema;
use {
    crate::basis::WorthQueryProductBranch,
    crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchCloseReceipt,
    crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt,
    crate::domain_computation::primary_graph::WorthQueryApplicationProductBranchCleanupFailure,
    crate::domain_computation::primary_graph::WorthQueryApplicationProductBranchCloseDenial,
    crate::domain_computation::primary_graph::WorthQueryOwnedProductHead,
    crate::domain_computation::primary_graph::WorthQueryOwnedProductHeadCloseFailure,
    crate::domain_computation::request_execution::WorthQueryAdvancementPhase,
};

pub(super) struct HistoryHeadSlot<'r, S> {
    state: Mutex<HistoryHeadCustody<'r, S>>,
    _allocation: Option<worth_execution::ExecutionMemoryReservation>,
}

impl<'r, S> HistoryHeadSlot<'r, S> {
    pub(super) fn reserve(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, super::WorthQueryDocumentHistoryDenial> {
        let allocation = super::allocation::reserve_arc::<Self>(execution)
            .map_err(super::WorthQueryDocumentHistoryDenial::Recording)?;
        Ok(Arc::new(Self {
            state: Mutex::new(HistoryHeadCustody::Reserved),
            _allocation: allocation,
        }))
    }
    pub(super) fn lock(&self) -> MutexGuard<'_, HistoryHeadCustody<'r, S>> {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}

pub(super) enum HistoryHeadCustody<'r, S> {
    Reserved,
    Advancing,
    Closed(ClosedHistoryHead),
    Live(WorthQueryOwnedProductHead<'r, S>),
    Refused {
        head: WorthQueryOwnedProductHead<'r, S>,
        denial: WorthQueryApplicationProductBranchCloseDenial,
    },
    Retired {
        branch: WorthQueryProductBranch,
        failure: WorthQueryApplicationProductBranchCleanupFailure,
    },
}

pub(super) enum ClosedHistoryHead {
    Direct {
        _receipt: WorthQueryProductBranchCloseReceipt,
    },
    Cleanup {
        _receipt: WorthQueryProductBranchOwnerCleanupReceipt,
    },
}

impl<S> HistoryHeadCustody<'_, S> {
    pub(super) fn branch(&self) -> Option<WorthQueryProductBranch> {
        match self {
            Self::Reserved | Self::Advancing | Self::Closed(_) => None,
            Self::Live(head) | Self::Refused { head, .. } => Some(head.branch()),
            Self::Retired { branch, .. } => Some(*branch),
        }
    }
}

impl<'r, S: ApplicationSchema> HistoryHeadCustody<'r, S> {
    pub(super) fn retry(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<Option<ClosedHistoryHead>, Self> {
        match self {
            Self::Reserved => Ok(None),
            Self::Advancing => unreachable!("the actual slot mutex spans an entire transition"),
            Self::Closed(receipt) => Ok(Some(receipt)),
            Self::Live(head) | Self::Refused { head, .. } => match head.close(phase) {
                Ok(receipt) => Ok(Some(ClosedHistoryHead::Direct { _receipt: receipt })),
                Err(WorthQueryOwnedProductHeadCloseFailure::Retained { head, denial }) => {
                    Err(Self::Refused { head, denial })
                }
                Err(WorthQueryOwnedProductHeadCloseFailure::Retired { branch, failure }) => {
                    Err(Self::Retired { branch, failure })
                }
            },
            Self::Retired { branch, failure } => {
                match failure.into_cleanup().retry_in_advancement(phase) {
                    Ok(receipt) => Ok(Some(ClosedHistoryHead::Cleanup { _receipt: receipt })),
                    Err(failure) => Err(Self::Retired { branch, failure }),
                }
            }
        }
    }

    pub(super) fn advance(&mut self, phase: &WorthQueryAdvancementPhase<'_>) {
        if matches!(self, Self::Reserved | Self::Closed(_)) {
            return;
        }
        let original = std::mem::replace(self, Self::Advancing);
        *self = match original.retry(phase) {
            Ok(Some(receipt)) => Self::Closed(receipt),
            Ok(None) => unreachable!("a filled release slot owns an actual head"),
            Err(pending) => pending,
        };
    }
}

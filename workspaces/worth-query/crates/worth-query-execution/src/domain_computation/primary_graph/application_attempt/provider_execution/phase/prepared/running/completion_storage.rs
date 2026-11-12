//! Prepaid completion storage belongs to the running managed commit owner.
use super::progression::WorthQueryProgressedApplicationCommit;
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitOutcome,
};
use std::cell::Cell;
use worth_execution::{
    ExecutionArray, ExecutionRequest, ExecutionRequestArrayBuilder, ExecutionRequestArrayDenial,
};

pub struct ApplicationCommitCompletionStorage {
    storage: PhaseStorage<WorthQueryProgressedApplicationCommit>,
}
pub struct CompletedApplicationCommit {
    storage: PhaseStorage<WorthQueryProgressedApplicationCommit>,
}
impl ApplicationCommitCompletionStorage {
    pub(super) fn prepare(
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, WorthQueryApplicationCommitOutcome> {
        Ok(Self {
            storage: PhaseStorage::prepare(request)?,
        })
    }
    pub(super) fn complete(
        self,
        actual: WorthQueryProgressedApplicationCommit,
    ) -> CompletedApplicationCommit {
        CompletedApplicationCommit {
            storage: self.storage.fill(actual),
        }
    }
}
impl CompletedApplicationCommit {
    pub(super) fn consume(self) -> WorthQueryProgressedApplicationCommit {
        self.storage.consume()
    }
}

// Empty cells are allocated, charged and sealed while preparation may still refuse.
// Filling a cell after publication performs no consultation, allocation or refusal.
pub(in crate::domain_computation::primary_graph) struct PhaseStorage<T> {
    values: ExecutionArray<Cell<Option<T>>>,
}
impl<T> PhaseStorage<T> {
    pub(in crate::domain_computation::primary_graph) fn prepare(
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, WorthQueryApplicationCommitOutcome> {
        let mut builder = ExecutionRequestArrayBuilder::allocate(*request, 1).map_err(denied)?;
        builder.push(Cell::new(None)).map_err(denied)?;
        Ok(Self {
            values: builder.seal().map_err(denied)?,
        })
    }
    pub(in crate::domain_computation::primary_graph) fn fill(self, actual: T) -> Self {
        assert!(
            self.values.elements()[0].replace(Some(actual)).is_none(),
            "stage storage filled once"
        );
        self
    }
    pub(in crate::domain_computation::primary_graph) fn consume(self) -> T {
        self.values
            .into_iter()
            .next()
            .expect("one admitted stage cell")
            .into_inner()
            .expect("actual managed stage")
    }
}
fn denied(cause: ExecutionRequestArrayDenial) -> WorthQueryApplicationCommitOutcome {
    match cause {
        ExecutionRequestArrayDenial::Request(cause) => {
            use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
            match crate::domain_computation::primary_graph::application_contribution::publication_execution_kind(cause) {
                Ok(kind) => WorthQueryApplicationCommitOutcome::Denied(
                    WorthQueryApplicationCommitDenial::provider_execution_denied(
                        crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenialStage::ResourceAdmission,
                        Ok(kind), "managed commit completion backing refused before start",
                    ),
                ),
                Err(Control::Cancelled) => WorthQueryApplicationCommitOutcome::Cancelled,
                Err(Control::TimedOut) => WorthQueryApplicationCommitOutcome::TimedOut,
            }
        }
        ExecutionRequestArrayDenial::Allocation(cause) => {
            let failure =
                crate::domain_computation::WorthQueryProviderSessionFailure::allocation_denied(
                    cause,
                    "managed commit completion storage",
                );
            WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::provider_execution_denied(
                    crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenialStage::ResourceAdmission,
                    Ok(failure.kind()),
                    failure.detail(),
                )
                .with_provider_session_failure(failure),
            )
        }
    }
}

mod control;
pub use control::WorthQueryManagedPhaseStorageForTest;

impl ApplicationCommitCompletionStorage {
    #[doc(hidden)]
    pub fn complete_for_test(
        self,
        actual: WorthQueryProgressedApplicationCommit,
    ) -> CompletedApplicationCommit {
        self.complete(actual)
    }
}

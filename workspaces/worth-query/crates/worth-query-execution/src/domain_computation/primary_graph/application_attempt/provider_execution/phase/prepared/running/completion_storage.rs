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
struct PhaseStorage<T> {
    values: ExecutionArray<Cell<Option<T>>>,
}
impl<T> PhaseStorage<T> {
    fn prepare(
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, WorthQueryApplicationCommitOutcome> {
        let mut builder = ExecutionRequestArrayBuilder::allocate(*request, 1).map_err(denied)?;
        builder.push(Cell::new(None)).map_err(denied)?;
        Ok(Self {
            values: builder.seal().map_err(denied)?,
        })
    }
    fn fill(self, actual: T) -> Self {
        assert!(
            self.values.elements()[0].replace(Some(actual)).is_none(),
            "stage storage filled once"
        );
        self
    }
    fn consume(self) -> T {
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

#[cfg(test)]
mod tests {
    use super::*;
    use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};
    #[test]
    fn managed_phase_storage_refuses_the_original_grant_before_start() {
        let budget = SerialMemoryBudget::new(Some(0));
        let cancellation = CancellationSource::new();
        let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
        let Err(WorthQueryApplicationCommitOutcome::Denied(denial)) =
            PhaseStorage::<u64>::prepare(&ExecutionRequest::serial(&host))
        else {
            panic!("the original zero grant must refuse before start");
        };
        assert!(matches!(denial.kind(),
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenialKind::ExecutionResource { .. }
        ));
        assert_eq!(budget.charged_memory_bytes(), 0);
    }
    #[test]
    fn managed_phase_storage_retains_the_actual_value_and_releases_its_grant() {
        let bytes = std::mem::size_of::<Cell<Option<u64>>>() as u64;
        let budget = SerialMemoryBudget::new(Some(65_536));
        let cancellation = CancellationSource::new();
        let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
        let actual = PhaseStorage::prepare(&ExecutionRequest::serial(&host))
            .unwrap()
            .fill(37_u64);
        assert_eq!(budget.charged_memory_bytes(), bytes);
        assert_eq!(actual.consume(), 37);
        assert_eq!(budget.charged_memory_bytes(), 0);
    }
    #[test]
    fn cancelled_completion_preflight_is_cancelled_without_indeterminate_business() {
        let budget = SerialMemoryBudget::new(Some(65_536));
        let cancellation = CancellationSource::new();
        cancellation.cancel();
        let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
        assert!(matches!(
            PhaseStorage::<u64>::prepare(&ExecutionRequest::serial(&host)),
            Err(WorthQueryApplicationCommitOutcome::Cancelled)
        ));
        assert_eq!(budget.charged_memory_bytes(), 0);
    }
}

impl ApplicationCommitCompletionStorage {
    #[doc(hidden)]
    pub fn complete_for_test(
        self,
        actual: WorthQueryProgressedApplicationCommit,
    ) -> CompletedApplicationCommit {
        self.complete(actual)
    }
}

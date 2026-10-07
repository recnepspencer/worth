use crate::data::error::SignalError;
use crate::data::retained_storage::RetainedStoragePreparation;

/// Application callers name their work owner. Conditional execution always
/// borrows its existing attempt; ordinary evaluation retains its installed
/// waiter policy. Neither posture creates execution or publication authority.
pub(crate) enum EvaluationWork<'borrow, 'observer> {
    Ordinary,
    /// Ordinary execution borrows both owners from its admitted request.
    RequestPreparation {
        work: &'borrow mut RetainedStoragePreparation<'observer>,
        memory: &'borrow mut crate::data::request_preparation::SignalPreparationBudget,
    },
    Conditional(&'borrow mut RetainedStoragePreparation<'observer>),
    /// Read-only request preparation lends the actual kernel safe point.
    /// This grants no retained node mutation or waiter publication capability.
    RequestCheckpoint(&'borrow mut dyn FnMut(usize) -> Result<(), SignalError>),
}

impl EvaluationWork<'_, '_> {
    pub(crate) fn reserve(&mut self, visits: Option<usize>) -> Result<(), SignalError> {
        match self {
            Self::Ordinary => visits
                .map(|_| ())
                .ok_or_else(|| SignalError::internal("evaluation work bound overflow")),
            Self::RequestCheckpoint(checkpoint) => checkpoint(
                visits.ok_or_else(|| SignalError::internal("request work bound overflow"))?,
            ),
            Self::Conditional(work) | Self::RequestPreparation { work, .. } => {
                crate::data::conditional_execution::conditional_work::reserve(work, visits)
            }
        }
    }

    pub(crate) fn claim_preparation_vec<T>(&mut self, capacity: usize) -> Result<(), SignalError> {
        match self {
            Self::RequestPreparation { memory, .. } => memory.claim_vec::<T>(capacity),
            Self::Ordinary | Self::Conditional(_) | Self::RequestCheckpoint(_) => Ok(()),
        }
    }

    pub(crate) fn with_waiter_limit<R>(
        &mut self,
        maximum_waiter_visits: usize,
        prepare: impl FnOnce(&mut RetainedStoragePreparation) -> Result<R, SignalError>,
    ) -> Result<R, SignalError> {
        match self {
            Self::Ordinary => prepare(&mut RetainedStoragePreparation::new(maximum_waiter_visits)),
            Self::RequestCheckpoint(_) => Err(SignalError::internal(
                "request discovery checkpoint cannot prepare waiter publication",
            )),
            Self::Conditional(work) | Self::RequestPreparation { work, .. } => {
                let maximum_attempt_visits = work.maximum_visits();
                let mut limit = work.limit_additional_visits(maximum_waiter_visits);
                let outer_limited = limit.outer_limited();
                prepare(&mut limit).map_err(|error| match error {
                    SignalError::WaiterResolutionWorkExhausted { .. } if outer_limited => {
                        SignalError::ConditionalEvaluationWorkExhausted {
                            maximum_visits: maximum_attempt_visits,
                        }
                    }
                    SignalError::WaiterResolutionWorkExhausted { .. } => {
                        SignalError::WaiterResolutionWorkExhausted {
                            maximum_visits: maximum_waiter_visits,
                        }
                    }
                    error => error,
                })
            }
        }
    }
}

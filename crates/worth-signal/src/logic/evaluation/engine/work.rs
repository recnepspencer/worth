use crate::data::error::SignalError;
use crate::data::retained_storage::RetainedStoragePreparation;

/// Application callers name their work owner. Conditional execution always
/// borrows its existing attempt; ordinary evaluation retains its installed
/// waiter policy. Neither posture creates execution or publication authority.
pub(crate) enum EvaluationWork<'a> {
    Ordinary,
    Conditional(&'a mut RetainedStoragePreparation),
}

impl EvaluationWork<'_> {
    pub(crate) fn reserve(&mut self, visits: Option<usize>) -> Result<(), SignalError> {
        match self {
            Self::Ordinary => visits
                .map(|_| ())
                .ok_or_else(|| SignalError::internal("evaluation work bound overflow")),
            Self::Conditional(work) => {
                crate::data::conditional_execution::conditional_work::reserve(work, visits)
            }
        }
    }

    pub(crate) fn with_waiter_limit<R>(
        &mut self,
        maximum_waiter_visits: usize,
        prepare: impl FnOnce(&mut RetainedStoragePreparation) -> Result<R, SignalError>,
    ) -> Result<R, SignalError> {
        match self {
            Self::Ordinary => prepare(&mut RetainedStoragePreparation::new(maximum_waiter_visits)),
            Self::Conditional(work) => {
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

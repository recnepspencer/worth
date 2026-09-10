use std::panic::{catch_unwind, AssertUnwindSafe};

#[cfg(test)]
mod tests;

use crate::data::conditional_execution::SignalConditionalAttemptOutcome;
use crate::data::graph::SignalGraph;
use crate::logic::transaction::SignalObservationSession;

use super::{
    ActivatedConditionalOutcome, SignalPartitionConditionalCompletion,
    SignalPartitionConditionalUnwindReason,
};

/// Finish and drop an acquired session without allowing a secondary panic to
/// destroy earlier evidence. The caller restores graph custody before resuming.
pub(super) fn finish_attempt(
    graph: &SignalGraph,
    observation: SignalObservationSession,
    attempt: SignalConditionalAttemptOutcome,
    work: &mut crate::data::retained_storage::RetainedStoragePreparation,
) -> ActivatedConditionalOutcome {
    let finished = catch_unwind(AssertUnwindSafe(|| {
        graph.finish_optional_observation_session_with_work(
            &observation,
            &mut crate::logic::evaluation::EvaluationWork::Conditional(work),
        )
    }));
    let cleanup = catch_unwind(AssertUnwindSafe(|| drop(observation)));
    match attempt {
        SignalConditionalAttemptOutcome::Unwound { payload, counters } => {
            ActivatedConditionalOutcome::Unwound {
                payload,
                report: SignalPartitionConditionalUnwindReason::Execution {
                    counters,
                    observation: finished,
                    cleanup,
                },
            }
        }
        SignalConditionalAttemptOutcome::Completed(decision) => match finished {
            Err(payload) => ActivatedConditionalOutcome::Unwound {
                payload,
                report: SignalPartitionConditionalUnwindReason::Observation { decision, cleanup },
            },
            Ok(observation) => {
                let completion = SignalPartitionConditionalCompletion {
                    decision,
                    observation,
                    rejected: None,
                };
                match cleanup {
                    Ok(()) => ActivatedConditionalOutcome::Completed(completion),
                    Err(payload) => ActivatedConditionalOutcome::Unwound {
                        payload,
                        report: SignalPartitionConditionalUnwindReason::Cleanup { completion },
                    },
                }
            }
        },
    }
}

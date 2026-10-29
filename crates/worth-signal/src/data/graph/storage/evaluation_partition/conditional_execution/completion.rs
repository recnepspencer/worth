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
    let mut observation = Some(observation);
    let mut finished_session = None;
    let finished = crate::panic_boundary::catch_operation(&mut || {
        finished_session = Some(
            graph.finish_optional_observation_session_with_work(
                observation
                    .as_ref()
                    .expect("observation remains for finish"),
                &mut crate::logic::evaluation::EvaluationWork::Conditional(work),
            ),
        );
    })
    .map(|()| finished_session.expect("observation finish returned"));
    let cleanup = crate::panic_boundary::catch_operation(&mut || {
        drop(observation.take().expect("one observation drop"));
    });
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

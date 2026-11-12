//! Named progression across the exact retained recovery states.
use super::*;
impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(super) fn continue_inbound_recovery_state(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,

        accepted: &Arc<WorthQueryAcceptedInboundOccurrence>,
        state: State,
        request: &WorthQueryRequestScope,
        retirements: &mut OwnerCleanupRetirements,
    ) -> RecoveryStep {
        match state {
            State::Recorded(mut continuation) => {
                if let Some(denial) = self.inbound_source_consumption_denial(accepted.operation()) {
                    return RecoveryStep::Retain(State::Recorded(continuation), Err(denial));
                }
                match continuation.advance(self, execution, retirements) {
                    Ok(super::super::super::inbound_publication::recovered::RecordedCompletionProgress::Completed(terminal)) => {
                        let released = self
                            .runtime
                            .primary_provider
                            .release_recorded_terminal_dispatch_provenance(
                                accepted.owner(),
                                &terminal,
                            );
                        if released.is_err()
                            || self
                                .acknowledge_retained_inbound_terminal(execution, accepted)
                                .is_err()
                        {
                            return RecoveryStep::Retain(
                                State::Recorded(continuation),
                                Err(Denial::TerminalCleanupUnavailable),
                            );
                        }
                        RecoveryStep::RecordedTerminal(terminal.completion_world_commit().clone())
                    }
                    Ok(super::super::super::inbound_publication::recovered::RecordedCompletionProgress::StaleReleased) => RecoveryStep::StaleReleased,
                    Err(cause) => RecoveryStep::Retain(
                        State::Recorded(continuation),
                        Err(cause.into_inbound_denial()),
                    ),
                }
            }
            State::Active(recovery) => self.continue_active_inbound_recovery(
                execution,
                accepted,
                recovery,
                request,
                retirements,
            ),
            State::HandoffWorld { old, next } => {
                self.finish_inbound_handoff(execution, old, next, retirements)
            }
            State::HandoffCleanup { cleanup, next } => match cleanup
                .retry_in_request_with_retirements(execution, retirements)
            {
                Ok(_) => RecoveryStep::Retain(State::Active(next), Ok(Posture::AcceptedPending)),
                Err(failed) => RecoveryStep::Retain(
                    State::HandoffCleanup {
                        cleanup: failed.into_cleanup(),
                        next,
                    },
                    Err(Denial::RecoveryUnavailable),
                ),
            },
            State::PublishedWorld { old, terminal } => {
                self.finish_inbound_published(execution, old, terminal, retirements)
            }
            State::PublishedCleanup { cleanup, terminal } => {
                match cleanup.retry_in_request_with_retirements(execution, retirements) {
                    Ok(_) => RecoveryStep::Terminal(terminal),
                    Err(failed) => RecoveryStep::Retain(
                        State::PublishedCleanup {
                            cleanup: failed.into_cleanup(),
                            terminal,
                        },
                        Err(Denial::RecoveryUnavailable),
                    ),
                }
            }
            State::StaleWorld(old) => self.finish_inbound_stale(execution, old, retirements),
            State::StaleCleanup(cleanup) => {
                match cleanup.retry_in_request_with_retirements(execution, retirements) {
                    Ok(_) => RecoveryStep::StaleReleased,
                    Err(failed) => RecoveryStep::Retain(
                        State::StaleCleanup(failed.into_cleanup()),
                        Err(Denial::RecoveryUnavailable),
                    ),
                }
            }
        }
    }
}

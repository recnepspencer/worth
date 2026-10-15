//! One fresh phase delegates the existing native command and publication owners.
use super::super::operation::native_publication::{native_retry_eligible, NativeKeyedProgress};
use super::*;

enum Step {
    Progress,
    Completed(WorthQueryCapabilityCommandCompletion),
    Terminal,
}
impl<'r, S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>, D>
    WorthQueryCapabilityCommand<'r, S, P, D>
where
    WorthQueryProgramApplicationRuntime<S, P>: Send + Sync + 'static,
{
    /// Advances original meaning using only this request's fresh native authority.
    pub fn advance(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> WorthQueryCapabilityCommandAdvance<
        Self,
        WorthQueryCompletedCapabilityCommand<'r, S, P, D>,
        WorthQueryTerminalCapabilityCommand<'r, S, P, D>,
        D,
    > {
        if !self.matches(request) {
            return WorthQueryCapabilityCommandAdvance::Refused {
                command: self,
                denial: WorthQueryCapabilityCommandDenial::FreshRequestMismatch,
            };
        }
        let keepalive = self.runtime.clone();
        let result = keepalive
            .runtime()
            .with_application_advancement(request.scope, |phase| self.step(&phase, request));
        match result {
            Ok(Ok(Step::Progress)) => WorthQueryCapabilityCommandAdvance::Progress(self),
            Ok(Ok(Step::Completed(evidence))) => WorthQueryCapabilityCommandAdvance::Completed(
                WorthQueryCompletedCapabilityCommand {
                    command: self,
                    evidence,
                },
            ),
            Ok(Ok(Step::Terminal)) => {
                WorthQueryCapabilityCommandAdvance::Terminal(WorthQueryTerminalCapabilityCommand {
                    command: self,
                })
            }
            Ok(Err(denial)) => WorthQueryCapabilityCommandAdvance::Refused {
                command: self,
                denial,
            },
            Err(denial) => {
                self.state.record_unresolved_advancement(denial);
                WorthQueryCapabilityCommandAdvance::Refused {
                    command: self,
                    denial: WorthQueryCapabilityCommandDenial::Execution(denial),
                }
            }
        }
    }
    fn step(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<Step, WorthQueryCapabilityCommandDenial<D>> {
        use NativePublicationState as State;
        let action = match &self.state {
            State::Initial | State::RetryableNoEffect(_) => NativeCommandAction::Execute,
            State::Unpublished(recovery) => {
                self.cleanup
                    .reserve_release_failure()
                    .map_err(WorthQueryCapabilityCommandDenial::Cleanup)?;
                NativeCommandAction::Recover(recovery)
            }
            State::Performed(performed) => {
                self.cleanup
                    .absorb_recovered(performed)
                    .map_err(WorthQueryCapabilityCommandDenial::Cleanup)?;
                NativeCommandAction::Resolve
            }
            State::Keyed(WorthQueryApplicationUncommitted::SettlementDeferred(deferred)) => {
                NativeCommandAction::Settle(deferred)
            }
            State::Keyed(_) | State::UnresolvedAdvancement(_) => NativeCommandAction::Resolve,
            State::Stopped(_) => return Ok(Step::Terminal),
            State::AdmissionStopped(never) | State::RetainedPending(never) => match *never {},
        };
        let result = self
            .backend
            .run(phase, request, action)
            .map_err(WorthQueryCapabilityCommandDenial::Native)?;
        match result {
            NativeCommandResult::Commit(outcome) => self.accept_commit(outcome),
            NativeCommandResult::Recovered(outcome) => {
                self.state.accept_recovery(outcome, &mut self.cleanup);
                Ok(Step::Progress)
            }
            NativeCommandResult::Resolved(read) => match self
                .state
                .accept_keyed(read.into_resolution(), &self.cleanup)
                .map_err(WorthQueryCapabilityCommandDenial::Cleanup)?
            {
                NativeKeyedProgress::Progress { .. } => Ok(Step::Progress),
                NativeKeyedProgress::Completed(evidence) => Ok(Step::Completed(
                    WorthQueryCapabilityCommandCompletion::recovered(evidence),
                )),
                NativeKeyedProgress::Terminal(resolution) => {
                    self.keyed_terminal = Some(resolution);
                    Ok(Step::Terminal)
                }
            },
            NativeCommandResult::Validated => unreachable!("validation does not execute a command"),
        }
    }
    fn accept_commit(
        &mut self,
        outcome: WorthQueryApplicationCommitOutcome,
    ) -> Result<Step, WorthQueryCapabilityCommandDenial<D>> {
        let outcome = self.cleanup.absorb_command_outcome(&self.runtime, outcome);
        match outcome.landed() {
            Ok((receipt, false)) => Ok(Step::Completed(
                WorthQueryCapabilityCommandCompletion::Fresh { receipt },
            )),
            Ok((receipt, true))
                if matches!(
                    self.state,
                    NativePublicationState::Initial | NativePublicationState::RetryableNoEffect(_)
                ) =>
            {
                Ok(Step::Completed(
                    WorthQueryCapabilityCommandCompletion::Recovered {
                        receipt,
                        publication: None,
                        initial_posture: None,
                        initial_advancement_failure: None,
                    },
                ))
            }
            Ok((receipt, true)) => match self
                .state
                .accept_keyed(
                    WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(receipt),
                    &self.cleanup,
                )
                .map_err(WorthQueryCapabilityCommandDenial::Cleanup)?
            {
                NativeKeyedProgress::Completed(evidence) => Ok(Step::Completed(
                    WorthQueryCapabilityCommandCompletion::recovered(evidence),
                )),
                _ => unreachable!("a genuine committed receipt completes keyed settlement"),
            },
            Err(WorthQueryApplicationUncommitted::SettlementDeferred(deferred)) => {
                self.state = NativePublicationState::Keyed(
                    WorthQueryApplicationUncommitted::SettlementDeferred(deferred),
                );
                Ok(Step::Progress)
            }
            // These native command outcomes prove publication did not occur. A fresh
            // attempt rebuilds only the command projection, never a performed effect.
            Err(
                posture @ (WorthQueryApplicationUncommitted::Deferred(_)
                | WorthQueryApplicationUncommitted::ProductStale(_)),
            ) => {
                self.state = NativePublicationState::RetryableNoEffect(posture);
                Ok(Step::Progress)
            }
            Err(posture) => match self.state.accept_uncommitted(posture) {
                Ok(()) => Ok(Step::Progress),
                Err(posture) if native_retry_eligible(&posture) => {
                    self.state = NativePublicationState::RetryableNoEffect(posture);
                    Ok(Step::Progress)
                }
                Err(posture) => {
                    self.state = NativePublicationState::Stopped(posture);
                    Ok(Step::Terminal)
                }
            },
        }
    }
}

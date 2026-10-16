//! Product-owned continuation keeps the real operation and its original presentation purpose.
use super::error::PlatformPulseQueryActionIndeterminate;
use super::*;
use std::num::NonZeroUsize;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_ui::facade::query_binding::{
    WorthUiScalarProjectionActionPreconditionDenial, WorthUiStatusActionOutcome,
    WorthUiStatusMutation, WorthUiStatusMutationClose, WorthUiStatusMutationResult,
};

#[cfg(test)]
use worth_ui::facade::query_binding::WorthUiStatusMutationStop;

#[derive(Debug)]
pub(super) enum PlatformPulseQueryMutationPurpose {
    Source,
    CurrentAction,
    DeniedAction,
}

#[must_use = "retain the original mutation until fresh-request continuation or lawful close"]
#[derive(Debug)]
pub(super) struct PlatformPulseQueryMutation {
    operation: WorthUiStatusMutation,
    purpose: PlatformPulseQueryMutationPurpose,
}
impl PlatformPulseQueryMutation {
    pub(super) fn new(
        operation: WorthUiStatusMutation,
        purpose: PlatformPulseQueryMutationPurpose,
    ) -> Self {
        Self { operation, purpose }
    }
    #[cfg(test)]
    pub(super) fn purpose(&self) -> &PlatformPulseQueryMutationPurpose {
        &self.purpose
    }
    #[cfg(test)]
    pub(super) fn stop(&self) -> Option<&WorthUiStatusMutationStop> {
        self.operation.stop()
    }
    pub(super) fn advance(
        self,
        scope: &WorthQueryRequestScope,
        history: NonZeroUsize,
    ) -> Result<
        (
            PlatformPulseQueryMutationPurpose,
            WorthUiStatusMutationResult,
        ),
        Self,
    > {
        match self.operation.advance(scope, history) {
            Ok(result) => Ok((self.purpose, result)),
            Err(operation) => Err(Self {
                operation,
                purpose: self.purpose,
            }),
        }
    }
    pub(super) fn close_without_commit(
        self,
        scope: &WorthQueryRequestScope,
    ) -> Result<WorthUiStatusMutationClose, Self> {
        match self.operation.close_without_commit(scope) {
            Ok(closed) => Ok(closed),
            Err(operation) => Err(Self {
                operation,
                purpose: self.purpose,
            }),
        }
    }
}
pub(crate) enum PlatformPulseQueryContinuation {
    Source(UiProjectionObservation),
    Action(PlatformPulseQueryActionOutcome),
}
impl PlatformPulseQueryLifecycle {
    /// Cancels only when the actual original native owner proves a no-commit close.
    /// A committed or unresolved operation remains owned for normal continuation.
    pub(crate) fn cancel_pending_mutation(
        &mut self,
    ) -> Result<(), PlatformPulseQueryLifecycleDenial> {
        let state = std::mem::replace(&mut self.state, PlatformPulseQueryOwnerState::Indeterminate);
        let PlatformPulseQueryOwnerState::Mutation(mutation) = state else {
            self.state = state;
            return Err(PlatformPulseQueryLifecycleDenial::OwnerNotLive);
        };
        match mutation.close_without_commit(&super::fresh_query_scope()) {
            Err(mutation) => {
                self.state = PlatformPulseQueryOwnerState::Mutation(mutation);
                Err(PlatformPulseQueryLifecycleDenial::MutationPending)
            }
            Ok(closed) => {
                if matches!(closed.live, Some(worth_query_host::facade::primary_graph::WorthQueryApplicationLiveCloseOutcome::Unavailable)) {
                    return Err(PlatformPulseQueryLifecycleDenial::IndeterminateAtClose);
                }
                self.state = PlatformPulseQueryOwnerState::Live;
                Ok(())
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn mutation_stop(&self) -> Option<&WorthUiStatusMutationStop> {
        match &self.state {
            PlatformPulseQueryOwnerState::Mutation(mutation) => mutation.stop(),
            _ => None,
        }
    }
    /// Exactly one original-owner step under new authentication. Refusal restores
    /// this same product stage; no source command, key or presentation purpose is rebuilt.
    pub(crate) fn continue_mutation(
        &mut self,
        history: NonZeroUsize,
    ) -> Result<PlatformPulseQueryContinuation, PlatformPulseQueryLifecycleDenial> {
        let state = std::mem::replace(&mut self.state, PlatformPulseQueryOwnerState::Indeterminate);
        let PlatformPulseQueryOwnerState::Mutation(mutation) = state else {
            self.state = state;
            return Err(PlatformPulseQueryLifecycleDenial::OwnerNotLive);
        };
        let scope = self.next_request_scope();
        let (purpose, result) = match mutation.advance(&scope, history) {
            Ok(result) => result,
            Err(mutation) => {
                self.state = PlatformPulseQueryOwnerState::Mutation(mutation);
                return Err(PlatformPulseQueryLifecycleDenial::MutationPending);
            }
        };
        match (purpose, result) {
            (
                PlatformPulseQueryMutationPurpose::Source,
                WorthUiStatusMutationResult::Source(publication),
            ) => {
                let observation = publication
                    .into_projection_observation()
                    .map_err(PlatformPulseQueryLifecycleDenial::Projection)?;
                self.retain_awaiting(&observation);
                Ok(PlatformPulseQueryContinuation::Source(observation))
            }
            (
                PlatformPulseQueryMutationPurpose::CurrentAction,
                WorthUiStatusMutationResult::Action(WorthUiStatusActionOutcome::Committed(
                    execution,
                )),
            ) => {
                let (publication, evidence) = execution.into_parts();
                let observation = publication
                    .into_projection_observation()
                    .map_err(PlatformPulseQueryLifecycleDenial::Projection)?;
                self.retain_awaiting(&observation);
                Ok(PlatformPulseQueryContinuation::Action(
                    PlatformPulseQueryActionOutcome::Executed {
                        evidence,
                        observation,
                    },
                ))
            }
            (
                PlatformPulseQueryMutationPurpose::DeniedAction,
                WorthUiStatusMutationResult::Action(
                    WorthUiStatusActionOutcome::DeniedRevisionMismatch {
                        active_revision,
                        submitted_revision,
                        ..
                    },
                ),
            ) => {
                self.state = PlatformPulseQueryOwnerState::Live;
                Ok(PlatformPulseQueryContinuation::Action(
                    PlatformPulseQueryActionOutcome::Denied {
                        denial:
                            WorthUiScalarProjectionActionPreconditionDenial::SourceRevisionMismatch,
                        active_query_source_revision: active_revision,
                        submitted_query_source_revision: submitted_revision,
                    },
                ))
            }
            (
                PlatformPulseQueryMutationPurpose::CurrentAction,
                WorthUiStatusMutationResult::Action(
                    WorthUiStatusActionOutcome::DeniedRevisionMismatch { .. },
                ),
            ) => Ok(PlatformPulseQueryContinuation::Action(
                PlatformPulseQueryActionOutcome::Indeterminate(
                    PlatformPulseQueryActionIndeterminate::CurrentRevisionLost,
                ),
            )),
            (
                PlatformPulseQueryMutationPurpose::DeniedAction,
                WorthUiStatusMutationResult::Action(WorthUiStatusActionOutcome::Committed(_)),
            ) => Ok(PlatformPulseQueryContinuation::Action(
                PlatformPulseQueryActionOutcome::Indeterminate(
                    PlatformPulseQueryActionIndeterminate::DeniedActionCommitted,
                ),
            )),
            _ => unreachable!(
                "original product purpose and native input family are retained together"
            ),
        }
    }
}

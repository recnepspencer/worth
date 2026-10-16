mod actions;
mod continuation;
mod error;
pub(crate) use continuation::PlatformPulseQueryContinuation;
use continuation::{PlatformPulseQueryMutation, PlatformPulseQueryMutationPurpose};
pub(crate) use error::{PlatformPulseQueryActionOutcome, PlatformPulseQueryLifecycleDenial};

use worth_query_host::facade::publication::domain_computation::WorthQueryApplicationQueryPublicationReceipt;
use worth_ui::facade::query_binding::{
    UiApplicationScalarProjectionObservation, UiProjectionObservation,
    WorthUiScalarProjectionSourceRecord,
    WorthUiStatusSourceOwner,
};

pub(crate) struct PlatformPulseQueryLifecycle {
    owner: WorthUiStatusSourceOwner,
    state: PlatformPulseQueryOwnerState,
    #[cfg(test)]
    next_attempt_scope_for_test: Option<worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope>,
}

enum PlatformPulseQueryOwnerState {
    BeforeInitial(UiProjectionObservation),
    AwaitingPublication(ExpectedPublication),
    Live,
    Indeterminate,
    Mutation(PlatformPulseQueryMutation),
    Closed,
}

struct ExpectedPublication {
    identity: String,
    status: String,
    revision: u64,
    owner_order: u64,
    receipt: WorthQueryApplicationQueryPublicationReceipt,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlatformPulseQueryShutdownReceipt {
    owner_terminal: bool,
}

impl PlatformPulseQueryLifecycle {
    pub(crate) fn new(owner: WorthUiStatusSourceOwner, initial: UiProjectionObservation) -> Self {
        Self {
            owner,
            state: PlatformPulseQueryOwnerState::BeforeInitial(initial),
            #[cfg(test)]
            next_attempt_scope_for_test: None,
        }
    }

    pub(crate) fn issue_initial(
        &mut self,
    ) -> Result<UiProjectionObservation, PlatformPulseQueryLifecycleDenial> {
        let initial = match std::mem::replace(&mut self.state, PlatformPulseQueryOwnerState::Closed)
        {
            PlatformPulseQueryOwnerState::BeforeInitial(initial) => initial,
            state => {
                self.state = state;
                return Err(PlatformPulseQueryLifecycleDenial::InitialAlreadyIssued);
            }
        };
        self.retain_awaiting(&initial);
        Ok(initial)
    }

    pub(crate) fn advance(
        &mut self,
        record: WorthUiScalarProjectionSourceRecord,
    ) -> Result<UiProjectionObservation, PlatformPulseQueryLifecycleDenial> {
        self.require_live()?;
        let scope = self.next_request_scope();
        let publication = match self.owner.prepare_source(record, &scope).and_then(|original| original.advance(&scope, std::num::NonZeroUsize::MAX)) {
            Ok(worth_ui::facade::query_binding::WorthUiStatusMutationResult::Source(publication)) => publication,
            Ok(_) => unreachable!("original source construction retains its family"),
            Err(operation) => {
                self.state = PlatformPulseQueryOwnerState::Mutation(PlatformPulseQueryMutation::new(operation, PlatformPulseQueryMutationPurpose::Source));
                return Err(PlatformPulseQueryLifecycleDenial::MutationPending);
            }
        };
        let observation = publication
            .into_projection_observation()
            .map_err(PlatformPulseQueryLifecycleDenial::Projection)?;
        self.retain_awaiting(&observation);
        Ok(observation)
    }

    fn next_request_scope(&mut self) -> worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope {
        #[cfg(test)]
        if let Some(scope) = self.next_attempt_scope_for_test.take() { return scope; }
        fresh_query_scope()
    }

    #[cfg(test)]
    pub(crate) fn cancel_next_attempt_for_test(&mut self) {
        use worth_query_host::facade::admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope};
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(None, cancellation.token());
        cancellation.cancel();
        self.next_attempt_scope_for_test = Some(scope);
    }

    #[cfg(test)]
    pub(crate) fn authentication_cancelled_for_test(&self) -> bool {
        matches!(self.mutation_stop(), Some(worth_ui::facade::query_binding::WorthUiStatusMutationStop::Setup(worth_ui::facade::query_binding::WorthUiStatusOwnerError::Authentication(denial))) if denial.kind() == worth_query_host::facade::admission::authenticated_principal::WorthQueryAuthenticationDenialKind::Cancelled)
    }

    pub(crate) fn has_pending_mutation(&self) -> bool {
        matches!(self.state, PlatformPulseQueryOwnerState::Mutation(_))
    }

    pub(crate) fn has_pending_publication(&self) -> bool {
        matches!(self.state, PlatformPulseQueryOwnerState::AwaitingPublication(_))
    }

    pub(crate) fn accepts_command(&self) -> bool {
        matches!(self.state, PlatformPulseQueryOwnerState::Live)
    }

    fn require_live(&self) -> Result<(), PlatformPulseQueryLifecycleDenial> {
        match self.state {
            PlatformPulseQueryOwnerState::Live => Ok(()),
            PlatformPulseQueryOwnerState::AwaitingPublication(_) => {
                Err(PlatformPulseQueryLifecycleDenial::PublicationAlreadyPending)
            }
            _ => Err(PlatformPulseQueryLifecycleDenial::OwnerNotLive),
        }
    }

    fn retain_awaiting(&mut self, observation: &UiProjectionObservation) {
        let UiProjectionObservation::ApplicationScalar(observation) = observation else {
            unreachable!("the authored status owner issues only application scalar facts")
        };
        self.state = PlatformPulseQueryOwnerState::AwaitingPublication(
            ExpectedPublication::from_observation(observation),
        );
    }

    pub(crate) fn admit_publication(
        &mut self,
        observation: UiApplicationScalarProjectionObservation,
    ) -> Result<(), PlatformPulseQueryLifecycleDenial> {
        let state = std::mem::replace(&mut self.state, PlatformPulseQueryOwnerState::Closed);
        let PlatformPulseQueryOwnerState::AwaitingPublication(expected) = state else {
            self.state = state;
            return Err(PlatformPulseQueryLifecycleDenial::PublicationNotPending);
        };
        if !expected.admits(&observation) {
            self.state = PlatformPulseQueryOwnerState::AwaitingPublication(expected);
            return Err(PlatformPulseQueryLifecycleDenial::ForeignPublication);
        }
        self.state = PlatformPulseQueryOwnerState::Live;
        Ok(())
    }

    pub(crate) fn close(self) -> Result<PlatformPulseQueryShutdownReceipt, (Box<Self>, PlatformPulseQueryLifecycleDenial)> {
        self.close_with_scope(&fresh_query_scope())
    }

    /// Every refusal returns this whole owner, including its presentation state.
    /// Closing never advances a user command or turns cleanup into delivery evidence.
    pub(super) fn close_with_scope(mut self, scope: &worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope) -> Result<PlatformPulseQueryShutdownReceipt, (Box<Self>, PlatformPulseQueryLifecycleDenial)> {
        let state = std::mem::replace(&mut self.state, PlatformPulseQueryOwnerState::Closed);
        match state {
            PlatformPulseQueryOwnerState::Live => {}
            PlatformPulseQueryOwnerState::Mutation(mutation) => {
                match mutation.close_without_commit(scope) {
                    Err(mutation) => {
                        self.state = PlatformPulseQueryOwnerState::Mutation(mutation);
                        return Err((Box::new(self), PlatformPulseQueryLifecycleDenial::MutationPending));
                    }
                    Ok(closed) => {
                        if matches!(closed.live, Some(worth_query_host::facade::primary_graph::WorthQueryApplicationLiveCloseOutcome::Unavailable)) {
                            self.state = PlatformPulseQueryOwnerState::Indeterminate;
                            return Err((Box::new(self), PlatformPulseQueryLifecycleDenial::IndeterminateAtClose));
                        }
                    }
                }
            }
            state => {
                let denial = match state {
                    PlatformPulseQueryOwnerState::AwaitingPublication(_) => PlatformPulseQueryLifecycleDenial::UnsettledPublicationAtClose,
                    PlatformPulseQueryOwnerState::Indeterminate => PlatformPulseQueryLifecycleDenial::IndeterminateAtClose,
                    _ => PlatformPulseQueryLifecycleDenial::OwnerNotLive,
                };
                self.state = state;
                return Err((Box::new(self), denial));
            }
        }
        let receipt = self.owner.close();
        Ok(PlatformPulseQueryShutdownReceipt { owner_terminal: receipt.owner_terminal() })
    }

}

impl ExpectedPublication {
    fn from_observation(observation: &UiApplicationScalarProjectionObservation) -> Self {
        let fact = observation.fact();
        Self {
            identity: fact.value().identity.clone(),
            status: fact.value().status.clone(),
            revision: fact.value().revision,
            owner_order: fact.owner_order(),
            receipt: fact.query_receipt().clone(),
        }
    }

    fn admits(&self, observation: &UiApplicationScalarProjectionObservation) -> bool {
        let fact = observation.fact();
        self.identity == fact.value().identity
            && self.status == fact.value().status
            && self.revision == fact.value().revision
            && self.owner_order == fact.owner_order()
            && self.receipt == *fact.query_receipt()
    }
}

impl PlatformPulseQueryShutdownReceipt {
    pub(crate) const fn owner_terminal(self) -> bool {
        self.owner_terminal
    }
}

#[cfg(test)]
mod tests;

/// A new unprivileged request scope for each Query mutation or close attempt.
fn fresh_query_scope() -> worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope {
    use worth_query_host::facade::admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope};
    WorthQueryRequestScope::new(None, WorthQueryCancellationSource::new().token())
}

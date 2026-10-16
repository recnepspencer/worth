mod actions;
mod continuation;
mod error;
pub(crate) use continuation::{PlatformPulseQueryContinuation, PlatformPulseQueryMutation, PlatformPulseQueryMutationPurpose};
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
}

enum PlatformPulseQueryOwnerState {
    BeforeInitial(UiProjectionObservation),
    AwaitingPublication(ExpectedPublication),
    Live,
    Indeterminate,
    Mutation(continuation::PlatformPulseQueryMutation),
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
        let publication = match self.owner.publish_source(record, std::num::NonZeroUsize::MAX) {
            Ok(publication) => publication,
            Err(operation) => {
                self.state = PlatformPulseQueryOwnerState::Mutation(continuation::PlatformPulseQueryMutation::new(operation, continuation::PlatformPulseQueryMutationPurpose::Source));
                return Err(PlatformPulseQueryLifecycleDenial::MutationPending);
            }
        };
        let observation = publication
            .into_projection_observation()
            .map_err(PlatformPulseQueryLifecycleDenial::Projection)?;
        self.retain_awaiting(&observation);
        Ok(observation)
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

    pub(crate) fn close(
        self,
    ) -> Result<PlatformPulseQueryShutdownReceipt, PlatformPulseQueryLifecycleDenial> {
        match self.state {
            PlatformPulseQueryOwnerState::Live => {}
            PlatformPulseQueryOwnerState::Mutation(mutation) => {
                return Err(PlatformPulseQueryLifecycleDenial::RetainedMutation(Box::new(mutation)));
            }
            PlatformPulseQueryOwnerState::AwaitingPublication(_) => {
                return Err(PlatformPulseQueryLifecycleDenial::UnsettledPublicationAtClose);
            }
            PlatformPulseQueryOwnerState::Indeterminate => {
                return Err(PlatformPulseQueryLifecycleDenial::IndeterminateAtClose);
            }
            _ => return Err(PlatformPulseQueryLifecycleDenial::OwnerNotLive),
        }
        let owner = self.owner;
        let receipt = owner.close();
        Ok(PlatformPulseQueryShutdownReceipt {
            owner_terminal: receipt.owner_terminal(),
        })
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

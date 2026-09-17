use worth_query_host::facade::publication::domain_computation::WorthQueryApplicationQueryPublicationReceipt;
use worth_ui::facade::query_binding::{
    UiApplicationScalarProjectionObservation, UiProjectionObservation,
    WorthUiQueryViewIdentityError, WorthUiScalarProjectionActionEvidence,
    WorthUiScalarProjectionActionPreconditionDenial, WorthUiScalarProjectionSourceRecord,
    WorthUiStatusActionOutcome, WorthUiStatusActionRequest, WorthUiStatusOwnerError,
    WorthUiStatusSourceOwner,
};

pub(crate) struct PlatformPulseQueryLifecycle {
    owner: WorthUiStatusSourceOwner,
    initial: Option<UiProjectionObservation>,
    state: PlatformPulseQueryOwnerState,
}

enum PlatformPulseQueryOwnerState {
    BeforeInitial,
    AwaitingPublication(ExpectedPublication),
    Live,
    Indeterminate,
    Closed,
}

struct ExpectedPublication {
    identity: String,
    status: String,
    revision: u64,
    owner_order: u64,
    receipt: WorthQueryApplicationQueryPublicationReceipt,
}

pub(crate) enum PlatformPulseQueryActionOutcome {
    Executed {
        evidence: WorthUiScalarProjectionActionEvidence,
        observation: UiProjectionObservation,
    },
    Denied {
        denial: WorthUiScalarProjectionActionPreconditionDenial,
        active_query_source_revision: u64,
        submitted_query_source_revision: u64,
    },
    Indeterminate(PlatformPulseQueryActionIndeterminate),
}

#[derive(Debug)]
pub(crate) enum PlatformPulseQueryActionIndeterminate {
    CurrentRevisionLost,
    DeniedActionCommitted,
    Owner(WorthUiStatusOwnerError),
}

impl std::fmt::Display for PlatformPulseQueryActionIndeterminate {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CurrentRevisionLost => {
                formatter.write_str("the current action lost its Query source revision")
            }
            Self::DeniedActionCommitted => {
                formatter.write_str("the revision-mismatched action unexpectedly committed")
            }
            Self::Owner(error) => write!(formatter, "{error}"),
        }
    }
}

#[derive(Debug)]
pub(crate) enum PlatformPulseQueryLifecycleDenial {
    InitialAlreadyIssued,
    PublicationAlreadyPending,
    PublicationNotPending,
    OwnerNotLive,
    ActionRequest(&'static str),
    Owner(WorthUiStatusOwnerError),
    Projection(WorthUiQueryViewIdentityError),
    ForeignPublication,
    UnsettledPublicationAtClose,
    IndeterminateAtClose,
}

impl std::fmt::Display for PlatformPulseQueryLifecycleDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InitialAlreadyIssued => formatter.write_str("initial fact already issued"),
            Self::PublicationAlreadyPending => formatter.write_str("publication already pending"),
            Self::PublicationNotPending => formatter.write_str("publication is not pending"),
            Self::OwnerNotLive => formatter.write_str("live owner unavailable"),
            Self::ActionRequest(denial) => write!(formatter, "action request: {denial}"),
            Self::Owner(denial) => write!(formatter, "Query owner: {denial}"),
            Self::Projection(denial) => write!(formatter, "projection: {denial:?}"),
            Self::ForeignPublication => formatter.write_str("foreign publication fact"),
            Self::UnsettledPublicationAtClose => {
                formatter.write_str("Query publication had not reached presentation at close")
            }
            Self::IndeterminateAtClose => {
                formatter.write_str("Query effect outcome remained indeterminate at close")
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlatformPulseQueryShutdownReceipt {
    owner_terminal: bool,
}

impl PlatformPulseQueryLifecycle {
    pub(crate) fn new(owner: WorthUiStatusSourceOwner, initial: UiProjectionObservation) -> Self {
        Self {
            owner,
            initial: Some(initial),
            state: PlatformPulseQueryOwnerState::BeforeInitial,
        }
    }

    pub(crate) fn issue_initial(
        &mut self,
    ) -> Result<UiProjectionObservation, PlatformPulseQueryLifecycleDenial> {
        if !matches!(self.state, PlatformPulseQueryOwnerState::BeforeInitial) {
            return Err(PlatformPulseQueryLifecycleDenial::InitialAlreadyIssued);
        }
        let initial = self
            .initial
            .take()
            .ok_or(PlatformPulseQueryLifecycleDenial::InitialAlreadyIssued)?;
        self.retain_awaiting(&initial);
        Ok(initial)
    }

    pub(crate) fn advance(
        &mut self,
        record: WorthUiScalarProjectionSourceRecord,
    ) -> Result<UiProjectionObservation, PlatformPulseQueryLifecycleDenial> {
        self.require_live()?;
        let publication = match self.owner.publish_source(record) {
            Ok(publication) => publication,
            Err(denial) => {
                self.state = PlatformPulseQueryOwnerState::Indeterminate;
                return Err(PlatformPulseQueryLifecycleDenial::Owner(denial));
            }
        };
        let observation = publication
            .into_projection_observation()
            .map_err(PlatformPulseQueryLifecycleDenial::Projection)?;
        self.retain_awaiting(&observation);
        Ok(observation)
    }

    pub(crate) fn execute_current_action(
        &mut self,
        status: impl Into<String>,
        idempotency_session: u64,
        idempotency_lineage: u64,
    ) -> Result<PlatformPulseQueryActionOutcome, PlatformPulseQueryLifecycleDenial> {
        let source_revision = self.current_source_revision()?;
        let action = WorthUiStatusActionRequest::new(
            source_revision,
            status,
            idempotency_session,
            idempotency_lineage,
        )
        .map_err(PlatformPulseQueryLifecycleDenial::ActionRequest)?;
        let execution = match self.owner.execute_action(action) {
            Ok(WorthUiStatusActionOutcome::Committed(execution)) => execution,
            Ok(WorthUiStatusActionOutcome::DeniedRevisionMismatch { .. }) => {
                self.state = PlatformPulseQueryOwnerState::Indeterminate;
                return Ok(PlatformPulseQueryActionOutcome::Indeterminate(
                    PlatformPulseQueryActionIndeterminate::CurrentRevisionLost,
                ));
            }
            Err(detail) => {
                self.state = PlatformPulseQueryOwnerState::Indeterminate;
                return Ok(PlatformPulseQueryActionOutcome::Indeterminate(
                    PlatformPulseQueryActionIndeterminate::Owner(detail),
                ));
            }
        };
        let (publication, evidence) = execution.into_parts();
        let observation = publication
            .into_projection_observation()
            .map_err(PlatformPulseQueryLifecycleDenial::Projection)?;
        self.retain_awaiting(&observation);
        Ok(PlatformPulseQueryActionOutcome::Executed {
            evidence,
            observation,
        })
    }

    pub(crate) fn execute_denied_action(
        &mut self,
        status: impl Into<String>,
        idempotency_session: u64,
        idempotency_lineage: u64,
    ) -> Result<PlatformPulseQueryActionOutcome, PlatformPulseQueryLifecycleDenial> {
        self.require_live()?;
        let submitted_revision = self.current_source_revision()?.checked_add(1).unwrap_or(0);
        let action = WorthUiStatusActionRequest::new(
            submitted_revision,
            status,
            idempotency_session,
            idempotency_lineage,
        )
        .map_err(PlatformPulseQueryLifecycleDenial::ActionRequest)?;
        match self.owner.execute_action(action) {
            Ok(WorthUiStatusActionOutcome::DeniedRevisionMismatch {
                active_revision,
                submitted_revision,
                ..
            }) => Ok(PlatformPulseQueryActionOutcome::Denied {
                denial: WorthUiScalarProjectionActionPreconditionDenial::SourceRevisionMismatch,
                active_query_source_revision: active_revision,
                submitted_query_source_revision: submitted_revision,
            }),
            Ok(WorthUiStatusActionOutcome::Committed(_)) => {
                self.state = PlatformPulseQueryOwnerState::Indeterminate;
                Ok(PlatformPulseQueryActionOutcome::Indeterminate(
                    PlatformPulseQueryActionIndeterminate::DeniedActionCommitted,
                ))
            }
            Err(detail) => {
                self.state = PlatformPulseQueryOwnerState::Indeterminate;
                Ok(PlatformPulseQueryActionOutcome::Indeterminate(
                    PlatformPulseQueryActionIndeterminate::Owner(detail),
                ))
            }
        }
    }

    fn current_source_revision(&self) -> Result<u64, PlatformPulseQueryLifecycleDenial> {
        self.require_live()?;
        self.owner
            .read_status()
            .map(|publication| publication.value().revision)
            .map_err(PlatformPulseQueryLifecycleDenial::Owner)
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
mod tests {
    use super::{PlatformPulseQueryLifecycle, PlatformPulseQueryLifecycleDenial};
    use worth_ui::facade::query_binding::{
        UiProjectionObservation, WorthUiScalarProjectionSourceRecord, WorthUiStatusSourceOwner,
    };

    #[test]
    fn committed_query_update_cannot_report_clean_close_before_presentation() {
        let owner = WorthUiStatusSourceOwner::install().expect("install authored Query owner");
        let (_, initial) = owner
            .initial_projection()
            .expect("issue initial Query fact");
        let mut lifecycle = PlatformPulseQueryLifecycle::new(owner, initial);
        let UiProjectionObservation::ApplicationScalar(initial) = lifecycle
            .issue_initial()
            .expect("issue the initial observation")
        else {
            panic!("the authored Query owner must issue an application scalar fact");
        };
        lifecycle
            .admit_publication(initial)
            .expect("admit the exact initial fact");
        let pending = lifecycle
            .advance(
                WorthUiScalarProjectionSourceRecord::new("ONLINE", 1)
                    .expect("first status revision"),
            )
            .expect("Query commits and issues the next fact");
        assert!(matches!(
            pending,
            UiProjectionObservation::ApplicationScalar(_)
        ));
        assert!(matches!(
            lifecycle.close(),
            Err(PlatformPulseQueryLifecycleDenial::UnsettledPublicationAtClose)
        ));
    }
}
